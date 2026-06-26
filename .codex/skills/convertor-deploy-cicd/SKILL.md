---
name: convertor-deploy-cicd
description: Maintain convertor CI/CD and deployment flows. Use for GitHub Actions workflows, Dockerfile/base.Dockerfile, compose deployment, registry/tag/release behavior, multi-arch image pipelines, deployment troubleshooting, and coordination with crates/builder command outputs.
---

# Convertor Deploy CI/CD

Use this skill for deployment orchestration and pipeline behavior. Use `convertor-deploy-builder` when changing `crates/builder` code or command semantics.

## Required Orientation

Read only the references needed for the task:

- `references/pipeline-map.md` for workflow and deployment file roles.
- `references/image-release.md` for Docker image, tag, registry, and multi-arch behavior.
- `references/deployment-boundaries.md` for what can be edited locally versus what needs user-run deployment.
- `references/validation.md` for local and CI-oriented checks.

## Working Rules

- Treat `.github/workflows/_build-shared.yml` as the shared release pipeline used by release/debug workflow entrypoints.
- Treat `crates/builder` as the executable fact source for build/release command semantics; do not rewrite those semantics directly in workflow logic or skill scripts without checking `convertor-deploy-builder`.
- Keep Docker build arguments aligned with `Dockerfile`, `base.Dockerfile`, and builder-generated image commands.
- Keep compose deployment assumptions separate from image build and release assumptions.
- Do not perform git commit, push, image push, registry mutation, or production deployment unless the user explicitly asks for that action.
