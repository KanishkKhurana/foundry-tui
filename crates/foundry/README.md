# foundry-tui-foundry

Process execution layer for Foundry binaries used by `foundry-tui`.

## Responsibilities

- Defines `ToolKind`, `ToolRequest`, `ToolEvent`, and `ToolResult`.
- Spawns and streams process output for:
  - `forge`
  - `cast`
  - `anvil`
  - `chisel`
  - `foundryup`
- Handles cancellation and structured event reporting back to the app controller.

## Notes

- This crate is runtime/process focused and intentionally has no UI/state logic.
