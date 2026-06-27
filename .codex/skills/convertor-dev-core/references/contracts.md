# Core Contracts

## Rust To Dashboard

Keep these synchronized when changing query or API shape:

- Rust `crates/convertor/src/url/conv_query.rs` and dashboard `dashboard/src/app/common/model/core/conv-query.ts`.
- Rust `ProxyClient` and dashboard `proxy-client.ts`.
- Rust `Policy` and dashboard `policy.ts`.
- Rust `crates/convd/src/server/model/url_result.rs` and dashboard `common/model/api/url_result.ts`.
- Rust backend status model and dashboard `backend-status.ts`.

## Query Serialization

- Rust parses and serializes `ConvQuery` through `serde_qs`.
- Dashboard serializes query strings through `qs`.
- Optional Rust fields that are omitted should map to dashboard `null` or optional fields intentionally.
- `sub_url` is encrypted/decrypted by Rust `Encryptor` and dashboard `EncryptorService`.

## Subscription Routes

`crates/convd/src/server/router/subscription.rs` owns:

- raw profile endpoint
- profile endpoint
- proxy provider endpoint
- rule provider endpoint

Client support differs by endpoint. For example, Clash supports proxy provider output while Surge returns an unsupported-client error for proxy provider.
