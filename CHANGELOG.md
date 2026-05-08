# Changelog

All notable changes to this project are documented in this file.

## [0.0.5] - 2026-05-08

### Added
- Command Builder now accepts arbitrary `cast ...` commands from pasted input and saved templates.
- Forge broadcast preview now validates private-key usage and migrates legacy templates to `--private-key <key>`.

### Changed
- Command Builder now uses a generic `contract_address` placeholder instead of `counter_addr`, while still migrating older saved templates automatically.
- Forge/Anvil RPC defaults in the builder now prefer the selected running Anvil instance and preserve edited RPC values when cycling presets.
- Dashboard copy now distinguishes dry-run `s` flows from broadcast-oriented `x` flows.

## [0.0.4] - 2026-05-07

### Added
- Dual log text mode for long outputs in log panes: default horizontal mode for copy-safe values and wrapped mode toggle (`w`).
- Horizontal scrolling for long log lines in focused log panes with `← / →`.
- Right-click shortcut to switch from mouse-interaction mode to text-selection mode.

### Changed
- Anvil logs and global logs now show a mode badge (`[H]` / `[W]`) and preserve long-value visibility in small panes.
- Job Queue pane width reduced from 32% to 24% for more main workspace space.

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
