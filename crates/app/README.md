# foundry-tui-app

Application state + controller crate for `foundry-tui`.

## Responsibilities

- Defines the core runtime model (`AppModel`, job/anvil/custom modal state).
- Owns `AppController` orchestration for input handling, workflow actions, and job lifecycle.
- Builds and validates Forge builder commands before dispatch.
- Tracks workspace inventory (`.sol` files, foundry config/remappings presence).

## Internal module layout

- `model`: shared public state/types consumed by UI.
- `job_manager`: async process job spawning/cancelation plumbing.
- `controller/{input,actions,anvil,custom,jobs}`: behavior split by concern.
- `parsing`: CLI/token parsing, placeholder handling, validation, key matching.
- `project_inventory`: project filesystem scanning logic.

## Out of scope

- No terminal rendering (handled by `foundry-tui-ui`).
- No process execution primitives (handled by `foundry-tui-foundry`).
