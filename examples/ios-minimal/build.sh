#!/bin/sh
set -eu

example_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$example_dir/../.." && pwd)
kind=${1:-simulator}
features=

case "$kind" in
    simulator)
        target=aarch64-apple-ios-sim
        sdk=iphonesimulator
        ;;
    simulator-metal)
        target=aarch64-apple-ios-sim
        sdk=iphonesimulator
        features=custom-render-metal-demo
        ;;
    simulator-x86)
        target=x86_64-apple-ios
        sdk=iphonesimulator
        ;;
    simulator-x86-metal)
        target=x86_64-apple-ios
        sdk=iphonesimulator
        features=custom-render-metal-demo
        ;;
    device)
        target=aarch64-apple-ios
        sdk=iphoneos
        ;;
    device-metal)
        target=aarch64-apple-ios
        sdk=iphoneos
        features=custom-render-metal-demo
        ;;
    *)
        echo "usage: $0 [simulator|simulator-metal|simulator-x86|simulator-x86-metal|device|device-metal]" >&2
        exit 2
        ;;
esac

export SDKROOT="$(xcrun --sdk "$sdk" --show-sdk-path)"
export IPHONEOS_DEPLOYMENT_TARGET=17.0
if [ -n "$features" ]; then
    cargo build --manifest-path "$example_dir/Cargo.toml" --target "$target" --release --features "$features"
else
    cargo build --manifest-path "$example_dir/Cargo.toml" --target "$target" --release
fi

app_dir="$repo_dir/target/ios-minimal/$kind/ios-minimal.app"
rm -rf "$app_dir"
mkdir -p "$app_dir"
cp "$repo_dir/target/$target/release/ios-minimal" "$app_dir/ios-minimal"
cp "$example_dir/Info.plist" "$app_dir/Info.plist"
echo "Built $app_dir"
