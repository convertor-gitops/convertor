# Pipeline Map

## GitHub Actions

- `.github/workflows/build.yml`: release workflow triggered by `v*` tags and manual dispatch; delegates to `_build-shared.yml` with `profile: release`.
- `.github/workflows/build-debug.yml`: debug workflow entrypoint; check this when debug image behavior changes.
- `.github/workflows/_build-shared.yml`: shared self-hosted macOS ARM64 pipeline for checkout, installing builder, Node/fnm setup, dashboard dependency install, Harbor login, Buildx setup, base image check/build, and convd multi-arch image build/push.
- `.github/archived`: historical workflows; do not treat as active without user direction.

## Deployment Files

- `Dockerfile`: convd runtime image, copies built binary from target triple/profile path.
- `base.Dockerfile`: Alpine base image with certificates, tzdata, app user, and `/app` home.
- `compose/compose.yaml`: runtime compose deployment for nginx-proxy-manager and convd.
- `compose/DEPLOY.md`: deployment notes.
- `justfile`: local image helper commands around `conv image`.

## Builder Dependency

The active workflow installs `crates/builder` and invokes `builder tag` and `builder image`. Treat builder as the executable fact source for command semantics. For command tree inspection, tag validation, or plan/dry-run evolution, load `convertor-deploy-builder`.
