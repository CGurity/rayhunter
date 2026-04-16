<script lang="ts">
    import { get_config, set_config, test_notification, test_webdav, type Config } from '../utils.svelte';
    import Modal from './Modal.svelte';

    let { shown = $bindable() }: { shown: boolean } = $props();
    let config = $state<Config | null>(null);

    let loading = $state(false);
    let saving = $state(false);
    let testingNotification = $state(false);
    let message = $state('');
    let messageType = $state<'success' | 'error' | null>(null);
    let testMessage = $state('');
    let testMessageType = $state<'success' | 'error' | null>(null);
    let testingWebdav = $state(false);
    let webdavTestMessage = $state('');
    let webdavTestMessageType = $state<'success' | 'error' | null>(null);

    async function load_config() {
        try {
            loading = true;
            config = await get_config();
            message = '';
            messageType = null;
        } catch (error) {
            message = `Failed to load config: ${error}`;
            messageType = 'error';
        } finally {
            loading = false;
        }
    }

    async function save_config() {
        if (!config) return;

        try {
            saving = true;
            await set_config(config);
            message =
                'Config saved successfully! Rayhunter is restarting now. Reload the page in a few seconds.';
            messageType = 'success';
        } catch (error) {
            message = `Failed to save config: ${error}`;
            messageType = 'error';
        } finally {
            saving = false;
        }
    }

    async function send_test_notification() {
        try {
            testingNotification = true;
            testMessage = '';
            testMessageType = null;
            await test_notification();
            testMessage = 'Test notification sent successfully!';
            testMessageType = 'success';
        } catch (error) {
            testMessage = `${error}`;
            testMessageType = 'error';
        } finally {
            testingNotification = false;
        }
    }

    async function send_test_webdav() {
        try {
            testingWebdav = true;
            webdavTestMessage = '';
            webdavTestMessageType = null;
            await test_webdav();
            webdavTestMessage = 'WebDAV connection successful!';
            webdavTestMessageType = 'success';
        } catch (error) {
            webdavTestMessage = `${error}`;
            webdavTestMessageType = 'error';
        } finally {
            testingWebdav = false;
        }
    }

    $effect(() => {
        if (shown && !config) {
            load_config();
        }
    });
</script>

