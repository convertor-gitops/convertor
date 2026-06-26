#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"

cd "${repo_root}"

if cargo run -p builder -- image --help | grep -Eq -- '(^|[[:space:]])--(plan|dry-run|format)([[:space:],]|$)'; then
  cat <<'MSG'
builder appears to expose plan/dry-run/format support. Inspect the current help
output and run the supported non-mutating plan command explicitly.
MSG
  cargo run -p builder -- image --help
else
  cat <<'MSG'
builder does not currently expose --plan, --dry-run, or --format for image
release planning. Use inspect-builder-help.sh and validate-builder-tags.sh for
safe contract checks, then inspect crates/builder command construction before
approving any mutating image/publish operation.
MSG
fi
