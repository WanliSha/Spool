# Settings Spec

## Overview
A settings page (Cmd+,) to centralise all user-configurable options. Accessible from both drop zone and toolbar.

## Settings

| Setting | Default | Section |
|---|---|---|
| Theme (Light / Dark / System) | System | Appearance |
| Load subfolders when opening a folder | Off | File Import |
| Thumbnail cache size limit | 200 MB | Thumbnail Cache |
| Clear thumbnail cache (action) | — | Thumbnail Cache |
| Custom metadata fields (add/remove) | Empty | Custom Fields |

## Custom Fields

- User defines field name in settings
- Stored in XMP under `spool:` namespace (e.g. `spool:FilmStock`)
- Appears in editor panel under "Custom" section
- Persisted to settings.json

## Persistence

- Settings stored in OS-standard config directory as `settings.json`
  - macOS: `~/Library/Application Support/Spool/settings.json`
  - Linux: `~/.config/spool/settings.json`
  - Windows: `%APPDATA%\Spool\settings.json`
- Settings survive app restarts
