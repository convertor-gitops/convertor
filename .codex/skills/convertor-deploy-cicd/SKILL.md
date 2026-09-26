---
name: convertor-deploy-cicd
description: Deploy convertor feature branches to debug or main through merge commits, and maintain the related GitHub Actions, image build, and release workflows.
---

# Convertor Deploy CI/CD

Use this skill when deploying a feature to `debug` or `main`, or maintaining the associated CI/CD pipeline. Here, deployment means merging a feature branch into the target branch and pushing the target; the workflow builds the image afterward. Use `convertor-deploy-builder` when changing `crates/builder` code or command semantics.

## References

Read only those needed for the task:

- `references/feature-deployment.md` for the standard feature-to-debug or feature-to-main deployment procedure.
- `references/pipeline-map.md` for workflow and deployment file roles.
- `references/image-release.md` for Docker image, tag, registry, and multi-arch behavior.
- `references/deployment-boundaries.md` for the scope of a deployment request and other mutations.
- `references/validation.md` for local and CI-oriented checks.

## Deployment Rules

- Start on a `feature/*` or `codex/*` branch. If the current branch matches neither, remind the user before proceeding with deployment.
- Supported targets are `debug` and `main`; `main` is production (`prod`). If the user does not name the target, resolve it from the request or ask.
- If the target already contains every feature commit and there are no pending feature changes, end with “无新增内容可部署”. Otherwise commit intended changes, validate, and push the feature branch before merging. Every deployment merge must use `--no-ff` and produce a new merge commit.
- A `debug` deployment finishes after pushing `debug` and returning to the starting branch. GitHub Actions runs asynchronously; do not wait for it as part of this procedure.
- Before a `main` deployment with new feature content, find the latest version tag in `main` history and ensure the feature's `metadata.json` version is newer. If needed, choose a version increment based on the feature, run `just version-sync`, commit, and push the feature before merging. After pushing `main`, create and push the matching version tag from the new merge commit, then return to the starting branch. The tag triggers the release workflow.

## Pipeline Rules

- Treat `.github/workflows/_build-shared.yml` as the shared release pipeline used by release/debug workflow entrypoints.
- Treat `crates/builder` as the executable fact source for build/release command semantics; do not rewrite those semantics directly in workflow logic or skill scripts without checking `convertor-deploy-builder`.
- Keep Docker build arguments aligned with `Dockerfile`, `base.Dockerfile`, and builder-generated image commands.
- Keep compose deployment assumptions separate from image build and release assumptions.
- Do not perform deployment or other remote mutations in response to a read-only request. A request to deploy to a named target authorizes the corresponding sequence in `feature-deployment.md`.
