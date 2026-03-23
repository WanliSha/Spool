# Setup CI/CD

## Summary
Configure GitHub Actions for cross-platform build, test, and release pipeline.

## Scope
- PR workflow: build + test on Linux, macOS, Windows
- Release workflow: triggered on tag push, build + package + upload to GitHub Releases
- Install libraw dependency on each platform in CI
- Use `tauri-apps/tauri-action`
- Create Homebrew tap repo (`homebrew-spool`) with cask definition
- Auto-update cask on release

## Out of Scope
- AUR or other package managers
- Code signing / notarization (can add later)

## References
- Spec: `openspec/specs/ci-cd/spec.md`

## Status
Pending
