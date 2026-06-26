# Builder Crate Map

`crates/builder` is a Rust CLI that generates and executes release/build commands.

## Entry Points

- `src/main.rs`: parses CLI, asks a command to create `std::process::Command` values, prints each command, executes in order, and fails on the first non-success status.
- `src/commands.rs`: subcommand enum and `Commander` trait.

## Commands

- `commands/build.rs`: cargo build orchestration, target/profile/package handling, optional dashboard preparation for convd.
- `commands/image.rs`: Docker image build, tag, push, and manifest command generation.
- `commands/publish.rs`: cargo publish orchestration and dashboard build preparation for convd publish.
- `commands/dashboard.rs`: dashboard build commands.
- `commands/tag.rs`: image tag calculation.
- `commands/version.rs`: version output.

## Args

- `args/arch.rs`: architecture aliases and target triples/platforms.
- `args/package.rs`: package selection.
- `args/profile.rs`: release/debug profile mapping.
- `args/registry.rs`: registry target mapping.
- `args/tag.rs`: local/remote tag construction.
- `args/target.rs`: native/musl target selection.
- `args/version.rs`: explicit/latest version values.
