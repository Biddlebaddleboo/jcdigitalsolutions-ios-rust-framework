#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

manifest=platform/ios/ios-file-provider/Cargo.toml
source_files="platform/ios/ios-file-provider/src/lib.rs platform/ios/ios-file-provider/src/completion.rs platform/ios/ios-file-provider/src/platform.rs platform/ios/ios-file-provider/examples/link_probe.rs"

if ! rg -Fq 'objc2-file-provider = { version = "=0.3.2", default-features = false, features = ["Extension", "NSFileProviderDomain", "block2"] }' "$manifest"; then
    echo "unexpected objc2-file-provider feature set" >&2
    exit 1
fi
if ! rg -Fq 'objc2-foundation = { workspace = true, features = ["NSArray", "NSError", "NSString"] }' "$manifest"; then
    echo "unexpected objc2-foundation feature set" >&2
    exit 1
fi
if ! rg -Fq 'NSFileProviderManager::getDomainsWithCompletionHandler(&handler)' platform/ios/ios-file-provider/src/platform.rs; then
    echo "no registered-domain query call found" >&2
    exit 1
fi
if ! rg -Fq 'objc2::available!(ios = 11.0, ..)' platform/ios/ios-file-provider/src/platform.rs; then
    echo "unexpected FileProvider API availability floor" >&2
    exit 1
fi
if rg -n 'UIDocumentPicker|NSFileCoordinator|startAccessingSecurityScopedResource|addDomain:|removeDomain:|NSFileProviderExtension|NSFileProviderReplicatedExtension|NSFileProviderEnumerating|NSFileProviderItem|NSFileProviderSync|NSFileProviderSearch|FileProviderUI' $source_files; then
    echo "out-of-scope document, domain-mutation, extension-lifecycle, or file API found" >&2
    exit 1
fi
if find platform/ios/ios-file-provider -type f -name '*.swift' -print | grep -q .; then
    echo "Swift source is not allowed in ios-file-provider" >&2
    exit 1
fi

sh -n platform/ios/ios-file-provider/check.sh platform/ios/ios-file-provider/check-link-imports.sh
cargo fmt --manifest-path "$manifest" -- --check
cargo check --locked -p ios-file-provider
cargo check --locked -p ios-file-provider --target aarch64-apple-ios
cargo check --locked -p ios-file-provider --target aarch64-apple-ios-sim
cargo clippy --locked --lib -p ios-file-provider --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --lib -p ios-file-provider --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked --no-deps -p ios-file-provider --target aarch64-apple-ios
sh platform/ios/ios-file-provider/check-link-imports.sh
cargo xtask docs-check
