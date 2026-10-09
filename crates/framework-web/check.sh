#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$repo_root"
cargo check --locked --no-default-features -p framework-web
cargo test --locked -p framework-web
cargo clippy --locked --all-targets -p framework-web -- -D warnings
