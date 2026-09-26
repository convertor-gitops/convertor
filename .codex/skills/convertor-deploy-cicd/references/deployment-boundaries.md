# Deployment Boundaries

## Safe Local Work

- Edit workflow YAML, Dockerfiles, compose files, and justfile entries.
- Run lint/compile/build checks that do not push images or mutate registries.
- Generate commands for the user to run.
- Inspect local Dockerfile/build command consistency.

## Mutating Work

A request to deploy a feature to `debug` or `main` authorizes the branch commit, feature push, non-fast-forward merge, target push, and (for `main`) version commit and tag push described in `feature-deployment.md`. A read-only explanation or review does not authorize them.

Other mutations require an explicit request for the operation:

- pushing images
- creating or replacing remote manifests
- logging into registries with secrets

## Coordination

- For builder command changes, use `convertor-deploy-builder`.
- For dashboard packaging changes, use `convertor-dev-dashboard` and `convertor-deploy-builder`.
- For runtime behavior changes in convd, use `convertor-dev-core`.
