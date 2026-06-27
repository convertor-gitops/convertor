---
name: convertor-deploy-builder
description: Maintain convertor crates/builder release tooling. Use for Rust development in crates/builder, builder CLI command modeling, build/image/publish/tag/version/dashboard subcommands, generated shell command ordering, Docker build argument contracts, CI workflow integration, and release automation tooling tests.
---

# Convertor Deploy Builder

Use this skill for `crates/builder`. This crate is Rust code, but its domain is deployment automation, so keep it separate from product core development.

## Required Orientation

Read only the references needed for the task:

- `references/crate-map.md` for builder modules and responsibilities.
- `references/executable-fact-source.md` for treating builder as the release/build fact source.
- `references/command-workflows.md` for subcommand behavior and command ordering.
- `references/ci-contracts.md` for workflow, Dockerfile, and justfile contracts.
- `references/safe-operations.md` for safe versus mutating builder actions.
- `references/testing.md` for local validation.

## Working Rules

- Preserve the command-generator model: subcommands create `std::process::Command` values and `main.rs` executes them in order.
- Treat builder as the single executable fact source for convertor build/release command semantics; do not duplicate its Docker/cargo/manifest logic in skill scripts.
- Keep user-facing builder output clear because CI logs and local release runs depend on it.
- Keep image command arguments aligned with Dockerfiles and CI workflow expectations.
- Treat publish/image/push operations as potentially mutating; prefer compile/help/dry-run style validation unless the user explicitly asks for a live operation.
- When changing pipeline behavior outside `crates/builder`, load `convertor-deploy-cicd` too.

## Scripts

- Use `scripts/inspect-builder-help.sh` to snapshot the current builder command tree.
- Use `scripts/validate-builder-tags.sh` to validate non-mutating tag generation.
- Use `scripts/show-builder-plan.sh` only as a plan-capability probe until builder gains `--plan` or structured output.
