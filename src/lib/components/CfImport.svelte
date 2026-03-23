<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  let { oncomplete, oncancel } = $props();

  let scanPhotos = $state([]);
  let cfEntries = $state([]);
  let cfFolderPath = $state("");
  let scanThumbnails = $state({});
  let cfPreviews = $state({});
  let error = $state("");
  let writing = $state(false);

  // Drag state
  let dragSide = $state(null);
  let dragFrom = $state(null);
  let dragTo = $state(null);

  // === Import Scan Folder ===
  async function importScanFolder() {
    const selected = await open({ directory: true });
    if (!selected) return;
    error = "";

    try {
      const entries = await invoke("scan_paths", { paths: [selected] });
      scanPhotos = entries.map((e) => ({ path: e.path, filename: e.filename }));
      // Load thumbnails
      for (const p of scanPhotos) {
        loadScanThumbnail(p.path);
      }
    } catch (e) {
      error = `Scan folder error: ${e}`;
    }
  }

  async function loadScanThumbnail(path) {
    try {
      const data = await invoke("get_thumbnail", { path });
      scanThumbnails = { ...scanThumbnails, [path]: data };
    } catch (_) {}
  }

  // === Import C&F Folder ===
  async function importCfFolder() {
    const selected = await open({ directory: true });
    if (!selected) return;
    error = "";
    cfFolderPath = selected;

    try {
      const entries = await invoke("parse_cf_json", { folderPath: selected });
      cfEntries = entries;
      // Load previews
      for (const e of entries) {
        loadCfPreview(e.image_number);
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function loadCfPreview(imageNumber) {
    try {
      const data = await invoke("get_cf_preview", {
        folderPath: cfFolderPath,
        imageNumber,
      });
      if (data) {
        cfPreviews = { ...cfPreviews, [imageNumber]: data };
      }
    } catch (_) {}
  }

  // === Actions ===
  function reverseScan() {
    scanPhotos = [...scanPhotos].reverse();
  }

  function reverseCf() {
    cfEntries = [...cfEntries].reverse();
  }

  function clearScan() {
    scanPhotos = [];
    scanThumbnails = {};
  }

  function clearCf() {
    cfEntries = [];
    cfPreviews = {};
    cfFolderPath = "";
  }

  function deleteScan(index) {
    scanPhotos = scanPhotos.filter((_, i) => i !== index);
  }

  function deleteCf(index) {
    cfEntries = cfEntries.filter((_, i) => i !== index);
  }

  // === Drag Reorder ===
  function handleDragStart(side, index, event) {
    event.preventDefault();
    event.stopPropagation();
    dragSide = side;
    dragFrom = index;

    function onMouseMove(e) {
      const list = document.querySelector(`.align-panel.${side} .align-list`);
      if (!list) return;
      const items = list.querySelectorAll(".align-item");
      for (let i = 0; i < items.length; i++) {
        const rect = items[i].getBoundingClientRect();
        if (e.clientY >= rect.top && e.clientY < rect.bottom) {
          dragTo = i;
          return;
        }
      }
    }

    function onMouseUp() {
      if (dragFrom !== null && dragTo !== null && dragFrom !== dragTo) {
        if (dragSide === "left") {
          const arr = [...scanPhotos];
          const [moved] = arr.splice(dragFrom, 1);
          arr.splice(dragTo, 0, moved);
          scanPhotos = arr;
        } else {
          const arr = [...cfEntries];
          const [moved] = arr.splice(dragFrom, 1);
          arr.splice(dragTo, 0, moved);
          cfEntries = arr;
        }
      }
      dragSide = null;
      dragFrom = null;
      dragTo = null;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  // === Match Count ===
  function matchCount() {
    return Math.min(scanPhotos.length, cfEntries.length);
  }

  function maxRows() {
    return Math.max(scanPhotos.length, cfEntries.length);
  }

  // === Confirm & Write ===
  async function confirmAndWrite() {
    const count = matchCount();
    if (count === 0) return;

    writing = true;
    error = "";

    const pairs = [];
    for (let i = 0; i < count; i++) {
      pairs.push({
        photo_path: scanPhotos[i].path,
        fields: cfEntries[i].fields,
      });
    }

    try {
      const written = await invoke("write_cf_metadata", { pairs });
      // Switch to normal mode with scan photos loaded
      const photoPaths = scanPhotos.map((p) => p.path);
      oncomplete(photoPaths);
    } catch (e) {
      error = String(e);
      writing = false;
    }
  }
</script>

<div class="cf-import">
  <div class="cf-header">
    <h2>Import from Crown & Flint</h2>
    <button type="button" onclick={oncancel}>Back</button>
  </div>

  {#if error}
    <div class="cf-error">{error}</div>
  {/if}

  <div class="cf-panels">
    <div class="align-panel left">
      <div class="panel-toolbar">
        <span class="panel-title">Scan Photos ({scanPhotos.length})</span>
        <div class="panel-actions">
          <button type="button" onclick={importScanFolder}>Import Folder</button>
          <button type="button" onclick={reverseScan} disabled={scanPhotos.length === 0}>Reverse</button>
          <button type="button" onclick={clearScan} disabled={scanPhotos.length === 0}>Clear</button>
        </div>
      </div>
      <div class="align-list">
        {#each Array(maxRows()) as _, i}
          <div class="align-item" class:empty={i >= scanPhotos.length} class:drag-target={dragSide === "left" && dragTo === i && dragFrom !== i}>
            {#if i < scanPhotos.length}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span class="drag-handle" onmousedown={(e) => handleDragStart("left", i, e)}>≡</span>
              <div class="item-thumb">
                {#if scanThumbnails[scanPhotos[i].path]}
                  <img src="data:image/jpeg;base64,{scanThumbnails[scanPhotos[i].path]}" alt="" />
                {:else}
                  <div class="thumb-placeholder"></div>
                {/if}
              </div>
              <span class="item-name">{scanPhotos[i].filename}</span>
              <button type="button" class="item-delete" onclick={() => deleteScan(i)}>✕</button>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="align-divider"></div>

    <div class="align-panel right">
      <div class="panel-toolbar">
        <span class="panel-title">Crown & Flint ({cfEntries.length})</span>
        <div class="panel-actions">
          <button type="button" onclick={importCfFolder}>Import Folder</button>
          <button type="button" onclick={reverseCf} disabled={cfEntries.length === 0}>Reverse</button>
          <button type="button" onclick={clearCf} disabled={cfEntries.length === 0}>Clear</button>
        </div>
      </div>
      <div class="align-list">
        {#each Array(maxRows()) as _, i}
          <div class="align-item" class:empty={i >= cfEntries.length} class:drag-target={dragSide === "right" && dragTo === i && dragFrom !== i}>
            {#if i < cfEntries.length}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span class="drag-handle" onmousedown={(e) => handleDragStart("right", i, e)}>≡</span>
              <div class="item-thumb">
                {#if cfPreviews[cfEntries[i].image_number]}
                  <img src="data:image/jpeg;base64,{cfPreviews[cfEntries[i].image_number]}" alt="" />
                {:else}
                  <div class="thumb-placeholder">?</div>
                {/if}
              </div>
              <span class="item-info">
                <span class="item-number">#{cfEntries[i].image_number}</span>
                <span class="item-summary">{cfEntries[i].summary}</span>
              </span>
              <button type="button" class="item-delete" onclick={() => deleteCf(i)}>✕</button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>

  <div class="cf-footer">
    <span class="match-info">Matched: {matchCount()} pairs</span>
    <div class="cf-footer-actions">
      <button type="button" onclick={oncancel}>Cancel</button>
      <button
        type="button"
        class="primary"
        onclick={confirmAndWrite}
        disabled={matchCount() === 0 || writing}
      >
        {writing ? "Writing..." : `Confirm & Write (${matchCount()})`}
      </button>
    </div>
  </div>
</div>

<style>
  .cf-import {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: #f6f6f6;
  }

  .cf-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
  }

  .cf-header h2 {
    margin: 0;
    font-size: 16px;
  }

  .cf-error {
    padding: 8px 16px;
    background: #fef2f2;
    color: #dc2626;
    font-size: 13px;
    border-bottom: 1px solid #fecaca;
  }

  .cf-panels {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .align-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .align-divider {
    width: 2px;
    background: #e0e0e0;
    flex-shrink: 0;
  }

  .panel-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid #e0e0e0;
    flex-shrink: 0;
  }

  .panel-title {
    font-size: 13px;
    font-weight: 600;
  }

  .panel-actions {
    display: flex;
    gap: 4px;
  }

  .panel-actions button {
    font-size: 11px;
    padding: 3px 8px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
  }

  .panel-actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .align-list {
    flex: 1;
    overflow-y: auto;
  }

  .align-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid #eee;
    min-height: 52px;
  }

  .align-item.empty {
    background: #f9f9f9;
  }

  .align-item.drag-target {
    border-top: 2px solid #396cd8;
  }

  .drag-handle {
    cursor: grab;
    color: #bbb;
    font-size: 14px;
    user-select: none;
  }

  .item-thumb {
    width: 40px;
    height: 40px;
    border-radius: 4px;
    overflow: hidden;
    background: #e0e0e0;
    flex-shrink: 0;
  }

  .item-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumb-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #999;
    font-size: 16px;
  }

  .item-name {
    flex: 1;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .item-number {
    font-size: 11px;
    color: #888;
  }

  .item-summary {
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-delete {
    background: none;
    border: 1px solid #ddd;
    border-radius: 3px;
    cursor: pointer;
    color: #999;
    font-size: 11px;
    padding: 2px 6px;
    flex-shrink: 0;
  }

  .item-delete:hover {
    color: #e53e3e;
    border-color: #e53e3e;
  }

  .cf-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-top: 1px solid #e0e0e0;
    flex-shrink: 0;
  }

  .match-info {
    font-size: 13px;
    font-weight: 500;
  }

  .cf-footer-actions {
    display: flex;
    gap: 8px;
  }

  .cf-footer-actions button {
    padding: 6px 16px;
    border: 1px solid #ccc;
    border-radius: 6px;
    background: #fff;
    cursor: pointer;
    font-size: 13px;
  }

  .cf-footer-actions button.primary {
    background: #396cd8;
    color: #fff;
    border-color: #396cd8;
  }

  .cf-footer-actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  :global([data-theme="dark"]) .cf-import {
    background: #1a1a1a;
  }

  :global([data-theme="dark"]) .cf-header {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .cf-error {
    background: #2a1a1a;
    border-bottom-color: #4a2a2a;
  }

  :global([data-theme="dark"]) .align-divider {
    background: #333;
  }

  :global([data-theme="dark"]) .panel-toolbar {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .panel-actions button {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .align-item {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .align-item.empty {
    background: #1e1e1e;
  }

  :global([data-theme="dark"]) .item-thumb {
    background: #333;
  }

  :global([data-theme="dark"]) .item-delete {
    border-color: #444;
  }

  :global([data-theme="dark"]) .cf-footer {
    border-top-color: #333;
  }

  :global([data-theme="dark"]) .cf-footer-actions button {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .cf-footer-actions button.primary {
    background: #396cd8;
    border-color: #396cd8;
  }
</style>
