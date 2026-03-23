<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onclose } = $props();

  let settings = $state({
    recursive_folder_loading: false,
    cache_size_limit_mb: 200,
  });
  let cacheStats = $state({ size_bytes: 0, file_count: 0 });
  let clearing = $state(false);

  async function load() {
    try {
      settings = await invoke("load_settings");
      cacheStats = await invoke("get_cache_stats");
    } catch (e) {
      console.error("Load settings error:", e);
    }
  }

  async function save() {
    try {
      await invoke("save_settings", { settings });
    } catch (e) {
      console.error("Save settings error:", e);
    }
  }

  async function handleToggleRecursive() {
    settings.recursive_folder_loading = !settings.recursive_folder_loading;
    await save();
  }

  async function handleCacheLimitChange(e) {
    const val = parseInt(e.target.value);
    if (!isNaN(val) && val > 0) {
      settings.cache_size_limit_mb = val;
      await save();
    }
  }

  async function clearCache() {
    clearing = true;
    try {
      await invoke("clear_cache");
      cacheStats = await invoke("get_cache_stats");
    } catch (e) {
      console.error("Clear cache error:", e);
    }
    clearing = false;
  }

  function formatBytes(bytes) {
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  load();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" onclick={onclose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2>Settings</h2>
      <button type="button" class="close-btn" onclick={onclose}>✕</button>
    </div>
    <div class="modal-body">
      <section>
        <h3>File Import</h3>
        <label class="toggle-row">
          <input
            type="checkbox"
            checked={settings.recursive_folder_loading}
            onchange={handleToggleRecursive}
          />
          <span>Load subfolders when opening a folder</span>
        </label>
      </section>

      <section>
        <h3>Thumbnail Cache</h3>
        <div class="field-row">
          <span>Cache size limit</span>
          <div class="input-group">
            <input
              type="number"
              min="50"
              max="2000"
              value={settings.cache_size_limit_mb}
              onchange={handleCacheLimitChange}
            />
            <span class="unit">MB</span>
          </div>
        </div>
        <div class="field-row">
          <span>Current usage</span>
          <span class="value">{formatBytes(cacheStats.size_bytes)} ({cacheStats.file_count} files)</span>
        </div>
        <div class="field-row">
          <span></span>
          <button type="button" class="clear-btn" onclick={clearCache} disabled={clearing}>
            {clearing ? "Clearing..." : "Clear Cache"}
          </button>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #fff;
    border-radius: 12px;
    width: 450px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 16px;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 18px;
    cursor: pointer;
    color: #666;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #f0f0f0;
  }

  .modal-body {
    padding: 16px;
  }

  section {
    margin-bottom: 20px;
  }

  section:last-child {
    margin-bottom: 0;
  }

  h3 {
    margin: 0 0 10px 0;
    font-size: 13px;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 14px;
  }

  .toggle-row input {
    width: 16px;
    height: 16px;
  }

  .field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
    font-size: 14px;
  }

  .input-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .input-group input {
    width: 70px;
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
    text-align: right;
  }

  .unit {
    color: #888;
    font-size: 13px;
  }

  .value {
    color: #666;
    font-size: 13px;
  }

  .clear-btn {
    padding: 6px 12px;
    border: 1px solid #ccc;
    border-radius: 6px;
    background: #fff;
    cursor: pointer;
    font-size: 13px;
    color: #e53e3e;
  }

  .clear-btn:hover {
    border-color: #e53e3e;
  }

  .clear-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .modal {
      background: #1a1a1a;
    }

    .modal-header {
      border-bottom-color: #333;
    }

    .close-btn {
      color: #999;
    }

    .close-btn:hover {
      background: #333;
    }

    h3 {
      color: #777;
    }

    .input-group input {
      background: #2a2a2a;
      border-color: #444;
      color: #f6f6f6;
    }

    .clear-btn {
      background: #2a2a2a;
      border-color: #444;
    }
  }
</style>
