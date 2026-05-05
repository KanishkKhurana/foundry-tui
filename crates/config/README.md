# foundry-tui-config

Configuration and template loading crate for `foundry-tui`.

## Responsibilities

- Defines app/workflow/theme/keybinding config schema.
- Loads or bootstraps user config (`~/.config/foundry-tui/config.toml`).
- Loads global + project template files and merges template definitions.
- Ships defaults for workflows, keybindings, and built-in RPC presets.

## Notes

- This crate is intentionally pure data + loading logic.
- It does not execute tools and does not render UI.
