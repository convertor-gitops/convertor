# Safe Operations

## Safe By Default

These are suitable for routine validation:

```bash
cargo check -p builder
cargo test -p builder
cargo run -p builder -- --help
cargo run -p builder -- build --help
cargo run -p builder -- image --help
cargo run -p builder -- publish --help
cargo run -p builder -- tag --help
cargo run -p builder -- tag base release --registry ghcr --user convertor-gitops
cargo run -p builder -- tag convd release --registry ghcr --user convertor-gitops
```

## Mutating Or Potentially Expensive

Run only after explicit user request:

```bash
cargo run -p builder -- image base release --arch amd,arm --registry ghcr --user convertor-gitops
cargo run -p builder -- image convd release --arch amd,arm --dashboard --registry ghcr --user convertor-gitops
cargo run -p builder -- publish convd release
```

These may build binaries, build dashboard artifacts, build Docker images, push images, create manifests, or publish crates.

## Skill Script Boundary

Skill scripts may call safe builder commands. They must not reimplement builder's cargo, Docker, tag, push, or manifest command generation.
