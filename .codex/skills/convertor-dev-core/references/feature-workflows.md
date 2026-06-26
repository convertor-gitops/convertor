# Core Feature Workflows

## Profile Conversion Changes

1. Start in `crates/convertor/src/core/profile`, `parser`, or `renderer`.
2. Update both model and render/parse path when the output shape changes.
3. Add or update `crates/convertor/tests/profile_test.rs` snapshots.
4. If HTTP endpoint output changes, update `crates/convd/tests` snapshots.

## Subscription URL Or Query Changes

1. Start in `crates/convertor/src/url/conv_query.rs`, `conv_url.rs`, and `url_builder.rs`.
2. Update `crates/convd/src/server/router/subscription.rs` only for route/extraction behavior.
3. If dashboard generates or parses the query, load `convertor-dev-dashboard`.
4. Check `crates/convertor/tests/url_builder_test.rs` and dashboard query tests.

## Backend API Changes

1. Start in `crates/convd/src/server/router` and `src/server/service`.
2. Keep request extraction under `src/server/extractor`.
3. Keep response envelopes and API errors under `src/server/response`.
4. Mirror API model changes in dashboard models when used by the UI.

## Confly CLI Changes

1. Start in `crates/confly/src/command`.
2. Use `file_provider` for testable filesystem effects.
3. Update snapshots if command output or generated files change.

## Fetcher Changes

1. Keep transport-level behavior in `crates/fetcher`.
2. Avoid leaking product conversion semantics into fetcher.
3. Use `httpmock` integration tests for request/response behavior.
