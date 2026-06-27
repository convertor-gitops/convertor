---
name: convertor-maintainer
description: Top-level maintenance router for the convertor repository. Use when Codex needs to orient on convertor project direction, plan feature work, choose between Rust core, Angular dashboard, CI/CD deployment, and crates/builder maintenance skills, or coordinate cross-area changes in /Users/bppleman/RustroverProjects/convertor.
---

# Convertor Maintainer

Use this skill first for broad or ambiguous convertor work. Keep it as the project map and router; load a deeper peer skill when implementation details matter.

## Start Here

1. Read `references/project-map.md` to identify the affected project area.
2. Read `references/routing.md` to choose the peer skill.
3. Load only the relevant peer skill and its references before editing.
4. For cross-area work, start with the source-of-truth side, then load the dependent skill.

## Peer Skills

- Use `.codex/skills/convertor-dev-core/SKILL.md` for `crates/convertor`, `crates/convd`, `crates/confly`, and `crates/fetcher`.
- Use `.codex/skills/convertor-dev-dashboard/SKILL.md` for `dashboard/`.
- Use `.codex/skills/convertor-deploy-cicd/SKILL.md` for GitHub Actions, Dockerfiles, compose, deployment, registry, and release-flow work.
- Use `.codex/skills/convertor-deploy-builder/SKILL.md` for `crates/builder` code and command semantics.

## Coordination Rules

- Treat `crates/builder` as deployment tooling, not product core.
- Treat Rust `ConvQuery`, `ConvUrl`, API response models, and dashboard TypeScript models as shared contracts when a feature crosses backend and frontend.
- Treat CI/CD workflow files as orchestration over builder/Docker/compose primitives; do not duplicate builder command semantics in CI documentation without checking the builder skill.
- Do not create summary docs such as README files for skill work unless explicitly requested.
