---
name: convertor-dev-dashboard
description: Maintain the convertor Angular dashboard. Use for dashboard feature development, UI state, services, Angular models, backend API integration, URL/query building, component styling, tests, and synchronization with Rust ConvQuery, ConvUrl, UrlResult, status, and subscription contracts.
---

# Convertor Dev Dashboard

Use this skill for `dashboard/` work. If the frontend change depends on Rust API or query semantics, load `convertor-dev-core` as the source-of-truth companion.

## Required Orientation

Read only the references needed for the task:

- `references/app-map.md` for dashboard structure.
- `references/feature-workflows.md` for common UI and integration changes.
- `references/backend-contracts.md` for Rust/Angular contract sync points.
- `references/testing.md` for build and test commands.

## Working Rules

- Prefer existing Angular service/model/component patterns over adding a new state layer.
- Keep URL/query serialization aligned with Rust `ConvQuery` and backend `serde_qs` behavior.
- Keep API response models under `dashboard/src/app/common/model/api` aligned with `crates/convd/src/server/model`.
- Keep domain models under `dashboard/src/app/common/model/core` aligned with `crates/convertor/src`.
- For visual changes, inspect existing SCSS and component layout before introducing new UI idioms.
