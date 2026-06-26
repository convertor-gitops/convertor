# CI/CD Validation

## Static Checks

```bash
cargo check -p builder
cargo test -p builder
```

## Workflow-Aware Checks

Use builder help or non-mutating commands to confirm CLI shape:

```bash
cargo run -p builder -- version
cargo run -p builder -- tag base release --registry harbor.internal.bppleman.cn:30443 --user BppleMan
```

Prefer the builder skill scripts when validating command contracts:

```bash
bash .codex/skills/convertor-deploy-builder/scripts/inspect-builder-help.sh
bash .codex/skills/convertor-deploy-builder/scripts/validate-builder-tags.sh
```

## Docker Checks

Prefer local parse/build checks that do not push:

```bash
docker buildx build -f base.Dockerfile --load .
```

Only run image push or manifest creation after explicit user request.
