# Deployment Boundaries

## Safe Local Work

- Edit workflow YAML, Dockerfiles, compose files, and justfile entries.
- Run lint/compile/build checks that do not push images or mutate registries.
- Generate commands for the user to run.
- Inspect local Dockerfile/build command consistency.

## Mutating Work

These actions require explicit user request for the specific operation:

- pushing images
- creating or replacing remote manifests
- logging into registries with secrets
- production deployment
- git commit or push

## Coordination

- For builder command changes, use `convertor-deploy-builder`.
- For dashboard packaging changes, use `convertor-dev-dashboard` and `convertor-deploy-builder`.
- For runtime behavior changes in convd, use `convertor-dev-core`.
