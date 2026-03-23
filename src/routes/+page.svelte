<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import MapPicker from "$lib/components/MapPicker.svelte";
  import PhotoPreview from "$lib/components/PhotoPreview.svelte";
  import Settings from "$lib/components/Settings.svelte";

  // === Theme ===
  function applyTheme(theme) {
    const html = document.documentElement;
    if (theme === "dark") {
      html.setAttribute("data-theme", "dark");
    } else if (theme === "light") {
      html.setAttribute("data-theme", "light");
    } else {
      // system
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      html.setAttribute("data-theme", prefersDark ? "dark" : "light");
    }
  }

  onMount(async () => {
    // Load theme from settings on startup
    try {
      const settings = await invoke("load_settings");
      applyTheme(settings.theme || "system");
    } catch (e) {
      applyTheme("system");
    }

    // Listen for OS theme changes (for "system" mode)
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", async () => {
      try {
        const settings = await invoke("load_settings");
        if (settings.theme === "system") applyTheme("system");
      } catch (_) {}
    });

    // Listen for theme changes from settings page
    window.addEventListener("theme-change", (e) => {
      applyTheme(e.detail);
    });
  });

  // === State ===
  let files = $state([]);
  let thumbnails = $state({});
  let dragging = $state(false);
  let selectedPaths = $state(new Set());
  let exifCache = $state({}); // path -> ExifData
  let modifiedFiles = $state(new Set());
  let dirtyFields = $state(new Set());
  let mergedFields = $state({});
  let mergedSnapshot = $state({});
  let undoStack = $state([]);
  let mapSavedVersion = $state(0);
  let showSettings = $state(false);
  let showPreview = $state(true);
  let showEditor = $state(true);
  let showMap = $state(true);
  let previewHeight = $state(50); // percentage of right area
  let editorWidth = $state(50);   // percentage of bottom area

  // === File Import ===
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
    if (modifiedFiles.size > 0) {
      if (!confirm("You have unsaved changes. Discard them?")) return;
    }
    files = [];
    selectedPaths = new Set();
    exifCache = {};
    modifiedFiles = new Set();
    dirtyFields = new Set();
    mergedFields = {};
    mergedSnapshot = {};
    undoStack = [];
  }

  // === Selection ===
  function selectFile(file, event) {
    if (event.metaKey || event.ctrlKey) {
      // Toggle selection
      const next = new Set(selectedPaths);
      if (next.has(file.path)) {
        next.delete(file.path);
      } else {
        next.add(file.path);
      }
      selectedPaths = next;
    } else if (event.shiftKey && selectedPaths.size > 0) {
      // Range selection
      const lastSelected = [...selectedPaths].pop();
      const lastIdx = files.findIndex((f) => f.path === lastSelected);
      const curIdx = files.findIndex((f) => f.path === file.path);
      const [start, end] = lastIdx < curIdx ? [lastIdx, curIdx] : [curIdx, lastIdx];
      const next = new Set(selectedPaths);
      for (let i = start; i <= end; i++) {
        next.add(files[i].path);
      }
      selectedPaths = next;
    } else {
      // Single select
      selectedPaths = new Set([file.path]);
    }
    dirtyFields = new Set();
    refreshMergedFields();
  }

  function selectAll() {
    selectedPaths = new Set(files.map((f) => f.path));
    dirtyFields = new Set();
    refreshMergedFields();
  }

  function deselectAll() {
    selectedPaths = new Set();
    dirtyFields = new Set();
    mergedFields = {};
    mergedSnapshot = {};
  }

  function toggleCheckbox(path, event) {
    event.stopPropagation();
    const next = new Set(selectedPaths);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    selectedPaths = next;
    dirtyFields = new Set();
    refreshMergedFields();
  }

  // === EXIF Merged Fields ===
  async function refreshMergedFields() {
    const paths = [...selectedPaths];
    if (paths.length === 0) {
      mergedFields = {};
      mergedSnapshot = {};
      return;
    }

    // Load EXIF for all selected files
    try {
      const dataList = await invoke("get_exif_batch", { paths });
      const newCache = { ...exifCache };
      for (const d of dataList) {
        newCache[d.path] = d;
        if (d.modified) {
          modifiedFiles = new Set([...modifiedFiles, d.path]);
        }
      }
      exifCache = newCache;

      // Compute merged fields
      const merged = {};
      const snapshot = {};
      for (const field of Object.keys(FIELD_LABELS)) {
        const values = dataList.map((d) => d.fields[field] || "");
        const unique = [...new Set(values)];
        if (unique.length === 1) {
          merged[field] = unique[0];
        } else {
          merged[field] = "__MIXED__";
        }

        const snapValues = dataList.map((d) => d.snapshot[field] || "");
        const uniqueSnap = [...new Set(snapValues)];
        if (uniqueSnap.length === 1) {
          snapshot[field] = uniqueSnap[0];
        } else {
          snapshot[field] = "__MIXED__";
        }
      }
      mergedFields = merged;
      mergedSnapshot = snapshot;
    } catch (e) {
      console.error("Load EXIF batch error:", e);
    }
  }

  // === Edit ===
  async function updateField(field, value) {
    const paths = [...selectedPaths];
    if (paths.length === 0) return;

    // Save previous values for undo
    const previousValues = {};
    for (const p of paths) {
      previousValues[p] = exifCache[p]?.fields[field] || null;
    }

    // Push to undo stack
    undoStack = [...undoStack, { type: "edit", files: paths, field, previousValues }];

    try {
      const results = await invoke("update_exif_batch", {
        request: { paths, field, value },
      });
      const newCache = { ...exifCache };
      const newModified = new Set(modifiedFiles);
      for (const d of results) {
        newCache[d.path] = d;
        if (d.modified) {
          newModified.add(d.path);
        } else {
          newModified.delete(d.path);
        }
      }
      exifCache = newCache;
      modifiedFiles = newModified;
    } catch (e) {
      console.error("Update batch error:", e);
    }

    dirtyFields = new Set([...dirtyFields, field]);
    refreshMergedFields();
  }

  function resetField(field) {
    dirtyFields = new Set([...dirtyFields].filter((f) => f !== field));
    // Undo the field change by restoring previous values
    // Just recompute merged from cache
    refreshMergedFields();
  }

  // === Undo ===
  async function undo() {
    if (undoStack.length === 0) return;

    const entry = undoStack[undoStack.length - 1];
    undoStack = undoStack.slice(0, -1);

    if (entry.type === "edit") {
      // Restore previous values for each file
      for (const [path, prevValue] of Object.entries(entry.previousValues)) {
        try {
          await invoke("update_exif_batch", {
            request: {
              paths: [path],
              field: entry.field,
              value: prevValue || "",
            },
          });
        } catch (e) {
          console.error("Undo error:", e);
        }
      }
    } else if (entry.type === "save") {
      // Restore previous snapshots
      const entries = Object.entries(entry.previousSnapshots).map(([path, snapshot]) => ({
        path,
        snapshot,
      }));
      try {
        await invoke("restore_snapshot_batch", { entries });
      } catch (e) {
        console.error("Undo save error:", e);
      }
    }

    // Refresh modified files list
    try {
      const modified = await invoke("get_modified_files");
      modifiedFiles = new Set(modified);
    } catch (e) {
      console.error("Get modified error:", e);
    }

    refreshMergedFields();
  }

  // === Reset ===
  async function reset() {
    const paths = [...selectedPaths];
    if (paths.length === 0) return;

    try {
      const results = await invoke("reset_exif_batch", { paths });
      const newCache = { ...exifCache };
      const newModified = new Set(modifiedFiles);
      for (const d of results) {
        newCache[d.path] = d;
        newModified.delete(d.path);
      }
      exifCache = newCache;
      modifiedFiles = newModified;
      dirtyFields = new Set();
      refreshMergedFields();
    } catch (e) {
      console.error("Reset error:", e);
    }
  }

  // === Save ===
  async function save() {
    const paths = [...selectedPaths].filter((p) => modifiedFiles.has(p));
    if (paths.length === 0) return;

    // Save previous snapshots for undo
    const previousSnapshots = {};
    for (const p of paths) {
      if (exifCache[p]) {
        previousSnapshots[p] = { ...exifCache[p].snapshot };
      }
    }

    try {
      const results = await invoke("save_exif_batch", { paths });
      const newCache = { ...exifCache };
      const newModified = new Set(modifiedFiles);
      for (const d of results) {
        newCache[d.path] = d;
        newModified.delete(d.path);
      }
      exifCache = newCache;
      modifiedFiles = newModified;

      undoStack = [...undoStack, { type: "save", files: paths, previousSnapshots }];
      dirtyFields = new Set();
      mapSavedVersion++;
      refreshMergedFields();
    } catch (e) {
      console.error("Save error:", e);
    }
  }

  // === Map ===
  function getSavedGps() {
    const snap = mergedSnapshot;
    if (!snap || snap.GPSLatitude === "__MIXED__" || snap.GPSLongitude === "__MIXED__") {
      return { lat: null, lng: null };
    }
    return parseGps(snap);
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

  async function handleMapChange({ lat, lng }) {
    if (selectedPaths.size === 0) return;
    const latRef = lat >= 0 ? "N" : "S";
    const lngRef = lng >= 0 ? "E" : "W";
    const absLat = Math.abs(lat).toString();
    const absLng = Math.abs(lng).toString();
    await updateField("GPSLatitude", absLat);
    await updateField("GPSLongitude", absLng);
    await updateField("GPSLatitudeRef", latRef);
    await updateField("GPSLongitudeRef", lngRef);
  }

  // === Drag Reorder ===
  let reorderFrom = $state(null);
  let reorderTo = $state(null);
  let ghostEl = null;

  function handleReorderStart(event, index) {
    event.preventDefault();
    event.stopPropagation();
    reorderFrom = index;

    // Create ghost element
    const file = files[index];
    ghostEl = document.createElement("div");
    ghostEl.className = "drag-ghost";
    const thumbSrc = thumbnails[file.path];
    const isDark = document.documentElement.getAttribute("data-theme") === "dark";
    const bgColor = isDark ? "rgba(42,42,42,0.9)" : "rgba(255,255,255,0.9)";
    const textColor = isDark ? "#f6f6f6" : "#0f0f0f";
    ghostEl.innerHTML = `
      <div style="display:flex;align-items:center;gap:8px;padding:6px 12px;background:${bgColor};color:${textColor};border:1px solid #396cd8;border-radius:6px;box-shadow:0 4px 12px rgba(0,0,0,0.25);pointer-events:none;font-size:13px;backdrop-filter:blur(4px);">
        ${thumbSrc ? `<img src="data:image/jpeg;base64,${thumbSrc}" style="width:32px;height:32px;border-radius:3px;object-fit:cover;" />` : ""}
        <span>${file.filename}</span>
      </div>
    `;
    ghostEl.style.cssText = "position:fixed;z-index:9999;pointer-events:none;";
    ghostEl.style.left = event.clientX + 12 + "px";
    ghostEl.style.top = event.clientY - 16 + "px";
    document.body.appendChild(ghostEl);

    function onMouseMove(e) {
      // Move ghost
      if (ghostEl) {
        ghostEl.style.left = e.clientX + 12 + "px";
        ghostEl.style.top = e.clientY - 16 + "px";
      }

      // Find target index
      const listEl = document.querySelector(".file-list");
      if (!listEl) return;
      const items = listEl.querySelectorAll(".file-item");
      let found = false;
      for (let i = 0; i < items.length; i++) {
        const rect = items[i].getBoundingClientRect();
        if (e.clientY >= rect.top && e.clientY < rect.bottom) {
          reorderTo = i;
          found = true;
          break;
        }
      }
      if (!found) reorderTo = null;
    }

    function onMouseUp() {
      if (reorderFrom !== null && reorderTo !== null && reorderFrom !== reorderTo) {
        const newFiles = [...files];
        const [moved] = newFiles.splice(reorderFrom, 1);
        newFiles.splice(reorderTo, 0, moved);
        files = newFiles;
      }
      reorderFrom = null;
      reorderTo = null;
      if (ghostEl) {
        ghostEl.remove();
        ghostEl = null;
      }
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  // === Splitters ===
  function startHSplit(e) {
    e.preventDefault();
    const rightArea = document.querySelector(".right-area");
    if (!rightArea) return;
    const startY = e.clientY;
    const startHeight = previewHeight;
    const totalH = rightArea.getBoundingClientRect().height;

    function onMove(ev) {
      const delta = ev.clientY - startY;
      const pct = startHeight + (delta / totalH) * 100;
      previewHeight = Math.max(15, Math.min(85, pct));
    }
    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  function startVSplit(e) {
    e.preventDefault();
    const bottomArea = document.querySelector(".bottom-area");
    if (!bottomArea) return;
    const startX = e.clientX;
    const startWidth = editorWidth;
    const totalW = bottomArea.getBoundingClientRect().width;

    function onMove(ev) {
      const delta = ev.clientX - startX;
      const pct = startWidth + (delta / totalW) * 100;
      editorWidth = Math.max(20, Math.min(80, pct));
    }
    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // === Keyboard ===
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
    if ((event.metaKey || event.ctrlKey) && event.key === "a" && files.length > 0) {
      event.preventDefault();
      selectAll();
    }
  }

  // === Helpers ===
  function formatSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function getDisplayValue(field) {
    const val = mergedFields[field];
    if (val === "__MIXED__") return "";
    return val || "";
  }

  function getPlaceholder(field) {
    const val = mergedFields[field];
    if (val === "__MIXED__") return "Mixed";
    return "";
  }

  function hasSelection() {
    return selectedPaths.size > 0;
  }

  function isMultiSelect() {
    return selectedPaths.size > 1;
  }

  function selectionHasModified() {
    return [...selectedPaths].some((p) => modifiedFiles.has(p));
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
        <button onclick={selectAll}>Select All</button>
        <button onclick={deselectAll}>Deselect</button>
        <button onclick={openFiles}>Add Files</button>
        <button onclick={openFolder}>Add Folder</button>
        <button class="panel-toggle" class:active={showPreview} onclick={() => showPreview = !showPreview}>Preview</button>
        <button class="panel-toggle" class:active={showEditor} onclick={() => showEditor = !showEditor}>Editor</button>
        <button class="panel-toggle" class:active={showMap} onclick={() => showMap = !showMap}>Map</button>
        <button class="clear" onclick={clearFiles}>Clear</button>
        <button class="settings-btn" onclick={() => showSettings = true} title="Settings (Cmd+,)">&#9881;</button>
      </div>
    </div>
    <div class="main-content">
      <div class="file-list" class:dragging>
        {#each files as file, index}
          <button
            type="button"
            class="file-item"
            class:selected={selectedPaths.has(file.path)}
            class:modified={modifiedFiles.has(file.path)}
            class:reorder-target={reorderTo === index && reorderFrom !== null && reorderFrom !== index}
            class:reorder-source={reorderFrom === index}
            onclick={(e) => selectFile(file, e)}
          >
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="drag-handle" title="Drag to reorder" onmousedown={(e) => handleReorderStart(e, index)}>≡</span>
            {#if isMultiSelect()}
              <input
                type="checkbox"
                checked={selectedPaths.has(file.path)}
                onclick={(e) => toggleCheckbox(file.path, e)}
              />
            {/if}
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

      <div class="right-area">
        {#if !hasSelection()}
          <div class="no-selection">
            <p>Select one or more photos to view and edit metadata</p>
          </div>
        {:else}
          {#if showPreview}
            <div class="panel preview-panel" style="height: {(showEditor || showMap) ? previewHeight + '%' : '100%'}">
              <div class="panel-header">
                <span>Preview</span>
                <button type="button" class="panel-close" onclick={() => showPreview = false}>✕</button>
              </div>
              <div class="panel-content">
                <PhotoPreview paths={[...selectedPaths]} />
              </div>
            </div>
          {/if}

          {#if showPreview && (showEditor || showMap)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="h-splitter" onmousedown={startHSplit}></div>
          {/if}

          {#if showEditor || showMap}
            <div class="bottom-area" style="height: {showPreview ? (100 - previewHeight) + '%' : '100%'}">
              {#if showEditor}
                <div class="panel editor-panel" style="width: {showMap ? editorWidth + '%' : '100%'}">
                  <div class="panel-header">
                    <span>
                      {#if isMultiSelect()}
                        Editor — {selectedPaths.size} photos
                      {:else}
                        Editor — {files.find(f => selectedPaths.has(f.path))?.filename || ""}
                      {/if}
                    </span>
                    <div class="panel-header-actions">
                      <button onclick={undo} title="Undo (Cmd+Z)" disabled={undoStack.length === 0}>Undo</button>
                      <button onclick={reset} disabled={!selectionHasModified()}>Reset</button>
                      <button onclick={save} title="Save (Cmd+S)" class:primary={selectionHasModified()} disabled={!selectionHasModified()}>Save</button>
                      <button type="button" class="panel-close" onclick={() => showEditor = false}>✕</button>
                    </div>
                  </div>
                  <div class="panel-content editor-scroll">
                    {#each Object.entries(FIELD_LABELS) as [field, label]}
                      <div class="field-row">
                        <label for={field}>{label}</label>
                        {#if EDITABLE_FIELDS.includes(field)}
                          <input
                            id={field}
                            value={getDisplayValue(field)}
                            placeholder={getPlaceholder(field)}
                            onchange={(e) => updateField(field, e.target.value)}
                          />
                        {:else}
                          <span class="field-value">
                            {mergedFields[field] === "__MIXED__" ? "Mixed" : mergedFields[field] || "—"}
                          </span>
                        {/if}
                        {#if dirtyFields.has(field)}
                          <span class="dirty-dot"></span>
                          <button type="button" class="field-reset" onclick={() => resetField(field)} title="Reset field">✕</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if showEditor && showMap}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="v-splitter" onmousedown={startVSplit}></div>
              {/if}

              {#if showMap}
                <div class="panel map-panel" style="width: {showEditor ? (100 - editorWidth) + '%' : '100%'}">
                  <div class="panel-header">
                    <span>Map</span>
                    <button type="button" class="panel-close" onclick={() => showMap = false}>✕</button>
                  </div>
                  <div class="panel-content">
                    <MapPicker
                      lat={getSavedGps().lat}
                      lng={getSavedGps().lng}
                      savedVersion={mapSavedVersion}
                      onchange={handleMapChange}
                    />
                  </div>
                </div>
              {/if}
            </div>
          {/if}
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

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
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
    gap: 6px;
    padding: 6px 8px;
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

  .file-item.reorder-target {
    border-top: 2px solid #396cd8;
  }

  .file-item.reorder-source {
    background: #e0e0e0;
  }

  .file-item.reorder-source .file-thumb,
  .file-item.reorder-source .file-info,
  .file-item.reorder-source .drag-handle {
    opacity: 0.3;
  }

  .drag-handle {
    cursor: grab;
    color: #bbb;
    font-size: 14px;
    flex-shrink: 0;
    user-select: none;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .file-item input[type="checkbox"] {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    cursor: pointer;
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

  .right-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    font-size: 14px;
  }

  .panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .preview-panel {
    flex-shrink: 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid #e0e0e0;
    font-size: 12px;
    font-weight: 600;
    color: #666;
    flex-shrink: 0;
  }

  .panel-header-actions {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .panel-close {
    background: none;
    border: none;
    cursor: pointer;
    color: #888;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .panel-close:hover {
    background: #eee;
    color: #333;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
  }

  .editor-scroll {
    overflow-y: auto;
    padding: 8px 12px;
  }

  .bottom-area {
    display: flex;
    flex-shrink: 0;
    min-height: 0;
  }

  .editor-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .map-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .h-splitter {
    height: 4px;
    cursor: row-resize;
    background: #e0e0e0;
    flex-shrink: 0;
  }

  .h-splitter:hover {
    background: #396cd8;
  }

  .v-splitter {
    width: 4px;
    cursor: col-resize;
    background: #e0e0e0;
    flex-shrink: 0;
  }

  .v-splitter:hover {
    background: #396cd8;
  }

  .panel-toggle {
    font-size: 11px;
    padding: 3px 8px;
  }

  .panel-toggle.active {
    background: #396cd8;
    color: #fff;
    border-color: #396cd8;
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

  .field-row input::placeholder {
    color: #999;
    font-style: italic;
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

  .dirty-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #f59e0b;
    flex-shrink: 0;
  }

  .field-reset {
    padding: 2px 6px;
    font-size: 11px;
    border: 1px solid #ddd;
    background: none;
    cursor: pointer;
    color: #888;
    flex-shrink: 0;
    border-radius: 3px;
  }

  .field-reset:hover {
    color: #e53e3e;
    border-color: #e53e3e;
  }


  :global([data-theme="dark"]) {
    color: #f6f6f6;
    background-color: #1a1a1a;
  }

  :global([data-theme="dark"]) button {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) button.primary {
    background: #396cd8;
    border-color: #396cd8;
  }

  :global([data-theme="dark"]) .toolbar {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .file-list {
    border-right-color: #333;
  }

  :global([data-theme="dark"]) .file-item {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .file-item:hover {
    background: #2a2a2a;
  }

  :global([data-theme="dark"]) .file-item.selected {
    background: #1e2a4a;
  }

  :global([data-theme="dark"]) .drop-zone {
    border-color: #444;
  }

  :global([data-theme="dark"]) .file-thumb,
  :global([data-theme="dark"]) .thumb-placeholder {
    background: #333;
  }

  :global([data-theme="dark"]) .panel-header {
    border-bottom-color: #333;
    color: #999;
  }

  :global([data-theme="dark"]) .panel-close {
    color: #666;
  }

  :global([data-theme="dark"]) .panel-close:hover {
    background: #333;
    color: #ccc;
  }

  :global([data-theme="dark"]) .h-splitter,
  :global([data-theme="dark"]) .v-splitter {
    background: #333;
  }

  :global([data-theme="dark"]) .field-row label {
    color: #999;
  }

  :global([data-theme="dark"]) .field-row input {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .field-row input::placeholder {
    color: #666;
  }

  :global([data-theme="dark"]) .field-value {
    color: #ccc;
  }

  :global([data-theme="dark"]) .field-reset {
    border-color: #444;
    color: #888;
  }

  :global([data-theme="dark"]) .drag-handle {
    color: #555;
  }
</style>
