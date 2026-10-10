#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

exec cargo +1.94.1 xtask validate --capability ios-alarmkit-status
