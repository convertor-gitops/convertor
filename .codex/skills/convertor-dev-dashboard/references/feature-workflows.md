# Dashboard Feature Workflows

## URL Builder Or Query Changes

1. Start with `dashboard/src/app/service/url.service.ts`.
2. Update `dashboard/src/app/common/model/core/conv-query.ts`.
3. Confirm matching Rust fields in `crates/convertor/src/url/conv_query.rs`.
4. Update `dashboard/src/app/common/model/core/conv-query.spec.ts`.

## Backend API Integration

1. Start with the service under `dashboard/src/app/service`.
2. Update API models under `common/model/api`.
3. Confirm matching Rust models under `crates/convd/src/server/model`.
4. Keep error handling aligned with `dashboard-http.error.ts` and generic response wrappers.

## UI Component Changes

1. Inspect existing `.ts`, `.html`, and `.scss` for the component.
2. Keep page-specific components under `page/dashboard`.
3. Keep reusable controls under `page/shared` only when they are genuinely shared.
4. Keep text and layout compact; this dashboard is an operational tool, not a landing page.

## Environment Or Metadata Changes

1. Start with `env.service.ts`, `metadata.service.ts`, or `generate-metadata.js`.
2. Check whether builder or CI packages dashboard artifacts; load deployment skills if packaging behavior changes.
