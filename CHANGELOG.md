# Changelog

All notable changes to this project are documented in this file.

## [0.0.3] - 2026-05-07

### Changed
- Build dashboard onboarding is now always visible and the `?` toggle keybinding was removed.
- Existing configs automatically drop the legacy `toggle-build-onboarding` keybinding on load.
- TUI now starts in text-selection mode by default (`F2` still toggles mouse interaction mode).

## [0.0.2] - 2026-05-05

### Changed
- Redirected crate `documentation` metadata links to GitHub-hosted README files.

## [0.0.1] - 2026-05-05

### Added
- Initial `foundry-tui` release with tabs for build/test/script/anvil/cast/verify/builder/logs/history.
- Multi-instance Anvil management with per-instance streaming logs.
- Forge command builder with preset picker, paste parsing, and form-based editing.
- Built-in Across RPC presets and richer cast target logging.
- Keyboard + mouse navigation, scrolling, focus, and command palette support.
