<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onclose } = $props();

  let settings = $state({
    recursive_folder_loading: false,
    cache_size_limit_mb: 200,
    theme: "system",
    custom_fields: [],
  });
  let cacheStats = $state({ size_bytes: 0, file_count: 0 });
  let clearing = $state(false);
  let newFieldName = $state("");

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

  async function handleThemeChange(e) {
    settings.theme = e.target.value;
    await save();
    // Dispatch event so +page.svelte can apply the theme
    window.dispatchEvent(new CustomEvent("theme-change", { detail: settings.theme }));
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

  async function addCustomField() {
    const name = newFieldName.trim();
    if (!name || settings.custom_fields.some((f) => f.name === name)) return;
    settings.custom_fields = [...settings.custom_fields, { name, field_type: "string" }];
    newFieldName = "";
    await save();
    window.dispatchEvent(new CustomEvent("custom-fields-change"));
  }

  async function removeCustomField(name) {
    settings.custom_fields = settings.custom_fields.filter((f) => f.name !== name);
    await save();
    window.dispatchEvent(new CustomEvent("custom-fields-change"));
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
        <h3>Appearance</h3>
        <div class="theme-selector">
          <label class:active={settings.theme === "light"}>
            <input type="radio" name="theme" value="light" checked={settings.theme === "light"} onchange={handleThemeChange} />
            Light
          </label>
          <label class:active={settings.theme === "dark"}>
            <input type="radio" name="theme" value="dark" checked={settings.theme === "dark"} onchange={handleThemeChange} />
            Dark
          </label>
          <label class:active={settings.theme === "system"}>
            <input type="radio" name="theme" value="system" checked={settings.theme === "system"} onchange={handleThemeChange} />
            System
          </label>
        </div>
      </section>

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

      <section>
        <h3>Custom Fields</h3>
        <p class="section-desc">Custom metadata stored in XMP (spool: namespace)</p>
        {#each settings.custom_fields as cf}
          <div class="custom-field-row">
            <span class="custom-field-name">{cf.name}</span>
            <button type="button" class="remove-field" onclick={() => removeCustomField(cf.name)}>Remove</button>
          </div>
        {/each}
        <div class="add-field-row">
          <input
            type="text"
            placeholder="Field name..."
            bind:value={newFieldName}
            onkeydown={(e) => e.key === "Enter" && addCustomField()}
          />
          <button type="button" onclick={addCustomField} disabled={!newFieldName.trim()}>Add</button>
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

  .theme-selector {
    display: flex;
    gap: 4px;
  }

  .theme-selector label {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
  }

  .theme-selector label:hover {
    border-color: #396cd8;
  }

  .theme-selector label.active {
    border-color: #396cd8;
    background: rgba(57, 108, 216, 0.08);
  }

  .theme-selector input[type="radio"] {
    display: none;
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

  .section-desc {
    font-size: 12px;
    color: #999;
    margin: 0 0 8px 0;
  }

  .custom-field-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
  }

  .custom-field-name {
    font-size: 14px;
  }

  .remove-field {
    font-size: 12px;
    color: #e53e3e;
    background: none;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 2px 8px;
    cursor: pointer;
  }

  .remove-field:hover {
    border-color: #e53e3e;
  }

  .add-field-row {
    display: flex;
    gap: 4px;
    margin-top: 6px;
  }

  .add-field-row input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 13px;
  }

  .add-field-row button {
    padding: 4px 12px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
    font-size: 13px;
  }

  .add-field-row button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  :global([data-theme="dark"]) .modal {
    background: #1a1a1a;
  }

  :global([data-theme="dark"]) .modal-header {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .close-btn {
    color: #999;
  }

  :global([data-theme="dark"]) .close-btn:hover {
    background: #333;
  }

  :global([data-theme="dark"]) h3 {
    color: #777;
  }

  :global([data-theme="dark"]) .input-group input {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .clear-btn {
    background: #2a2a2a;
    border-color: #444;
  }

  :global([data-theme="dark"]) .theme-selector label {
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .theme-selector label.active {
    border-color: #396cd8;
    background: rgba(57, 108, 216, 0.15);
  }

  :global([data-theme="dark"]) .add-field-row input {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .add-field-row button {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .remove-field {
    border-color: #444;
  }
</style>
