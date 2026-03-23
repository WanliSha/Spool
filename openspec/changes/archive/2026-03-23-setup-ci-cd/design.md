# Setup CI/CD — Design

## Workflows

### 1. PR Check (`ci.yml`)
- Trigger: pull_request to main
- Matrix: ubuntu-latest, macos-latest, windows-latest
- Steps: install deps → build frontend → cargo check → tauri build

### 2. Release (`release.yml`)
- Trigger: push tag `v*`
- Matrix: same three platforms
- Steps: build → package → upload to GitHub Release
- Uses `tauri-apps/tauri-action` for build + upload

## Platform Dependencies

| Platform | System Dependencies |
|---|---|
| Linux | `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` |
| macOS | None (system WebView) |
| Windows | None (WebView2 bundled) |

Note: `libraw-rs` is not yet in the project, so no libraw deps needed in CI.

## Homebrew Cask
- Deferred to after first release — needs a separate repo `homebrew-spool`
- Can be set up manually after the first tag creates a GitHub Release
