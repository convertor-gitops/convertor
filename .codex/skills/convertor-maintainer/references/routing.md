# Routing

Use the narrowest skill that owns the changed surface.

## Single-Area Tasks

- Core conversion, parser, renderer, provider, config, encryption, or Rust API model: `convertor-dev-core`.
- Backend endpoint, request extractor, response body, service composition, backend status: `convertor-dev-core`.
- Local CLI/config/profile patch/install behavior: `convertor-dev-core`.
- Dashboard UI, Angular service, Angular model, URL form, status view, SCSS: `convertor-dev-dashboard`.
- GitHub Actions, Dockerfile, base image, compose, registry, release checklist: `convertor-deploy-cicd`.
- Builder CLI code, command ordering, build/image/publish/tag/version/dashboard subcommands: `convertor-deploy-builder`.

## Cross-Area Tasks

- Rust API contract plus dashboard UI: load `convertor-dev-core` first, then `convertor-dev-dashboard`.
- Dashboard-only bug caused by query serialization: load `convertor-dev-dashboard`, then `convertor-dev-core` for contract confirmation.
- CI command failure caused by builder output: load `convertor-deploy-cicd`, then `convertor-deploy-builder`.
- Builder change that affects workflow YAML or Docker arguments: load `convertor-deploy-builder`, then `convertor-deploy-cicd`.

## Boundaries

- Do not put builder maintenance under product Rust core.
- Do not put deployment pipeline assumptions in dashboard or core skills.
- Do not duplicate detailed peer-skill instructions in this maintainer skill.
