# Builder CI Contracts

## GitHub Actions

`.github/workflows/_build-shared.yml` currently expects:

- `cargo install --path crates/builder` to install the CLI as `builder`.
- `builder tag base <profile> --registry <registry> --user <owner>` to print the base image tag.
- `builder image base <profile> --arch amd,arm --registry <registry> --user <owner>` to build/push the base image when missing.
- `builder image convd <profile> --arch amd,arm --dashboard --registry <registry> --user <owner>` to build/push convd and manifests.

## Dockerfile Arguments

`commands/image.rs` must keep generated build args aligned with `Dockerfile` and `base.Dockerfile`:

- `BASE_IMAGE`
- `NAME`
- `VERSION`
- `DESCRIPTION`
- `URL`
- `VENDOR`
- `LICENSE`
- `BUILD_DATE`
- `TARGET_TRIPLE`
- `TARGET_DIR`

## Local Entrypoints

`justfile` contains image helper commands around image build/push behavior. Check it when builder image commands or registry defaults change.
