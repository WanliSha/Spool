<script>
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import MapPicker from "$lib/components/MapPicker.svelte";
  import Settings from "$lib/components/Settings.svelte";

  let files = $state([]);
  let thumbnails = $state({});
  let dragging = $state(false);
  let selectedFile = $state(null);
  let exifData = $state(null);
  let modifiedFiles = $state(new Set());
  let mapSavedVersion = $state(0);
  let showSettings = $state(false);

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
    const entries = await invoke("scan_paths", { paths });
    files = [...files, ...entries];
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
    selectedFile = null;
    exifData = null;
    modifiedFiles = new Set();
  }

  async function selectFile(file) {
    selectedFile = file;
    try {
      exifData = await invoke("read_exif", { path: file.path });
    } catch (e) {
      console.error("Read EXIF error:", e);
      exifData = { fields: {}, modified: false };
    }
  }

  async function updateField(field, value) {
    if (!selectedFile) return;
    try {
      exifData = await invoke("update_exif", {
        request: { path: selectedFile.path, field, value },
      });
      if (exifData.modified) {
        modifiedFiles = new Set([...modifiedFiles, selectedFile.path]);
      } else {
        const next = new Set(modifiedFiles);
        next.delete(selectedFile.path);
        modifiedFiles = next;
      }
    } catch (e) {
      console.error("Update EXIF error:", e);
    }
  }

  async function undo() {
    if (!selectedFile) return;
    try {
      exifData = await invoke("undo_exif", { path: selectedFile.path });
      if (!exifData.modified) {
        const next = new Set(modifiedFiles);
        next.delete(selectedFile.path);
        modifiedFiles = next;
      }
    } catch (e) {
      console.error("Undo error:", e);
    }
  }

  async function reset() {
    if (!selectedFile) return;
    try {
      exifData = await invoke("reset_exif", { path: selectedFile.path });
      const next = new Set(modifiedFiles);
      next.delete(selectedFile.path);
      modifiedFiles = next;
    } catch (e) {
      console.error("Reset error:", e);
    }
  }

  async function resetAll() {
    try {
      const affected = await invoke("reset_all_exif");
      modifiedFiles = new Set();
      if (selectedFile) {
        exifData = await invoke("read_exif", { path: selectedFile.path });
      }
    } catch (e) {
      console.error("Reset all error:", e);
    }
  }

  async function save() {
    if (!selectedFile) return;
    try {
      exifData = await invoke("save_exif", { path: selectedFile.path });
      const next = new Set(modifiedFiles);
      next.delete(selectedFile.path);
      modifiedFiles = next;
      mapSavedVersion++;
    } catch (e) {
      console.error("Save error:", e);
    }
  }

  async function saveAll() {
    try {
      await invoke("save_all_exif");
      modifiedFiles = new Set();
      if (selectedFile) {
        // Re-read from in-memory state (not file) to get updated snapshot
        exifData = await invoke("save_exif", { path: selectedFile.path });
        mapSavedVersion++;
      }
    } catch (e) {
      console.error("Save all error:", e);
    }
  }

  function formatSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function parseGps(fields) {
    if (!fields) return { lat: null, lng: null };
    const latStr = fields.GPSLatitude;
    const lngStr = fields.GPSLongitude;
    const latRef = fields.GPSLatitudeRef;
    const lngRef = fields.GPSLongitudeRef;
    if (!latStr || !lngStr) return { lat: null, lng: null };
    let lat = parseFloat(latStr);
    let lng = parseFloat(lngStr);
    if (isNaN(lat) || isNaN(lng)) return { lat: null, lng: null };
    if (latRef === "S") lat = -lat;
    if (lngRef === "W") lng = -lng;
    return { lat, lng };
  }

  function getSavedGps() {
    return parseGps(exifData?.snapshot);
  }

  async function handleMapChange({ lat, lng }) {
    if (!selectedFile) return;
    const latRef = lat >= 0 ? "N" : "S";
    const lngRef = lng >= 0 ? "E" : "W";
    const absLat = Math.abs(lat).toString();
    const absLng = Math.abs(lng).toString();
    await updateField("GPSLatitude", absLat);
    await updateField("GPSLongitude", absLng);
    await updateField("GPSLatitudeRef", latRef);
    await updateField("GPSLongitudeRef", lngRef);
  }

  function handleKeydown(event) {
    if ((event.metaKey || event.ctrlKey) && event.key === "z") {
      event.preventDefault();
      undo();
    }
    if ((event.metaKey || event.ctrlKey) && event.key === "s") {
      event.preventDefault();
      save();
    }
    if ((event.metaKey || event.ctrlKey) && event.key === ",") {
      event.preventDefault();
      showSettings = !showSettings;
    }
  }

  const FIELD_LABELS = {
    Make: "Camera Make",
    Model: "Camera Model",
    LensMake: "Lens Make",
    LensModel: "Lens Model",
    LensInfo: "Lens Info",
    Software: "Software",
    Artist: "Artist",
    Copyright: "Copyright",
    ImageDescription: "Description",
    DateTimeOriginal: "Date Taken",
    CreateDate: "Date Created",
    ModifyDate: "Date Modified",
    ISO: "ISO",
    FNumber: "Aperture",
    ExposureTime: "Shutter Speed",
    FocalLength: "Focal Length",
    FocalLengthIn35mmFormat: "Focal Length (35mm)",
    ExposureProgram: "Exposure Program",
    MeteringMode: "Metering Mode",
    Flash: "Flash",
    WhiteBalance: "White Balance",
    ImageWidth: "Width",
    ImageHeight: "Height",
    Orientation: "Orientation",
    GPSLatitudeRef: "GPS Lat Ref",
    GPSLatitude: "GPS Latitude",
    GPSLongitudeRef: "GPS Lng Ref",
    GPSLongitude: "GPS Longitude",
    GPSAltitudeRef: "GPS Alt Ref",
    GPSAltitude: "GPS Altitude",
    UserComment: "Comment",
  };

  const EDITABLE_FIELDS = [
    "Make", "Model", "LensMake", "LensModel", "Software", "Artist", "Copyright",
    "ImageDescription", "DateTimeOriginal", "CreateDate", "ModifyDate",
    "GPSLatitudeRef", "GPSLongitudeRef", "UserComment",
  ];

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

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  {#if files.length === 0}
    <div class="drop-zone-wrapper">
      <div class="top-bar">
        <button class="settings-btn" onclick={() => showSettings = true} title="Settings (Cmd+,)">&#9881;</button>
      </div>
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
    </div>
  {:else}
    <div class="toolbar">
      <span class="file-count">{files.length} file{files.length !== 1 ? "s" : ""}</span>
      <div class="toolbar-buttons">
        <button onclick={openFiles}>Add Files</button>
        <button onclick={openFolder}>Add Folder</button>
        {#if modifiedFiles.size > 0}
          <button onclick={saveAll}>Save All</button>
          <button onclick={resetAll}>Reset All</button>
        {/if}
        <button class="clear" onclick={clearFiles}>Clear</button>
        <button class="settings-btn" onclick={() => showSettings = true} title="Settings">&#9881;</button>
      </div>
    </div>
    <div class="main-content">
      <div class="file-list" class:dragging>
        {#each files as file}
          <button
            type="button"
            class="file-item"
            class:selected={selectedFile?.path === file.path}
            class:modified={modifiedFiles.has(file.path)}
            onclick={() => selectFile(file)}
          >
            <div class="file-thumb">
              {#if thumbnails[file.path]}
                <img src="data:image/jpeg;base64,{thumbnails[file.path]}" alt={file.filename} />
              {:else}
                <div class="thumb-placeholder"></div>
              {/if}
            </div>
            <div class="file-info">
              <span class="file-name">
                {#if modifiedFiles.has(file.path)}<span class="mod-dot"></span>{/if}
                {file.filename}
              </span>
              <span class="file-size">{formatSize(file.size)}</span>
            </div>
          </button>
        {/each}
      </div>

      <div class="editor-panel">
        {#if selectedFile && exifData}
          <div class="editor-header">
            <h3>{selectedFile.filename}</h3>
            <div class="editor-actions">
              <button onclick={undo} title="Undo (Cmd+Z)">Undo</button>
              <button onclick={reset}>Reset</button>
              <button onclick={save} title="Save (Cmd+S)" class:primary={exifData.modified}>Save</button>
            </div>
          </div>
          <div class="editor-fields">
            {#each Object.entries(FIELD_LABELS) as [field, label]}
              <div class="field-row">
                <label for={field}>{label}</label>
                {#if EDITABLE_FIELDS.includes(field)}
                  <input
                    id={field}
                    value={exifData.fields[field] || ""}
                    onchange={(e) => updateField(field, e.target.value)}
                  />
                {:else}
                  <span class="field-value">{exifData.fields[field] || "—"}</span>
                {/if}
              </div>
            {/each}
            <div class="map-section">
              <MapPicker
                lat={getSavedGps().lat}
                lng={getSavedGps().lng}
                savedVersion={mapSavedVersion}
                onchange={handleMapChange}
              />
            </div>
          </div>
        {:else}
          <div class="editor-empty">
            <p>Select a photo to view and edit EXIF data</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if showSettings}
    <Settings onclose={() => showSettings = false} />
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

  .drop-zone-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .top-bar {
    display: flex;
    justify-content: flex-end;
    padding: 8px 16px;
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
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid #ccc;
    background: #fff;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  button:hover {
    border-color: #396cd8;
  }

  button.clear {
    color: #888;
  }

  .settings-btn {
    font-size: 16px;
    padding: 4px 8px;
  }

  button.primary {
    background: #396cd8;
    color: #fff;
    border-color: #396cd8;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid #e0e0e0;
    flex-shrink: 0;
  }

  .file-count {
    font-weight: 500;
  }

  .toolbar-buttons {
    display: flex;
    gap: 6px;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .file-list {
    width: 300px;
    overflow-y: auto;
    border-right: 1px solid #e0e0e0;
    flex-shrink: 0;
    transition: all 0.2s;
  }

  .file-list.dragging {
    background-color: rgba(57, 108, 216, 0.05);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid #eee;
    cursor: pointer;
    width: 100%;
    text-align: left;
    background: none;
    border-radius: 0;
    border-left: none;
    border-right: none;
    border-top: none;
    font-size: inherit;
    color: inherit;
  }

  .file-item:hover {
    background: #f0f0f0;
  }

  .file-item.selected {
    background: #e8eeff;
  }

  .file-thumb {
    flex-shrink: 0;
    width: 40px;
    height: 40px;
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
  }

  .file-name {
    font-weight: 500;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-size {
    color: #888;
    font-size: 11px;
  }

  .mod-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #f59e0b;
    margin-right: 4px;
    vertical-align: middle;
  }

  .editor-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid #e0e0e0;
  }

  .editor-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .editor-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .editor-fields {
    flex: 1;
    overflow-y: auto;
    padding: 8px 16px;
  }

  .field-row {
    display: flex;
    align-items: center;
    padding: 4px 0;
    gap: 8px;
  }

  .field-row label {
    width: 140px;
    flex-shrink: 0;
    font-size: 12px;
    color: #666;
    text-align: right;
  }

  .field-row input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
  }

  .field-row input:focus {
    outline: none;
    border-color: #396cd8;
  }

  .field-value {
    flex: 1;
    font-size: 13px;
    color: #333;
  }

  .map-section {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid #eee;
  }

  .editor-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
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

    button.primary {
      background: #396cd8;
      border-color: #396cd8;
    }

    .toolbar {
      border-bottom-color: #333;
    }

    .file-list {
      border-right-color: #333;
    }

    .file-item {
      border-bottom-color: #333;
    }

    .file-item:hover {
      background: #2a2a2a;
    }

    .file-item.selected {
      background: #1e2a4a;
    }

    .drop-zone {
      border-color: #444;
    }

    .file-thumb, .thumb-placeholder {
      background: #333;
    }

    .editor-header {
      border-bottom-color: #333;
    }

    .field-row label {
      color: #999;
    }

    .field-row input {
      background: #2a2a2a;
      border-color: #444;
      color: #f6f6f6;
    }

    .field-value {
      color: #ccc;
    }

    .map-section {
      border-top-color: #333;
    }
  }
</style>
