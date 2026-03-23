# Add Theme Setting — Design

## Approach
- Add `theme` field to AppSettings: "system" (default), "light", "dark"
- On app load, read theme and apply a class to `<html>`: `data-theme="light"` or `data-theme="dark"`
- For "system", listen to OS preference and update dynamically
- Replace all `@media (prefers-color-scheme: dark)` with `:global([data-theme="dark"])` selectors
- Settings page: radio buttons for Light / Dark / System
