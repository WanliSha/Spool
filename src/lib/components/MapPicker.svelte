<script>
  import { onMount } from "svelte";
  import { untrack } from "svelte";
  import L from "leaflet";
  import "leaflet/dist/leaflet.css";
  import { fetch } from "@tauri-apps/plugin-http";

  let { lat = null, lng = null, savedVersion = 0, onchange } = $props();

  let mapEl;
  let inlineSlot;
  let expandedSlot;
  let map;
  let savedMarker;
  let selectedMarker;
  let selectedLat = $state(null);
  let selectedLng = $state(null);
  let savedLat = $state(lat);
  let savedLng = $state(lng);
  let expanded = $state(false);
  let searchQuery = $state("");
  let searchResults = $state([]);
  let searchTimeout;

  // React to save events: when savedVersion changes, update blue pin
  $effect(() => {
    // Access savedVersion to subscribe to changes
    const _v = savedVersion;
    if (_v === 0) return; // skip initial
    if (!map) return;

    // lat/lng props now reflect the newly saved GPS
    if (lat !== null && lng !== null) {
      savedLat = lat;
      savedLng = lng;
      if (savedMarker) {
        savedMarker.setLatLng([lat, lng]);
      } else {
        savedMarker = L.marker([lat, lng], { icon: savedIcon }).addTo(map);
      }
    }

    // Remove selected marker since it's now saved
    if (selectedMarker) {
      map.removeLayer(selectedMarker);
      selectedMarker = null;
    }
    selectedLat = null;
    selectedLng = null;
  });

  // React to photo change: when lat/lng props change, reset map
  $effect(() => {
    const newLat = lat;
    const newLng = lng;
    if (!map) return;

    // Update saved state
    savedLat = newLat;
    savedLng = newLng;

    // Update or remove saved marker
    if (newLat !== null && newLng !== null) {
      if (savedMarker) {
        savedMarker.setLatLng([newLat, newLng]);
      } else {
        savedMarker = L.marker([newLat, newLng], { icon: savedIcon }).addTo(map);
      }
      map.setView([newLat, newLng], PIN_ZOOM);
    } else {
      if (savedMarker) {
        map.removeLayer(savedMarker);
        savedMarker = null;
      }
      map.setView([DEFAULT_LAT, DEFAULT_LNG], DEFAULT_ZOOM);
    }

    // Clear selected marker (new photo = no unsaved selection)
    if (selectedMarker) {
      map.removeLayer(selectedMarker);
      selectedMarker = null;
    }
    selectedLat = null;
    selectedLng = null;
  });

  const DEFAULT_LAT = 25.033;
  const DEFAULT_LNG = 121.565;
  const DEFAULT_ZOOM = 2;
  const PIN_ZOOM = 15;

  const savedIcon = L.divIcon({
    className: "saved-pin",
    html: '<div style="width:14px;height:14px;border-radius:50%;background:#3b82f6;border:2px solid #fff;box-shadow:0 1px 4px rgba(0,0,0,0.4);"></div>',
    iconSize: [14, 14],
    iconAnchor: [7, 7],
  });

  const selectedIcon = L.divIcon({
    className: "selected-pin",
    html: '<div style="width:14px;height:14px;border-radius:50%;background:#ef4444;border:2px solid #fff;box-shadow:0 1px 4px rgba(0,0,0,0.4);"></div>',
    iconSize: [14, 14],
    iconAnchor: [7, 7],
  });

  function placeSelectedMarker(newLat, newLng) {
    selectedLat = Math.round(newLat * 1000000) / 1000000;
    selectedLng = Math.round(newLng * 1000000) / 1000000;

    if (selectedMarker) {
      selectedMarker.setLatLng([selectedLat, selectedLng]);
    } else {
      selectedMarker = L.marker([selectedLat, selectedLng], {
        icon: selectedIcon,
        draggable: true,
      }).addTo(map);
      selectedMarker.on("dragend", (e) => {
        const pos = e.target.getLatLng();
        selectedLat = Math.round(pos.lat * 1000000) / 1000000;
        selectedLng = Math.round(pos.lng * 1000000) / 1000000;
        notifyChange();
      });
    }
    notifyChange();
  }

  function notifyChange() {
    if (onchange && selectedLat !== null && selectedLng !== null) {
      onchange({ lat: selectedLat, lng: selectedLng });
    }
  }


  async function searchLocation() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }
    try {
      const params = new URLSearchParams({
        q: searchQuery,
        format: "json",
        limit: "5",
      });
      const res = await fetch(
        `https://nominatim.openstreetmap.org/search?${params}`,
        { headers: { "User-Agent": "Spool/0.1.0" } }
      );
      searchResults = await res.json();
    } catch (e) {
      console.error("Search error:", e);
      searchResults = [];
    }
  }

  function handleSearchInput() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(searchLocation, 300);
  }

  function selectSearchResult(result) {
    const resultLat = parseFloat(result.lat);
    const resultLng = parseFloat(result.lon);
    searchResults = [];
    searchQuery = result.display_name;
    // Delay to let dropdown close, then move map
    setTimeout(() => {
      map.invalidateSize();
      map.setView([resultLat, resultLng], PIN_ZOOM);
      placeSelectedMarker(resultLat, resultLng);
    }, 50);
  }

  function toggleExpand() {
    expanded = !expanded;
    // Move the map element to the new container
    setTimeout(() => {
      const target = expanded ? expandedSlot : inlineSlot;
      if (target && mapEl) {
        target.appendChild(mapEl);
        map.invalidateSize();
      }
    }, 50);
  }

  onMount(() => {
    // Place mapEl into inline slot initially
    if (inlineSlot && mapEl) {
      inlineSlot.appendChild(mapEl);
    }

    const hasCoords = lat !== null && lng !== null;
    const initLat = hasCoords ? lat : DEFAULT_LAT;
    const initLng = hasCoords ? lng : DEFAULT_LNG;
    const initZoom = hasCoords ? PIN_ZOOM : DEFAULT_ZOOM;

    map = L.map(mapEl).setView([initLat, initLng], initZoom);

    L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      maxZoom: 19,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a>',
    }).addTo(map);

    if (hasCoords) {
      savedMarker = L.marker([lat, lng], { icon: savedIcon }).addTo(map);
    }

    map.on("click", (e) => {
      placeSelectedMarker(e.latlng.lat, e.latlng.lng);
    });

    setTimeout(() => map.invalidateSize(), 200);

    // Resize observer to handle panel resizing
    const observer = new ResizeObserver(() => {
      if (map) map.invalidateSize();
    });
    if (mapEl) observer.observe(mapEl);

    return () => {
      observer.disconnect();
      map.remove();
    };
  });
