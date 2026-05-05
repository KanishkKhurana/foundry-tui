# Publishing `foundry-tui`

## 1) Preflight

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 2) Public API guard (app/ui crates)

For refactors that should preserve downstream APIs, verify the facade exports in:

- `crates/app/src/lib.rs`
- `crates/ui/src/lib.rs`

Optional mechanical check when you have a baseline commit/tag:

```bash
cargo install cargo-public-api

# run on baseline commit/tag
cargo public-api -p foundry-tui-app > /tmp/app-api.before.txt
cargo public-api -p foundry-tui-ui > /tmp/ui-api.before.txt

# run on release candidate commit
cargo public-api -p foundry-tui-app > /tmp/app-api.after.txt
cargo public-api -p foundry-tui-ui > /tmp/ui-api.after.txt

diff -u /tmp/app-api.before.txt /tmp/app-api.after.txt
diff -u /tmp/ui-api.before.txt /tmp/ui-api.after.txt
```

## 3) Crates.io auth

```bash
cargo login <CRATES_IO_TOKEN>
```

Or set:

```bash
export CARGO_REGISTRY_TOKEN=<CRATES_IO_TOKEN>
```

## 4) Dry-run publish order

```bash
cargo publish --dry-run -p foundry-tui-config
cargo publish --dry-run -p foundry-tui-foundry
cargo publish --dry-run -p foundry-tui-app
cargo publish --dry-run -p foundry-tui-ui
cargo publish --dry-run -p foundry-tui
```

## 5) Real publish order

```bash
cargo publish -p foundry-tui-config
cargo publish -p foundry-tui-foundry
cargo publish -p foundry-tui-app
cargo publish -p foundry-tui-ui
cargo publish -p foundry-tui
```

## 6) Post-publish verify

```bash
cargo install foundry-tui
foundry-tui
```

Manual smoke gate (visual output is not snapshot-tested):

- Launch in a Foundry project and verify tabs/panes render correctly.
- Open command palette and anvil launch prompt; confirm modal background masking.
- Start at least one Anvil instance; verify logs stream and pane focus/scroll behavior.

## 7) Deferred structural work

These are intentionally out of scope for the current app/ui refactor and should be handled as separate changesets:

- Split `crates/config/src/lib.rs` if/when maintainability warrants it.
- Consider renaming `foundry-tui-foundry` to `foundry-tui-process` only with a crate migration plan.
