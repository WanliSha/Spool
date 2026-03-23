# Add Photo Preview — Design

## UI Layout

```
┌──────────┬──────────────────────────────────┐
│          │ [⊞Preview] [⊞Editor] [⊞Map]     │
│  File    ├──────────────────────────────────┤
│  List    │ Preview                      [✕] │
│          │ ┌──────────────────────────────┐ │
│          │ │    photo (zoom/pan)          │ │
│          │ └──────────────────────────────┘ │
│          ├───────────────┬──────────────────┤
│          │ Editor    [✕] │ Map          [✕] │
│          │ Make: ...     │ ┌────────────┐  │
│          │ Model: ...    │ │  OSM Map   │  │
│          └───────────────┴──────────────────┘
```

## Resizable Panels

Use CSS `resize` or a custom splitter approach:
- Vertical splitter between file list and right area
- Horizontal splitter between preview and editor+map
- Vertical splitter between editor and map
- Each splitter is a 4px draggable bar

## Rust Backend

New command:
```rust
get_preview(path: String) -> String  // returns base64 JPEG, ~2048px long edge
```

Uses `image` crate to decode, resize to 2048px, encode as JPEG quality 85.
Cache preview in memory (HashMap) to avoid re-decoding on subsequent requests.

## Preview Component (PhotoPreview.svelte)

### Single select
- Display full 2048px image
- Scroll wheel = zoom in/out (centered on cursor)
- Mouse drag = pan
- Double click = reset zoom to fit

### Multi select
- Horizontal strip of previews
- Horizontal scroll to slide between photos
- Scroll wheel = synchronized zoom on all photos
- Each photo independently pannable? No — sync pan too for comparison

## Panel Toggle

Toolbar buttons: [⊞ Preview] [⊞ Editor] [⊞ Map]
- Highlighted when panel is open
- Click to toggle
- Panel header has ✕ to close

## Panel States

When panels are closed, remaining panels fill the space:
- All three open: preview top, editor bottom-left, map bottom-right
- Preview closed: editor left, map right (full height)
- Editor closed: preview top, map bottom (full width)
- Map closed: preview top, editor bottom (full width)
- Only one open: fills entire right area
