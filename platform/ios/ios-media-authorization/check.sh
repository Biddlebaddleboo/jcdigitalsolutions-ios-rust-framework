#!/bin/sh
set -eu

cargo +1.94.1 test --locked -p ios-media-authorization
