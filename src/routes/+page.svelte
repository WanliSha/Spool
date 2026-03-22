<script>
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";

  let files = $state([]);
  let thumbnails = $state({});
  let dragging = $state(false);

  async function loadThumbnail(path) {
    if (thumbnails[path]) return;
    try {
      const data = await invoke("get_thumbnail", { path });
      thumbnails = { ...thumbnails, [path]: data };
    } catch (e) {
      console.error("Thumbnail error:", e);
    }
  }

  async function importPaths(paths) {
    if (!paths || paths.length === 0) return;
    const entries = await invoke("scan_paths", { paths, recursive: false });
    files = [...files, ...entries];
    // Load thumbnails in background
    for (const entry of entries) {
      loadThumbnail(entry.path);
    }
  }

  async function openFiles() {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Images",
          extensions: [
            "jpg", "jpeg", "tif", "tiff", "png", "webp", "bmp",
            "cr2", "cr3", "nef", "arw", "raf", "dng", "orf", "rw2",
          ],
        },
      ],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await importPaths(paths);
    }
  }

  async function openFolder() {
    const selected = await open({ directory: true });
    if (selected) {
      await importPaths([selected]);
    }
  }

  function clearFiles() {
    files = [];
  }

  function formatSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  // Listen for Tauri drag-and-drop events
  getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === "over") {
      dragging = true;
    } else if (event.payload.type === "drop") {
      dragging = false;
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        importPaths(paths);
      }
    } else if (event.payload.type === "leave") {
      dragging = false;
    }
  });
</script>

<main class="container">
  {#if files.length === 0}
    <div class="drop-zone" class:dragging>
      <div class="drop-content">
        <p class="drop-icon">📂</p>
        <p class="drop-text">Drag photos or folders here</p>
        <p class="drop-or">or</p>
        <div class="buttons">
          <button onclick={openFiles}>Open Files</button>
          <button onclick={openFolder}>Open Folder</button>
        </div>
      </div>
    </div>
  {:else}
    <div class="toolbar">
      <span class="file-count">{files.length} file{files.length !== 1 ? "s" : ""}</span>
      <div class="toolbar-buttons">
        <button onclick={openFiles}>Add Files</button>
        <button onclick={openFolder}>Add Folder</button>
        <button class="clear" onclick={clearFiles}>Clear</button>
      </div>
    </div>
    <div class="file-list" class:dragging>
      {#each files as file}
        <div class="file-item">
          <div class="file-thumb">
            {#if thumbnails[file.path]}
              <img src="data:image/jpeg;base64,{thumbnails[file.path]}" alt={file.filename} />
            {:else}
              <div class="thumb-placeholder"></div>
            {/if}
          </div>
          <div class="file-info">
            <span class="file-name">{file.filename}</span>
            <span class="file-meta">
              <span class="file-size">{formatSize(file.size)}</span>
              <span class="file-path">{file.path}</span>
            </span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    color: #0f0f0f;
    background-color: #f6f6f6;
  }

  .container {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .drop-zone {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px dashed #ccc;
    margin: 16px;
    border-radius: 12px;
    transition: all 0.2s;
  }

  .drop-zone.dragging {
    border-color: #396cd8;
    background-color: rgba(57, 108, 216, 0.05);
  }

  .drop-content {
    text-align: center;
  }

  .drop-icon {
    font-size: 48px;
    margin: 0;
  }

  .drop-text {
    font-size: 18px;
    font-weight: 500;
    margin: 8px 0;
  }

  .drop-or {
    color: #888;
    margin: 8px 0;
  }

  .buttons {
    display: flex;
    gap: 8px;
    justify-content: center;
  }

  button {
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px solid #ccc;
    background: #fff;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
  }

  button:hover {
    border-color: #396cd8;
  }

  button.clear {
    color: #888;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
  }

  .file-count {
    font-weight: 500;
  }

  .toolbar-buttons {
    display: flex;
    gap: 8px;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 0;
    transition: all 0.2s;
  }

  .file-list.dragging {
    background-color: rgba(57, 108, 216, 0.05);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    border-bottom: 1px solid #eee;
  }

  .file-thumb {
    flex-shrink: 0;
    width: 48px;
    height: 48px;
    border-radius: 4px;
    overflow: hidden;
    background: #e0e0e0;
  }

  .file-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumb-placeholder {
    width: 100%;
    height: 100%;
    background: #e0e0e0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .file-name {
    font-weight: 500;
  }

  .file-meta {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .file-size {
    color: #888;
    font-size: 12px;
  }

  .file-path {
    color: #aaa;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #1a1a1a;
    }

    button {
      background: #2a2a2a;
      border-color: #444;
      color: #f6f6f6;
    }

    .toolbar {
      border-bottom-color: #333;
    }

    .file-item {
      border-bottom-color: #333;
    }

    .drop-zone {
      border-color: #444;
    }

    .file-thumb, .thumb-placeholder {
      background: #333;
    }
  }
</style>
