# Project Map

Convertor is a Rust workspace plus an Angular dashboard.

## Product Runtime

- `crates/convertor`: core library for config, URL/query modeling, encryption, providers, profile parsing, profile rendering, cache/storage helpers, and telemetry exports.
- `crates/convd`: Axum HTTP server that exposes subscription/profile/raw/provider endpoints, request extraction, API responses, service composition, frontend serving, and backend status.
- `crates/confly`: user-facing CLI/config tooling for local profile files, subscription commands, install assets, and profile patching.
- `crates/fetcher`: HTTP fetch abstraction and request/response transport helpers.
- `dashboard/`: Angular UI for building subscription URLs, showing backend status, and interacting with convd APIs.

## Deployment Tooling

- `crates/builder`: Rust release/build helper used by CI and local release commands.
- `.github/workflows`: GitHub Actions entrypoints and shared build pipeline.
- `Dockerfile` and `base.Dockerfile`: runtime and base image definitions.
- `compose/`: compose deployment files and deployment notes.
- `justfile`: local image helper entrypoints.

## Source-Of-Truth Hints

- Profile parsing/rendering belongs to `crates/convertor`.
- HTTP API routing and response behavior belongs to `crates/convd`.
- Dashboard URL/query serialization mirrors Rust core/backend contracts.
- Builder command semantics belong to `crates/builder`.
- Pipeline orchestration belongs to `.github/workflows` and deployment files.
