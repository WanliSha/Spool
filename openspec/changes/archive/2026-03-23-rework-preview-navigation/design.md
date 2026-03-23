# Rework Preview Navigation — Design

## PhotoPreview Component

### Props
- `allPaths`: all file paths (for single-select navigation)
- `selectedPaths`: currently selected paths
- `isMultiSelect`: boolean
- `onnavigate`: callback when single-select swipe changes current photo

### State
- `currentIndex`: which photo is currently shown (index into browsable list)
- `zoom`, `panX`, `panY`: view transform
- `rotation`: per-path rotation state (Map<string, number>)

### Browsing
- Single select: browse through `allPaths`, `currentIndex` tracks position
- Multi select: browse through `selectedPaths`
- Left/right arrow buttons or two-finger horizontal scroll to navigate
- Shows one photo at a time (full size), not a strip

### Gestures
- Pinch (ctrlKey wheel): zoom
- Two-finger scroll (no ctrlKey, deltaX): navigate left/right
- Mouse scroll wheel: zoom
- Mouse drag: pan
- Double click: reset zoom/pan

### Toolbar (inside preview)
- Left arrow / Right arrow for navigation
- Filename + position indicator
- +/- zoom buttons
- Rotate CW / CCW buttons (single select only)

### Navigation Callback
- Single select: when user navigates to a different photo, call `onnavigate(newPath)`
- Parent updates selectedPaths, editor, map accordingly
- Multi select: navigation only changes what's shown in preview, no callback
