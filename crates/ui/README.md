# foundry-tui-ui

Ratatui rendering + terminal interaction crate for `foundry-tui`.

## Responsibilities

- Terminal lifecycle (`init_terminal`, raw mode, mouse capture restore).
- Theme mapping and shared visual style primitives.
- Hover hit-testing for pane focus.
- Rendering of header/tabs/panels/modals/palette/footer.

## Internal module layout

- `theme`, `terminal`, `hit_test`, `layout_utils`, `style_utils`
- `render/{header,tabs,panels,modals,palette,footer}`

## Notes

- This crate consumes `foundry-tui-app` model state and renders it.
- It does not manage jobs/processes directly.
