#!/bin/sh
set -eu

example_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$example_dir/../.." && pwd)
kind=${1:-simulator}

case "$kind" in
    simulator)
        target=aarch64-apple-ios-sim
        sdk=iphonesimulator
        ;;
    simulator-x86)
        target=x86_64-apple-ios
        sdk=iphonesimulator
        ;;
    device)
        target=aarch64-apple-ios
        sdk=iphoneos
        ;;
    *)
        echo "usage: $0 [simulator|simulator-x86|device]" >&2
        exit 2
        ;;
esac

export SDKROOT="$(xcrun --sdk "$sdk" --show-sdk-path)"
export IPHONEOS_DEPLOYMENT_TARGET=17.0
cargo build --manifest-path "$example_dir/Cargo.toml" --target "$target" --release

app_dir="$repo_dir/target/ios-minimal/$kind/ios-minimal.app"
rm -rf "$app_dir"
mkdir -p "$app_dir"
cp "$repo_dir/target/$target/release/ios-minimal" "$app_dir/ios-minimal"
cp "$example_dir/Info.plist" "$app_dir/Info.plist"
echo "Built $app_dir"
