# Image And Release Behavior

## Active CI Shape

`_build-shared.yml`:

1. Installs builder from `crates/builder`.
2. Initializes Node through `fnm` and installs dashboard dependencies with `pnpm install`.
3. Logs in to Harbor through `docker/login-action`.
4. Configures Docker Buildx with registry auth.
5. Checks whether the base image exists through `docker buildx imagetools inspect`.
6. Builds/pushes base image only when missing.
7. Runs `builder image convd <profile> --arch amd,arm --dashboard --registry <registry> --user <owner>`.

## Dockerfile Contracts

- `Dockerfile` expects `BASE_IMAGE`, `NAME`, `VERSION`, `DESCRIPTION`, `URL`, `VENDOR`, `LICENSE`, `BUILD_DATE`, `TARGET_TRIPLE`, and `TARGET_DIR`.
- Runtime binary path is `target/${TARGET_TRIPLE}/${TARGET_DIR}/${NAME}`.
- `base.Dockerfile` creates the `app` user and `/app/.convertor`.

## Registry And Tags

- Active CI registry env is `harbor.internal.bppleman.cn:30443`.
- Base image existence is checked before building.
- Multi-arch manifest creation is builder-owned.
