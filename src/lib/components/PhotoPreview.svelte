<script>
  import { invoke } from "@tauri-apps/api/core";

  let { paths = [] } = $props();

  let previews = $state({});
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let isPanning = $state(false);
  let panStart = { x: 0, y: 0, panX: 0, panY: 0 };
  let containerEl;

  $effect(() => {
    const newPaths = paths;
    // Load previews for new paths
    for (const p of newPaths) {
      if (!previews[p]) {
        loadPreview(p);
      }
    }
    // Reset zoom/pan when selection changes
    zoom = 1;
    panX = 0;
    panY = 0;
  });

  async function loadPreview(path) {
    try {
      const data = await invoke("get_preview", { path });
      previews = { ...previews, [path]: data };
    } catch (e) {
      console.error("Preview error:", e);
    }
  }

  function handleWheel(e) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(10, zoom * delta));
    zoom = newZoom;
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
    zoom = 1;
    panX = 0;
    panY = 0;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="preview-container"
  bind:this={containerEl}
  onwheel={handleWheel}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
  ondblclick={handleDblClick}
>
  {#if paths.length === 0}
    <div class="preview-empty">No photos selected</div>
  {:else if paths.length === 1}
    <div class="single-preview" style="transform: scale({zoom}) translate({panX / zoom}px, {panY / zoom}px);">
      {#if previews[paths[0]]}
        <img src="data:image/jpeg;base64,{previews[paths[0]]}" alt="Preview" draggable="false" />
      {:else}
        <div class="loading">Loading...</div>
      {/if}
    </div>
  {:else}
    <div class="multi-preview" style="transform: scale({zoom}) translate({panX / zoom}px, {panY / zoom}px);">
      {#each paths as path}
        <div class="multi-item">
          {#if previews[path]}
            <img src="data:image/jpeg;base64,{previews[path]}" alt="Preview" draggable="false" />
          {:else}
            <div class="loading">Loading...</div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .preview-container {
    width: 100%;
    height: 100%;
    overflow: hidden;
    cursor: grab;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #111;
    user-select: none;
  }

  .preview-container:active {
    cursor: grabbing;
  }

  .preview-empty {
    color: #666;
    font-size: 14px;
  }

  .single-preview {
    transform-origin: center center;
    transition: none;
  }

  .single-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }

  .multi-preview {
    display: flex;
    gap: 8px;
    align-items: center;
    transform-origin: center center;
    transition: none;
    padding: 0 16px;
  }

  .multi-item {
    flex-shrink: 0;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .multi-item img {
    max-height: 100%;
    object-fit: contain;
    display: block;
  }

  .loading {
    color: #666;
    font-size: 13px;
    padding: 40px;
  }
</style>
