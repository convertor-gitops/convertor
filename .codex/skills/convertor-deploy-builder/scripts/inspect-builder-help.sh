#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"

cd "${repo_root}"

commands=(
  "--help"
  "build --help"
  "image --help"
  "publish --help"
  "tag --help"
  "dashboard --help"
  "version --help"
)

for command_args in "${commands[@]}"; do
  printf '\n### builder %s\n' "${command_args}"
  # shellcheck disable=SC2086
  cargo run -p builder -- ${command_args}
done