<Modal bind:shown title="Configuration">
    <div class="p-2">
        {#if loading}
            <div class="text-center py-4">Loading config...</div>
        {:else if config}
            <form
                class="space-y-4"
                onsubmit={(e) => {
                    e.preventDefault();
                    save_config();
                }}
            >
                <div>
                    <label for="ui_level" class="block text-sm font-medium text-gray-700 mb-1">
                        Device UI Level
                    </label>
                    <select
                        id="ui_level"
                        bind:value={config.ui_level}
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                    >
                        <option value={0}>Invisible mode</option>
                        <option value={1}>Subtle mode (colored line)</option>
                        <option value={2}>Demo mode (orca gif)</option>
                        <option value={3}>EFF logo</option>
                        <option value={4}>High visibility (full screen color)</option>
                    </select>
                    <p class="text-xs text-gray-500 mt-1">
                        Note: Rayhunter draws over the device's native UI, so some flickering is
                        expected
                    </p>
                </div>

                <div>
                    <label
                        for="key_input_mode"
                        class="block text-sm font-medium text-gray-700 mb-1"
                    >
                        Device Input Mode
                    </label>
                    <select
                        id="key_input_mode"
                        bind:value={config.key_input_mode}
                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                    >
                        <option value={0}>Disable button control</option>
                        <option value={1}>Double-tap power button to start new recording</option>
                    </select>
                </div>

                <div class="space-y-3">
                    <div class="flex items-center">
                        <input
                            id="colorblind_mode"
                            type="checkbox"
                            bind:checked={config.colorblind_mode}
                            class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                        />
                        <label for="colorblind_mode" class="ml-2 block text-sm text-gray-700">
                            Colorblind Mode
                        </label>
                    </div>
                </div>

                <div class="border-t pt-4 mt-6 space-y-3">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Notification Settings</h3>
                    <div>
                        <label for="ntfy_url" class="block text-sm font-medium text-gray-700 mb-1">
                            ntfy URL for Sending Notifications (if unset you will not receive
                            notifications)
                        </label>
                        <input
                            id="ntfy_url"
                            type="url"
                            bind:value={config.ntfy_url}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                        <p class="text-xs text-gray-500 mt-1">
                            Test button below uses the saved configuration URL, not the input above
                        </p>
                    </div>

                    <div>
                        <button
                            type="button"
                            onclick={send_test_notification}
                            disabled={testingNotification}
                            class="bg-rayhunter-blue hover:bg-rayhunter-dark-blue disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold py-2 px-4 rounded-md flex flex-row gap-1 items-center"
                        >
                            {#if testingNotification}
                                <div
                                    class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                                ></div>
                                Sending...
                            {:else}
                                <svg
                                    class="w-4 h-4"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                                    ></path>
                                </svg>
                                Send Test Notification
                            {/if}
                        </button>
                        {#if testMessage}
                            <div
                                class="mt-2 p-2 rounded text-sm {testMessageType === 'error'
                                    ? 'bg-red-100 text-red-700'
                                    : 'bg-green-100 text-green-700'}"
                            >
                                {testMessage}
                            </div>
                        {/if}
                    </div>

                    <div class="space-y-2">
                        <div class="block text-sm font-medium text-gray-700 mb-1">
                            Enabled Notification Types
                        </div>
                        <div class="flex items-center">
                            <input
                                type="checkbox"
                                id="enable_warning_notifications"
                                value="Warning"
                                bind:group={config.enabled_notifications}
                            />
                            <label
                                for="enable_warning_notifications"
                                class="ml-2 block text-sm text-gray-700"
                            >
                                Warnings
                            </label>
                        </div>
                        <div class="flex items-center">
                            <input
                                type="checkbox"
                                id="enable_lowbattery_notifications"
                                value="LowBattery"
                                bind:group={config.enabled_notifications}
                            />
                            <label
                                for="enable_lowbattery_notifications"
                                class="ml-2 block text-sm text-gray-700"
                            >
                                Low Battery
                            </label>
                        </div>
                    </div>
                </div>

                <div class="border-t pt-4 mt-6 space-y-3">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Storage Management</h3>

                    <div>
                        <label
                            for="min_space_to_start_recording_mb"
                            class="block text-sm font-medium text-gray-700 mb-1"
                        >
                            Minimum Space to Start Recording (MB)
                        </label>
                        <input
                            id="min_space_to_start_recording_mb"
                            type="number"
                            min="1"
                            bind:value={config.min_space_to_start_recording_mb}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                        <p class="text-xs text-gray-500 mt-1">
                            Recording will not start if less than this amount of disk space is free
                        </p>
                    </div>

                    <div>
                        <label
                            for="min_space_to_continue_recording_mb"
                            class="block text-sm font-medium text-gray-700 mb-1"
                        >
                            Minimum Space to Continue Recording (MB)
                        </label>
                        <input
                            id="min_space_to_continue_recording_mb"
                            type="number"
                            min="1"
                            bind:value={config.min_space_to_continue_recording_mb}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                        <p class="text-xs text-gray-500 mt-1">
                            Recording will stop automatically if disk space drops below this level
                        </p>
                    </div>
                </div>

                <div class="border-t pt-4 mt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">
                        Analyzer Heuristic Settings
                    </h3>
                    <div class="space-y-3">
                        <div class="flex items-center">
                            <input
                                id="imsi_requested"
                                type="checkbox"
                                bind:checked={config.analyzers.imsi_requested}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label for="imsi_requested" class="ml-2 block text-sm text-gray-700">
                                IMSI Requested Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="connection_redirect_2g_downgrade"
                                type="checkbox"
                                bind:checked={config.analyzers.connection_redirect_2g_downgrade}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label
                                for="connection_redirect_2g_downgrade"
                                class="ml-2 block text-sm text-gray-700"
                            >
                                Connection Redirect 2G Downgrade Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="lte_sib6_and_7_downgrade"
                                type="checkbox"
                                bind:checked={config.analyzers.lte_sib6_and_7_downgrade}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label
                                for="lte_sib6_and_7_downgrade"
                                class="ml-2 block text-sm text-gray-700"
                            >
                                LTE SIB6 and SIB7 Downgrade Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="null_cipher"
                                type="checkbox"
                                bind:checked={config.analyzers.null_cipher}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label for="null_cipher" class="ml-2 block text-sm text-gray-700">
                                Null Cipher Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="nas_null_cipher"
                                type="checkbox"
                                bind:checked={config.analyzers.nas_null_cipher}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label for="nas_null_cipher" class="ml-2 block text-sm text-gray-700">
                                NAS Null Cipher Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="incomplete_sib"
                                type="checkbox"
                                bind:checked={config.analyzers.incomplete_sib}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label for="incomplete_sib" class="ml-2 block text-sm text-gray-700">
                                Incomplete SIB Heuristic
                            </label>
                        </div>

                        <div class="flex items-center">
                            <input
                                id="test_analyzer"
                                type="checkbox"
                                bind:checked={config.analyzers.test_analyzer}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label for="test_analyzer" class="ml-2 block text-sm text-gray-700">
                                Test Heuristic (noisy!)
                            </label>
                        </div>
                        <div class="flex items-center">
                            <input
                                id="diagnostic_analyzer"
                                type="checkbox"
                                bind:checked={config.analyzers.diagnostic_analyzer}
                                class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                            />
                            <label
                                for="diagnostic_analyzer"
                                class="ml-2 block text-sm text-gray-700"
                            >
                                Diagnostic Analyzer
                            </label>
                        </div>
                    </div>
                </div>

                <div class="border-t pt-4 mt-6 space-y-3">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">GPS Settings</h3>
                    <div>
                        <label for="gps_mode" class="block text-sm font-medium text-gray-700 mb-1">GPS Mode</label>
                        <select id="gps_mode" bind:value={config.gps_mode} class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue">
                            <option value={0}>Disabled</option>
                            <option value={1}>Fixed coordinates</option>
                            <option value={2}>API endpoint</option>
                        </select>
                        <p class="text-xs text-gray-500 mt-1">
                            {#if config.gps_mode === 2}
                                POST latitude, longitude, and timestamp to <code>/api/gps</code> from any device on the network.
                            {:else if config.gps_mode === 1}
                                GPS coordinates are fixed to the values below.
                            {:else}
                                GPS is disabled; no coordinates will be tracked.
                            {/if}
                        </p>
                    </div>
                    {#if config.gps_mode === 1}
                        <div>
                            <label for="gps_fixed_latitude" class="block text-sm font-medium text-gray-700 mb-1">Fixed Latitude</label>
                            <input id="gps_fixed_latitude" type="number" min="-90" max="90" step="any" required
                                bind:value={config.gps_fixed_latitude} placeholder="e.g. 37.7749"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue" />
                            <p class="text-xs text-gray-500 mt-1">Decimal degrees, -90 to 90</p>
                        </div>
                        <div>
                            <label for="gps_fixed_longitude" class="block text-sm font-medium text-gray-700 mb-1">Fixed Longitude</label>
                            <input id="gps_fixed_longitude" type="number" min="-180" max="180" step="any" required
                                bind:value={config.gps_fixed_longitude} placeholder="e.g. -122.4194"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue" />
                            <p class="text-xs text-gray-500 mt-1">Decimal degrees, -180 to 180</p>
                        </div>
                    {/if}
                </div>

                <div class="border-t pt-4 mt-6 space-y-3">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">WebDAV Upload</h3>
                    <p class="text-xs text-gray-500">
                        When a WebDAV URL is set, completed recordings are automatically uploaded
                        as PCAP files at the configured interval.
                    </p>

                    <div>
                        <label for="webdav_url" class="block text-sm font-medium text-gray-700 mb-1">
                            WebDAV Server URL
                        </label>
                        <input
                            id="webdav_url"
                            type="url"
                            bind:value={config.webdav_url}
                            placeholder="http://nas.local/recordings"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                        <p class="text-xs text-gray-500 mt-1">
                            Leave empty to disable automatic uploads
                        </p>
                    </div>

                    <div>
                        <label for="webdav_username" class="block text-sm font-medium text-gray-700 mb-1">
                            Username
                        </label>
                        <input
                            id="webdav_username"
                            type="text"
                            bind:value={config.webdav_username}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                    </div>

                    <div>
                        <label for="webdav_password" class="block text-sm font-medium text-gray-700 mb-1">
                            Password
                        </label>
                        <input
                            id="webdav_password"
                            type="password"
                            bind:value={config.webdav_password}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                    </div>

                    <div>
                        <label for="webdav_upload_interval_hours" class="block text-sm font-medium text-gray-700 mb-1">
                            Upload Interval (hours)
                        </label>
                        <input
                            id="webdav_upload_interval_hours"
                            type="number"
                            min="1"
                            bind:value={config.webdav_upload_interval_hours}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-rayhunter-blue"
                        />
                    </div>

                    <div class="flex items-center">
                        <input
                            id="webdav_auto_delete"
                            type="checkbox"
                            bind:checked={config.webdav_auto_delete}
                            class="h-4 w-4 text-rayhunter-blue focus:ring-rayhunter-blue border-gray-300 rounded"
                        />
                        <label for="webdav_auto_delete" class="ml-2 block text-sm text-gray-700">
                            Delete local recording after successful upload
                        </label>
                    </div>

                    <div>
                        <button
                            type="button"
                            onclick={send_test_webdav}
                            disabled={testingWebdav}
                            class="bg-rayhunter-blue hover:bg-rayhunter-dark-blue disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold py-2 px-4 rounded-md flex flex-row gap-1 items-center"
                        >
                            {#if testingWebdav}
                                <div
                                    class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                                ></div>
                                Testing...
                            {:else}
                                <svg
                                    class="w-4 h-4"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                                    ></path>
                                </svg>
                                Test WebDAV Connection
                            {/if}
                        </button>
                        <p class="text-xs text-gray-500 mt-1">
                            Uses the saved configuration, not the values above
                        </p>
                        {#if webdavTestMessage}
                            <div
                                class="mt-2 p-2 rounded text-sm {webdavTestMessageType === 'error'
                                    ? 'bg-red-100 text-red-700'
                                    : 'bg-green-100 text-green-700'}"
                            >
                                {webdavTestMessage}
                            </div>
                        {/if}
                    </div>
                </div>

                <div class="flex gap-2 pt-4">
                    <button
                        type="submit"
                        disabled={saving}
                        class="bg-blue-500 hover:bg-blue-700 disabled:opacity-50 text-white font-bold py-2 px-4 rounded-md flex flex-row gap-1 items-center"
                    >
                        {#if saving}
                            <div
                                class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                            ></div>
                            Saving...
                        {:else}
                            <svg
                                class="w-4 h-4"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M5 13l4 4L19 7"
                                ></path>
                            </svg>
                            Apply and restart
                        {/if}
                    </button>
                </div>
            </form>
            {#if message}
                <div
                    class="mt-4 p-3 rounded {messageType === 'error'
                        ? 'bg-red-100 text-red-700'
                        : 'bg-green-100 text-green-700'}"
                >
                    {message}
                </div>
            {/if}
        {:else}
            <div class="text-center py-4 text-red-600">
                Failed to load configuration. Please try reloading the page.
            </div>
        {/if}
    </div>
</Modal>
