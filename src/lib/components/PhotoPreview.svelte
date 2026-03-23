<script>
  import { invoke } from "@tauri-apps/api/core";

  let { allPaths = [], selectedPaths = [], isMultiSelect = false, onnavigate } = $props();

  let previews = $state({});
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let rotation = $state({});
  let currentIndex = $state(0);
  let isPanning = $state(false);
  let panStart = { x: 0, y: 0, panX: 0, panY: 0 };
  let viewEl;
  let suppressAutoNav = false;
  let previewMode = $state("quick"); // "quick" or "full"
  let loadingFull = $state(false);

  function getStripPaths() {
    return isMultiSelect ? selectedPaths : allPaths;
  }

  // Load previews
  $effect(() => {
    if (isMultiSelect) {
      for (const p of selectedPaths) {
        if (p && !previews[`${p}:quick`]) loadPreview(p, "quick");
      }
    } else {
      const paths = allPaths;
      for (let i = Math.max(0, currentIndex - 1); i <= Math.min(paths.length - 1, currentIndex + 1); i++) {
        const p = paths[i];
        if (p && !previews[`${p}:quick`]) loadPreview(p, "quick");
      }
    }
  });

  // Sync currentIndex when selection changes
  let lastSelKey = "";
  $effect(() => {
    const sel = selectedPaths;
    const multi = isMultiSelect;
    const key = JSON.stringify(sel) + multi;
    if (key === lastSelKey) return;
    lastSelKey = key;
    resetView();
    if (!multi && sel.length === 1) {
      const idx = allPaths.indexOf(sel[0]);
      if (idx >= 0) currentIndex = idx;
    }
  });

  async function loadPreview(path, mode = "quick") {
    const key = `${path}:${mode}`;
    if (previews[key]) return;
    try {
      const data = await invoke("get_preview", { path, mode });
      previews = { ...previews, [key]: data };
    } catch (e) {
      console.error("Preview error:", e);
    }
  }

  function getPreviewData(path) {
    // Try full first if in full mode, fallback to quick
    if (previewMode === "full") {
      const full = previews[`${path}:full`];
      if (full) return full;
    }
    return previews[`${path}:quick`] || null;
  }

  async function toggleMode() {
    if (isMultiSelect) return;
    if (previewMode === "quick") {
      previewMode = "full";
      const p = currentPath();
      if (p) {
        loadingFull = true;
        await loadPreview(p, "full");
        loadingFull = false;
      }
    } else {
      previewMode = "quick";
    }
  }

  function resetView() {
    zoom = 1;
    panX = 0;
    panY = 0;
  }

  function currentPath() {
    return allPaths[currentIndex] || null;
  }

  function currentFilename() {
    const p = currentPath();
    if (!p) return "";
    return p.split("/").pop() || p;
  }

  function positionLabel() {
    if (isMultiSelect) return `${selectedPaths.length} photos selected`;
    if (allPaths.length === 0) return "";
    return `${currentFilename()} (${currentIndex + 1}/${allPaths.length})`;
  }

  function navigateTo(idx) {
    const paths = allPaths;
    if (idx < 0 || idx >= paths.length) return;
    currentIndex = idx;
    previewMode = "quick";
    resetView();
    suppressAutoNav = true;
    if (onnavigate && paths[idx]) onnavigate(paths[idx]);
    setTimeout(() => { suppressAutoNav = false; }, 100);
    // Preload adjacent
    for (let i = Math.max(0, idx - 1); i <= Math.min(paths.length - 1, idx + 1); i++) {
      const p = paths[i];
      if (p && !previews[p]) loadPreview(p);
    }
  }

  function prev() { navigateTo(currentIndex - 1); }
  function next() { navigateTo(currentIndex + 1); }

  function zoomIn() { applyZoom(1.25, null); }
  function zoomOut() { applyZoom(0.8, null); }

  function applyZoom(factor, mouseEvent) {
    const newZoom = Math.max(0.1, Math.min(10, zoom * factor));
    if (mouseEvent && viewEl) {
      // Zoom centered on cursor
      const rect = viewEl.getBoundingClientRect();
      const mx = mouseEvent.clientX - rect.left;
      const my = mouseEvent.clientY - rect.top;
      // Adjust pan to keep the point under cursor stationary
      const cx = rect.width / 2;
      const cy = rect.height / 2;
      const dx = mx - cx;
      const dy = my - cy;
      const scale = newZoom / zoom;
      panX = panX * scale - dx * (scale - 1);
      panY = panY * scale - dy * (scale - 1);
    }
    zoom = newZoom;
  }

  function rotateCW() {
    const p = currentPath();
    if (!p) return;
    rotation = { ...rotation, [p]: ((rotation[p] || 0) + 90) % 360 };
  }

  function rotateCCW() {
    const p = currentPath();
    if (!p) return;
    rotation = { ...rotation, [p]: ((rotation[p] || 0) - 90 + 360) % 360 };
  }

  function handleWheel(e) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.92 : 1.08;
    applyZoom(factor, e);
  }

  function handleMouseDown(e) {
    if (e.button !== 0) return;
    isPanning = true;
    panStart = { x: e.clientX, y: e.clientY, panX, panY };
  }

  function handleMouseMove(e) {
    if (!isPanning) return;
    panX = panStart.panX + (e.clientX - panStart.x);
    panY = panStart.panY + (e.clientY - panStart.y);
  }

  function handleMouseUp() {
    isPanning = false;
  }

  function handleDblClick() {
    resetView();
  }

  function handleKeydown(e) {
    if (isMultiSelect) return;
    if (e.key === "ArrowLeft") { e.preventDefault(); prev(); }
    else if (e.key === "ArrowRight") { e.preventDefault(); next(); }
  }

  // Auto-layout grid for multi-select
  function gridStyle() {
    const count = selectedPaths.length;
    if (count <= 1) return "";
    const cols = Math.ceil(Math.sqrt(count));
    return `grid-template-columns: repeat(${cols}, 1fr);`;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="preview-wrapper">
  <div class="preview-toolbar">
    <div class="nav-group">
      {#if !isMultiSelect}
        <button type="button" onclick={prev} disabled={currentIndex === 0}>◀</button>
      {/if}
      <span class="position-label">{positionLabel()}</span>
      {#if !isMultiSelect}
        <button type="button" onclick={next} disabled={currentIndex >= allPaths.length - 1}>▶</button>
      {/if}
    </div>
    <div class="action-group">
      <button type="button" onclick={zoomOut}>−</button>
      <span class="zoom-label">{Math.round(zoom * 100)}%</span>
      <button type="button" onclick={zoomIn}>+</button>
      <button type="button" onclick={resetView} title="Reset to 100%">100%</button>
      {#if !isMultiSelect}
        <span class="separator"></span>
        <button type="button" class="mode-toggle" class:active={previewMode === "full"} onclick={toggleMode} title="Full resolution: 1 pixel = 1 screen pixel">
          {loadingFull ? "Loading..." : "1:1"}
        </button>
        <span class="separator"></span>
        <button type="button" onclick={rotateCCW} title="Rotate CCW">↺</button>
        <button type="button" onclick={rotateCW} title="Rotate CW">↻</button>
      {/if}
    </div>
  </div>

  <div
    class="preview-view"
    bind:this={viewEl}
    onwheel={handleWheel}
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    ondblclick={handleDblClick}
  >
    {#if isMultiSelect && selectedPaths.length > 0}
      <div class="zoom-frame" style="transform: translate({panX}px, {panY}px) scale({zoom});">
        <div class="photo-grid" style={gridStyle()}>
          {#each selectedPaths as path}
            <div class="grid-item">
              {#if getPreviewData(path)}
                <img src="data:image/jpeg;base64,{getPreviewData(path)}" alt={path.split("/").pop()} draggable="false" />
              {:else}
                <div class="loading">Loading...</div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {:else if allPaths.length > 0}
      {@const path = currentPath()}
      {@const rot = rotation[path] || 0}
      <div class="zoom-frame" style="transform: translate({panX}px, {panY}px) scale({zoom});">
        {#if path && getPreviewData(path)}
          <img
            src="data:image/jpeg;base64,{getPreviewData(path)}"
            alt={path.split("/").pop()}
            draggable="false"
            style="transform: rotate({rot}deg);"
          />
        {:else}
          <div class="loading">{loadingFull ? "Loading full resolution..." : "Loading..."}</div>
        {/if}
      </div>
    {:else}
      <div class="preview-empty">No photos to preview</div>
    {/if}
  </div>
</div>

<style>
  .preview-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #111;
  }

  .preview-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    flex-shrink: 0;
  }

  .nav-group, .action-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .preview-toolbar button {
    background: #2a2a2a;
    border: 1px solid #444;
    color: #ccc;
    border-radius: 4px;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 13px;
    line-height: 1.4;
  }

  .preview-toolbar button:hover {
    background: #3a3a3a;
    border-color: #666;
  }

  .preview-toolbar button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .position-label {
    color: #999;
    font-size: 12px;
    padding: 0 6px;
    white-space: nowrap;
  }

  .zoom-label {
    color: #999;
    font-size: 11px;
    min-width: 36px;
    text-align: center;
  }

  .mode-toggle.active {
    background: #396cd8;
    border-color: #396cd8;
    color: #fff;
  }

  .separator {
    width: 1px;
    height: 16px;
    background: #444;
    margin: 0 4px;
  }

  .preview-view {
    flex: 1;
    overflow: hidden;
    cursor: grab;
    display: flex;
    align-items: center;
    justify-content: center;
    user-select: none;
    position: relative;
  }

  .preview-view:active {
    cursor: grabbing;
  }

  .zoom-frame {
    transform-origin: center center;
  }

  .zoom-frame img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }

  .photo-grid {
    display: grid;
    gap: 4px;
    padding: 4px;
  }

  .grid-item {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .grid-item img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }

  .preview-empty {
    color: #666;
    font-size: 14px;
  }

  .loading {
    color: #666;
    font-size: 13px;
    padding: 40px;
  }
</style>
