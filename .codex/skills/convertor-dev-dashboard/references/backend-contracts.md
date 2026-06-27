# Backend Contracts

## Core Model Mirrors

- `ConvQuery`: Rust `crates/convertor/src/url/conv_query.rs`; dashboard `common/model/core/conv-query.ts`.
- `ProxyClient`: Rust `config/proxy_client.rs`; dashboard `common/model/core/proxy-client.ts`.
- `Policy`: Rust `core/profile/policy.rs`; dashboard `common/model/core/policy.ts`.

## API Model Mirrors

- `UrlResult`: Rust `crates/convd/src/server/model/url_result.rs`; dashboard `common/model/api/url_result.ts`.
- `BackendStatus`: Rust `crates/convd/src/server/model/backend_status.rs`; dashboard `common/model/api/backend-status.ts`.

## Serialization Notes

- Dashboard uses `qs` for query-string generation and parsing.
- Rust backend uses `serde_qs` for `ConvQuery`.
- Dashboard zod schemas are runtime API guards; update schemas with model changes.
- Keep optional backend fields intentionally represented as `null` or optional properties in TypeScript.
