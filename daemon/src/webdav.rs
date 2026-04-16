use std::sync::Arc;
use std::time::Duration;

use log::{error, info, warn};
use tokio::io::AsyncReadExt;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::diag::DiagDeviceCtrlMessage;
use crate::pcap::{generate_pcap_data, load_gps_records_for_entry};
use crate::server::ServerState;

pub fn run_webdav_upload_worker(
    task_tracker: &TaskTracker,
    state: Arc<ServerState>,
    shutdown_token: CancellationToken,
) {
    let webdav_url = match &state.config.webdav_url {
        Some(url) if !url.is_empty() => url.trim_end_matches('/').to_string(),
        _ => return, // WebDAV not configured, nothing to do
    };
    let username = state.config.webdav_username.clone().unwrap_or_default();
    let password = state.config.webdav_password.clone().unwrap_or_default();
    let auto_delete = state.config.webdav_auto_delete;
    let interval = Duration::from_secs(state.config.webdav_upload_interval_hours * 3600);
    let manage_recording = !state.config.debug_mode;

    task_tracker.spawn(async move {
        let client = reqwest::Client::new();
        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = shutdown_token.cancelled() => break,
            }

            // Stop the current recording so the latest data is included in the upload.
            // In debug mode there is no diag thread, so skip this.
            let was_recording = state.qmdl_store_lock.read().await.current_entry.is_some();
            if was_recording && manage_recording {
                if state
                    .diag_device_ctrl_sender
                    .send(DiagDeviceCtrlMessage::StopRecording)
                    .await
                    .is_err()
                {
                    warn!("WebDAV: failed to send StopRecording, skipping upload cycle");
                    continue;
                }

                // Poll until current_entry is cleared (up to 10 seconds)
                let mut stopped = false;
                for _ in 0..20 {
                    if state.qmdl_store_lock.read().await.current_entry.is_none() {
                        stopped = true;
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                if !stopped {
                    warn!("WebDAV: timed out waiting for recording to stop, skipping upload cycle");
                    // Best-effort restart so measurement continues
                    state
                        .diag_device_ctrl_sender
                        .send(DiagDeviceCtrlMessage::StartRecording { response_tx: None })
                        .await
                        .ok();
                    continue;
                }
                info!("WebDAV: recording stopped for upload cycle");
            }

            // Snapshot which entries need uploading (name + size only — indices can shift as
            // entries are deleted during this loop)
            let entries_to_upload: Vec<(String, usize)> = {
                let store = state.qmdl_store_lock.read().await;
                store
                    .manifest
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(idx, entry)| {
                        !entry.webdav_uploaded
                            && entry.qmdl_size_bytes > 0
                            && store.current_entry != Some(*idx)
                    })
                    .map(|(_, entry)| (entry.name.clone(), entry.qmdl_size_bytes))
                    .collect()
            };

            for (entry_name, qmdl_size_bytes) in entries_to_upload {
                // Re-look up index by name — earlier deletes in this loop can shift indices
                let entry_index = {
                    let store = state.qmdl_store_lock.read().await;
                    match store.entry_for_name(&entry_name) {
                        Some((idx, _)) => idx,
                        None => continue, // deleted concurrently
                    }
                };

                let gps_records = load_gps_records_for_entry(&state, entry_index).await;

                let qmdl_file = {
                    let store = state.qmdl_store_lock.read().await;
                    match store.open_entry_qmdl(entry_index).await {
                        Ok(f) => f,
                        Err(e) => {
                            error!("WebDAV: failed to open QMDL for {entry_name}: {e}");
                            continue;
                        }
                    }
                };

                // Generate PCAP into a buffer via a duplex pipe
                let (mut reader, writer) = tokio::io::duplex(64 * 1024);
                let entry_name_for_task = entry_name.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        generate_pcap_data(writer, qmdl_file, qmdl_size_bytes, gps_records).await
                    {
                        error!(
                            "WebDAV: failed to generate PCAP for {entry_name_for_task}: {e:?}"
                        );
                    }
                });

                let mut pcap_bytes = Vec::new();
                if let Err(e) = reader.read_to_end(&mut pcap_bytes).await {
                    error!("WebDAV: failed to buffer PCAP for {entry_name}: {e}");
                    continue;
                }

                let url = format!("{webdav_url}/{entry_name}.pcapng");
                info!(
                    "WebDAV: uploading {entry_name}.pcapng ({} bytes) to {url}",
                    pcap_bytes.len()
                );

                let result = client
                    .put(&url)
                    .basic_auth(&username, Some(&password))
                    .header("Content-Type", "application/vnd.tcpdump.pcap")
                    .body(pcap_bytes)
                    .send()
                    .await;

                match result {
                    Ok(response) if response.status().is_success() => {
                        info!("WebDAV: successfully uploaded {entry_name}.pcapng");
                        let mut store = state.qmdl_store_lock.write().await;
                        if auto_delete {
                            if let Err(e) = store.delete_entry(&entry_name).await {
                                error!(
                                    "WebDAV: failed to delete {entry_name} after upload: {e}"
                                );
                            } else {
                                info!("WebDAV: deleted {entry_name} after upload");
                            }
                        } else if let Err(e) = store.mark_entry_uploaded(&entry_name).await {
                            error!("WebDAV: failed to mark {entry_name} as uploaded: {e}");
                        }
                    }
                    Ok(response) => {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        warn!("WebDAV: upload of {entry_name} failed (HTTP {status}): {body}");
                    }
                    Err(e) => {
                        warn!("WebDAV: upload of {entry_name} failed: {e}");
                    }
                }
            }

            // Restart recording to minimize disruption
            if was_recording && manage_recording {
                let (tx, rx) = oneshot::channel();
                if state
                    .diag_device_ctrl_sender
                    .send(DiagDeviceCtrlMessage::StartRecording {
                        response_tx: Some(tx),
                    })
                    .await
                    .is_ok()
                {
                    match rx.await {
                        Ok(Ok(())) => info!("WebDAV: recording restarted after upload cycle"),
                        Ok(Err(e)) => error!("WebDAV: failed to restart recording: {e}"),
                        Err(_) => error!("WebDAV: recording restart response channel dropped"),
                    }
                } else {
                    error!("WebDAV: failed to send StartRecording after upload cycle");
                }
            }
        }
    });
}