</script>

<!-- Hidden persistent map element that gets reparented -->
<div bind:this={mapEl} class="map-el" style="display:none;"></div>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
{#if expanded}
  <div class="map-overlay" onclick={toggleExpand}>
    <div class="map-expanded" onclick={(e) => e.stopPropagation()}>
      <div class="map-toolbar">
        <input
          type="text"
          placeholder="Search location..."
          bind:value={searchQuery}
          oninput={handleSearchInput}
        />
        <button type="button" class="close-btn" onclick={toggleExpand}>✕</button>
        {#if searchResults.length > 0}
          <ul class="search-results">
            {#each searchResults as result}
              <li>
                <button type="button" onclick={() => selectSearchResult(result)}>
                  {result.display_name}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
      <div class="map-slot-expanded" bind:this={expandedSlot}></div>
      <div class="map-footer">
        <div class="legend">
          {#if savedLat !== null}
            <span class="legend-item"><span class="dot blue"></span> Saved</span>
          {/if}
          {#if selectedLat !== null}
            <span class="legend-item"><span class="dot red"></span> Selected</span>
          {/if}
        </div>
        <span class="coords">
          {#if selectedLat !== null}
            Lat: {selectedLat} &nbsp; Lng: {selectedLng}
          {:else if savedLat !== null}
            Lat: {savedLat} &nbsp; Lng: {savedLng}
          {:else}
            Click on the map to select coordinates
          {/if}
        </span>
      </div>
    </div>
  </div>
{/if}

<div class="map-inline">
  <div class="map-toolbar">
    <input
      type="text"
      placeholder="Search location..."
      bind:value={searchQuery}
      oninput={handleSearchInput}
    />
    <button type="button" class="expand-btn" onclick={toggleExpand} title="Expand map">⤢</button>
    {#if searchResults.length > 0}
      <ul class="search-results">
        {#each searchResults as result}
          <li>
            <button type="button" onclick={() => selectSearchResult(result)}>
              {result.display_name}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
  <div class="map-slot-inline" bind:this={inlineSlot}></div>
  <div class="map-footer">
    <div class="legend">
      {#if lat !== null}
        <span class="legend-item"><span class="dot blue"></span> Saved</span>
      {/if}
      {#if selectedLat !== null}
        <span class="legend-item"><span class="dot red"></span> Selected</span>
      {/if}
    </div>
    <span class="coords">
      {#if selectedLat !== null}
        {selectedLat}, {selectedLng}
      {:else if savedLat !== null}
        {savedLat}, {savedLng}
      {:else}
        No GPS
      {/if}
    </span>
  </div>
</div>

<style>
  .map-el {
    width: 100%;
    height: 100%;
    min-height: 0;
    display: block !important;
  }

  .map-inline {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .map-slot-inline {
    flex: 1;
    min-height: 0;
  }

  .map-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .map-expanded {
    background: #fff;
    border-radius: 12px;
    width: 80vw;
    height: 70vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .map-slot-expanded {
    flex: 1;
  }

  .map-toolbar {
    position: relative;
    display: flex;
    gap: 4px;
    padding: 6px;
    border-bottom: 1px solid #e0e0e0;
    flex-shrink: 0;
  }

  .map-toolbar input {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 13px;
    box-sizing: border-box;
  }

  .map-toolbar input:focus {
    outline: none;
    border-color: #396cd8;
  }

  .expand-btn, .close-btn {
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
  }

  .search-results {
    position: absolute;
    top: 100%;
    left: 6px;
    right: 6px;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 6px;
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 180px;
    overflow-y: auto;
    z-index: 1001;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .search-results li button {
    display: block;
    width: 100%;
    padding: 6px 10px;
    text-align: left;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 12px;
    color: #333;
    border-radius: 0;
  }

  .search-results li button:hover {
    background: #f0f0f0;
  }

  .map-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-top: 1px solid #e0e0e0;
    font-size: 11px;
    flex-shrink: 0;
  }

  .legend {
    display: flex;
    gap: 8px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 3px;
    color: #666;
  }

  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot.blue { background: #3b82f6; }
  .dot.red { background: #ef4444; }

  .coords {
    font-family: monospace;
    color: #666;
  }


  :global([data-theme="dark"]) .map-expanded {
    background: #1a1a1a;
  }

  :global([data-theme="dark"]) .map-toolbar {
    border-bottom-color: #333;
  }

  :global([data-theme="dark"]) .map-toolbar input {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .expand-btn,
  :global([data-theme="dark"]) .close-btn {
    background: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .search-results {
    background: #2a2a2a;
    border-color: #444;
  }

  :global([data-theme="dark"]) .search-results li button {
    color: #f6f6f6;
  }

  :global([data-theme="dark"]) .search-results li button:hover {
    background: #333;
  }

  :global([data-theme="dark"]) .map-footer {
    border-top-color: #333;
  }

  :global([data-theme="dark"]) .legend-item,
  :global([data-theme="dark"]) .coords {
    color: #999;
  }
</style>
