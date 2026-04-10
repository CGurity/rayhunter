use std::sync::Arc;
use std::time::Duration;

use log::{error, info, warn};
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

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

    task_tracker.spawn(async move {
        let client = reqwest::Client::new();
        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = shutdown_token.cancelled() => break,
            }

            // Snapshot which entries need uploading (name + size only — indices can shift)
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
                // Re-look up index by name; earlier deletes within this iteration can shift indices
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

                // Generate PCAP into a buffer via the duplex pipe
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
                        warn!("WebDAV: upload of {entry_name} failed ({status}): {body}");
                    }
                    Err(e) => {
                        warn!("WebDAV: upload of {entry_name} failed: {e}");
                    }
                }
            }
        }
    });
}
