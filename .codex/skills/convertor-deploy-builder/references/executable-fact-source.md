# Builder As Executable Fact Source

Treat `crates/builder` as the executable source of truth for convertor build and release commands.

## What This Means

- Read builder code to understand command construction.
- Run non-mutating builder commands to confirm current behavior.
- Let CI/CD references describe when builder is called and how to interpret its output.
- Keep skill scripts as wrappers around builder checks, not replacements for builder logic.

## Useful Safe Signals

- `cargo run -p builder -- --help` shows the top-level command tree.
- `cargo run -p builder -- image --help` shows image build arguments and options.
- `cargo run -p builder -- tag ...` prints remote tag values without pushing.
- `cargo check -p builder` validates builder code without running release operations.

## Future Direction

If builder gains `--plan`, `--dry-run`, or `--format json`, use that output for release-plan review before any mutating operation.

A structured plan should make side effects explicit, such as:

- cargo build commands
- dashboard build commands
- docker build commands
- remote tags
- docker push operations
- manifest creation

Do not infer these from duplicated shell logic when builder can expose them directly.
