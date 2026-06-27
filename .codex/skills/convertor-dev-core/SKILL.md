---
name: convertor-dev-core
description: Maintain convertor Rust product code. Use for feature development, refactors, tests, and reviews involving crates/convertor core conversion logic, crates/convd Axum backend, crates/confly CLI/config/install behavior, crates/fetcher HTTP fetching, Rust URL/profile/config contracts, and backend-dashboard API contract changes.
---

# Convertor Dev Core

Use this skill for product-side Rust development. Do not use it for `crates/builder`; use `convertor-deploy-builder` for that crate.

## Required Orientation

Read only the references needed for the task:

- `references/crate-map.md` for crate responsibilities and source-of-truth boundaries.
- `references/feature-workflows.md` for common Rust feature paths.
- `references/contracts.md` for cross-crate and dashboard contracts.
- `references/testing.md` for validation commands and snapshot handling.

## Working Rules

- Start from the crate that owns the domain concept, then update dependents.
- Keep parser/renderer/profile changes in `crates/convertor`; expose them through `convd` only after the core behavior is clear.
- Keep HTTP routing, request extraction, response shape, and service composition in `crates/convd`.
- Keep local CLI/config/profile patch behavior in `crates/confly`.
- Keep low-level request/response transport behavior in `crates/fetcher`.
- When a Rust contract changes dashboard behavior, load `convertor-dev-dashboard` before editing frontend code.
- Preserve snapshot tests as review artifacts; do not accept snapshot changes blindly.
