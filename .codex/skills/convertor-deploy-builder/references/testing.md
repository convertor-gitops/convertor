# Builder Validation

## Compile And Test

```bash
cargo check -p builder
cargo test -p builder
```

## CLI Shape

```bash
cargo run -p builder -- --help
cargo run -p builder -- version
cargo run -p builder -- tag base release --registry harbor.internal.bppleman.cn:30443 --user BppleMan
```

## Skill Script Checks

```bash
bash .codex/skills/convertor-deploy-builder/scripts/inspect-builder-help.sh
bash .codex/skills/convertor-deploy-builder/scripts/validate-builder-tags.sh
bash .codex/skills/convertor-deploy-builder/scripts/show-builder-plan.sh
```

The plan script is currently a capability probe. Until builder supports `--plan`, `--dry-run`, or structured output, do not treat it as a release-plan generator.

## Mutating Commands

Avoid running these unless explicitly requested:

```bash
cargo run -p builder -- image base release --arch amd,arm --registry harbor.internal.bppleman.cn:30443 --user BppleMan
cargo run -p builder -- image convd release --arch amd,arm --dashboard --registry harbor.internal.bppleman.cn:30443 --user BppleMan
cargo run -p builder -- publish convd release
```

They may build images, push to registries, create manifests, or publish crates.
