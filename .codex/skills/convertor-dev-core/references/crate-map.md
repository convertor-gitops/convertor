# Rust Crate Map

## `crates/convertor`

Core product library.

- `src/core`: parser/renderer/profile model logic.
- `src/url`: `ConvQuery`, `ConvUrl`, URL builder and URL result logic.
- `src/config`: subscription, proxy client, and Redis config models.
- `src/provider`: subscription provider behavior.
- `src/common`: cache, encryption, clap style, one-time helpers, extra wrappers.
- `src/error`: typed domain errors.
- `tests/snapshots`: insta snapshots for URL and profile behavior.

## `crates/convd`

HTTP backend.

- `src/server/router`: Axum routes for API, actuator, subscription, download, frontend.
- `src/server/service`: Clash/Surge/build URL services.
- `src/server/extractor`: request, headers, and scheme extraction.
- `src/server/response`: API and subscription response envelopes/errors.
- `src/server/model`: backend API models mirrored by dashboard API models.
- `tests/snapshots`: endpoint-level output snapshots.

## `crates/confly`

CLI and local config/profile tooling.

- `src/command`: config and subscription command implementations.
- `src/config.rs`: CLI config discovery and client configuration.
- `src/file_provider.rs`: filesystem or in-memory file abstraction.
- `src/profile_patcher.rs`: profile patching behavior.
- `assets/service`: service config assets.

## `crates/fetcher`

HTTP fetch layer.

- `src/client.rs`: request execution.
- `src/request.rs`: request model.
- `src/response.rs`: response model.
- `tests/fetcher_integration_test.rs`: integration behavior.
