# Builder Command Workflows

## Execution Model

Each subcommand implements `Commander::create_command` and returns ordered `std::process::Command` values. `main.rs` prints and runs those commands sequentially.

## `builder build`

- Builds a selected package.
- Uses `CommonArgs` for profile/package.
- Uses `Target` for native or musl architecture selection.
- Builds dashboard first when package is `convd` and `--dashboard` is set.

## `builder image`

- For `base`, skip Rust binary build and build the base Docker image.
- For `convd`, build the convd package for requested musl architectures.
- Build each architecture-specific local image.
- Tag each architecture image for the remote registry.
- Push each architecture image.
- Create multi-arch manifests for the explicit version and `latest`.

## `builder publish`

- For `convd`, builds dashboard prod and dev artifacts before `cargo publish`.
- Supports `--dry-run` and `--allow-dirty`.

## `builder tag`

- Computes tag values used by CI when checking whether a base image already exists.

## `builder dashboard`

- Builds dashboard artifacts used by convd/package release flows.
