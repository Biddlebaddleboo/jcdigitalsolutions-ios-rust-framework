#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

exec ios-rust-validate --workspace-root "$repo_root" --spec "$repo_root/tools/validation/specs/validation-v1.json" --capability ios-homekit-identify-status
