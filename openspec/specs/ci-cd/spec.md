# CI/CD Spec

## Platform
GitHub Actions

## Triggers

### On Pull Request
- Build and test on all three platforms
- Verify compilation succeeds
- Run tests

### On Tag Push (e.g. `v0.1.0`)
- Build on all three platforms
- Package platform-specific installers
- Upload to GitHub Releases
- Update Homebrew tap

## Build Matrix

| OS | Runner | Output | Notes |
|---|---|---|---|
| Linux | ubuntu-latest | .deb, .AppImage | Requires `libwebkit2gtk`, `libraw-dev` |
| macOS | macos-latest | .dmg | libraw via Homebrew in CI |
| Windows | windows-latest | .msi, .exe | libraw via vcpkg or prebuilt |

## Tooling
- `tauri-apps/tauri-action` — official Tauri GitHub Action for build + package

## Distribution

### GitHub Releases
- All platform installers uploaded automatically on tag push

### Homebrew Cask
- Maintain a Homebrew tap: `WanliSha/spool`
- Cask definition auto-updated on release
- User install: `brew tap WanliSha/spool && brew install --cask spool`

### Future (not in scope now)
- AUR (Arch Linux)
- Other package managers
