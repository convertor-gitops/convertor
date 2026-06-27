#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"

cd "${repo_root}"

registry="${BUILDER_REGISTRY:-ghcr}"
user="${BUILDER_USER:-convertor-gitops}"
project="${BUILDER_PROJECT:-convertor}"
profile="${BUILDER_PROFILE:-release}"

for image in base convd; do
  printf '\n### builder tag %s %s\n' "${image}" "${profile}"
  cargo run -p builder -- tag "${image}" "${profile}" \
    --registry "${registry}" \
    --user "${user}" \
    --project "${project}"
done
