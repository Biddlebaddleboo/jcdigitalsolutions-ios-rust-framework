#!/bin/sh
set -eu

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

if [ "$(uname -s)" != Darwin ]; then
    printf '%s\n' 'requires macOS with Xcode' >&2
    exit 77
fi

for tool in cargo rustc xcrun nm otool awk grep sed mktemp; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-runtime-ownership.XXXXXX")
trap 'rm -rf "$probe_dir"' EXIT HUP INT TERM

case "$probe_dir/" in
    "$repo_root/"*) fail 'temporary oracle dir must stay outside the checkout' ;;
esac

swiftc_path=$(xcrun --find swiftc) || fail 'xcrun could not locate swiftc'
swift_target_info=$("$swiftc_path" -print-target-info) || fail 'swiftc could not report target information'
swift_runtime_dir=$(printf '%s\n' "$swift_target_info" | awk '
    /"runtimeLibraryPaths": \[/ { in_runtime_paths = 1; next }
    in_runtime_paths && /\]/ { exit }
    in_runtime_paths {
        sub(/^[[:space:]]*"/, "")
        sub(/",?[[:space:]]*$/, "")
        if ($0 == "/usr/lib/swift") { print; exit }
    }')
[ -n "$swift_runtime_dir" ] || fail 'active Swift toolchain does not report /usr/lib/swift as a host runtime path'
[ -d "$swift_runtime_dir" ] || fail "active Swift host runtime path does not exist: $swift_runtime_dir"
host_triple=$(rustc -vV | sed -n 's/^host: //p')
[ -n "$host_triple" ] || fail 'rustc did not report a host target triple'
device_sdk=$(xcrun --sdk iphoneos --show-sdk-path) || fail 'xcrun could not locate the iPhoneOS SDK'
simulator_sdk=$(xcrun --sdk iphonesimulator --show-sdk-path) || fail 'xcrun could not locate the iPhoneSimulator SDK'

cat > "$probe_dir/OwnershipOracle.swift" <<'SWIFT'
public final class OwnershipProbe {
    public static var deinitCount: Int32 = 0
    public init() {}
    deinit { Self.deinitCount += 1 }
}

@_cdecl("swift_ownership_create") public func createOwnershipProbe() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(OwnershipProbe()).toOpaque()
}

@_cdecl("swift_ownership_deinit_count") public func ownershipProbeDeinitCount() -> Int32 {
    OwnershipProbe.deinitCount
}

@_cdecl("swift_ownership_retain_oracle") public func retainOwnershipOracle(_ pointer: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged<OwnershipProbe>.fromOpaque(pointer).retain().toOpaque()
}

@_cdecl("swift_ownership_release_oracle") public func releaseOwnershipOracle(_ pointer: UnsafeMutableRawPointer) {
    Unmanaged<OwnershipProbe>.fromOpaque(pointer).release()
}
SWIFT

cat > "$probe_dir/Cargo.toml" <<EOF
[package]
name = "swift-ownership-probe"
version = "0.1.0"
edition = "2024"

[features]
apple-runtime = ["swift-abi-core/apple-runtime"]

[dependencies]
swift-abi-core = { path = "$repo_root/interop/swift-abi-core", default-features = false }
EOF

mkdir "$probe_dir/src"
cat > "$probe_dir/src/main.rs" <<'RUST'
#[cfg(feature = "apple-runtime")]
use core::ffi::c_void;
#[cfg(feature = "apple-runtime")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(feature = "apple-runtime")]
unsafe extern "C" {
    fn swift_ownership_create() -> *mut c_void;
    fn swift_ownership_deinit_count() -> i32;
}

#[cfg(feature = "apple-runtime")]
fn deinit_count() -> i32 {
    unsafe { swift_ownership_deinit_count() }
}

#[cfg(feature = "apple-runtime")]
fn main() {
    assert!(unsafe { SwiftRetained::<SwiftObject>::from_owned_ptr(core::ptr::null_mut()) }.is_none());
    assert!(unsafe { SwiftRetained::<SwiftObject>::retain_borrowed(core::ptr::null_mut()) }.is_none());
    for expected in 1..=64 {
        let raw = unsafe { swift_ownership_create() }.cast::<SwiftObject>();
        assert_eq!(deinit_count(), expected - 1);
        let owned = unsafe { SwiftRetained::from_owned_ptr(raw) }.unwrap();
        let borrowed = unsafe { SwiftRetained::retain_borrowed(owned.as_ptr()) }.unwrap();
        let cloned = owned.clone();
        assert_eq!(owned.as_ptr(), borrowed.as_ptr());
        assert_eq!(owned.as_ptr(), cloned.as_ptr());
        assert_eq!(deinit_count(), expected - 1);
        drop(owned);
        assert_eq!(deinit_count(), expected - 1);
        let transferred = cloned.into_raw();
        let adopted = unsafe { SwiftRetained::from_owned_ptr(transferred) }.unwrap();
        drop(borrowed);
        assert_eq!(deinit_count(), expected - 1);
        drop(adopted);
        assert_eq!(deinit_count(), expected);
    }
}

#[cfg(not(feature = "apple-runtime"))]
fn main() {}
RUST

check_runtime_ir() {
    ir_file=$1
    target_name=$2
    grep -Eq '^[[:space:]]*declare ptr @swift_retain\(ptr returned\)( |$)' "$ir_file" || fail "$target_name Swift IR lacks compiler-derived swift_retain(ptr returned) -> ptr declaration"
    grep -Eq '^[[:space:]]*declare void @swift_release\(ptr\)( |$)' "$ir_file" || fail "$target_name Swift IR lacks compiler-derived swift_release(ptr) -> void declaration"
    grep -F 'call ptr @swift_retain(ptr returned' "$ir_file" >/dev/null || fail "$target_name Swift IR lacks a swift_retain call"
    grep -F 'call void @swift_release(ptr ' "$ir_file" >/dev/null || fail "$target_name Swift IR lacks a swift_release call"
}

xcrun swiftc -swift-version 5 -parse-as-library -emit-ir -Onone -module-name SwiftAbiOwnershipOracle -o "$probe_dir/oracle-host.ll" "$probe_dir/OwnershipOracle.swift"
xcrun swiftc -swift-version 5 -parse-as-library -emit-object -Onone -module-name SwiftAbiOwnershipOracle -o "$probe_dir/oracle-host.o" "$probe_dir/OwnershipOracle.swift"
check_runtime_ir "$probe_dir/oracle-host.ll" host

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=$device_sdk
        triple=arm64-apple-ios15.0
    else
        sdk=$simulator_sdk
        triple=arm64-apple-ios15.0-simulator
    fi
    xcrun swiftc -swift-version 5 -parse-as-library -emit-ir -Onone -module-name SwiftAbiOwnershipOracle -target "$triple" -sdk "$sdk" -o "$probe_dir/oracle-$target.ll" "$probe_dir/OwnershipOracle.swift"
    xcrun swiftc -swift-version 5 -parse-as-library -emit-object -Onone -module-name SwiftAbiOwnershipOracle -target "$triple" -sdk "$sdk" -o "$probe_dir/oracle-$target.o" "$probe_dir/OwnershipOracle.swift"
    check_runtime_ir "$probe_dir/oracle-$target.ll" "$target"
done

(
    cd "$probe_dir"
    cargo build --offline --target "$host_triple"
)
default_binary="$probe_dir/target/$host_triple/debug/swift-ownership-probe"
otool -L "$default_binary" > "$probe_dir/default-linkage.txt"
if grep -F 'libswiftCore' "$probe_dir/default-linkage.txt" >/dev/null; then
    fail 'default swift-abi-core feature path linked libswiftCore'
fi
if nm -u "$default_binary" | grep -E '_swift_(retain|release)$' >/dev/null; then
    fail 'default swift-abi-core feature path imports swift_retain or swift_release'
fi

cat > "$probe_dir/.cargo-config" <<EOF
[target."$host_triple"]
rustflags = ["-C", "link-arg=$probe_dir/oracle-host.o", "-C", "link-arg=-Wl,-rpath,$swift_runtime_dir"]
EOF
mkdir "$probe_dir/.cargo"
mv "$probe_dir/.cargo-config" "$probe_dir/.cargo/config.toml"
(
    cd "$probe_dir"
    cargo build --offline --target "$host_triple" --features apple-runtime
)
runtime_binary="$probe_dir/target/$host_triple/debug/swift-ownership-probe"
nm -u "$runtime_binary" > "$probe_dir/runtime-imports.txt"
grep -E '_swift_retain$' "$probe_dir/runtime-imports.txt" >/dev/null || fail 'runtime ownership probe does not import _swift_retain'
grep -E '_swift_release$' "$probe_dir/runtime-imports.txt" >/dev/null || fail 'runtime ownership probe does not import _swift_release'
otool -L "$runtime_binary" > "$probe_dir/runtime-linkage.txt"
grep -F '@rpath/libswiftCore.dylib' "$probe_dir/runtime-linkage.txt" >/dev/null || fail 'runtime ownership probe does not link @rpath/libswiftCore.dylib'
otool -l "$runtime_binary" > "$probe_dir/runtime-load-commands.txt"
grep -F "$swift_runtime_dir" "$probe_dir/runtime-load-commands.txt" >/dev/null || fail 'runtime ownership probe lacks the active Swift host runtime LC_RPATH'
DYLD_PRINT_LIBRARIES=1 "$runtime_binary" > "$probe_dir/runtime-stdout.txt" 2> "$probe_dir/runtime-libraries.txt"
swift_core_load_count=$(grep -c 'libswiftCore\.dylib' "$probe_dir/runtime-libraries.txt" || true)
[ "$swift_core_load_count" -eq 1 ] || fail "runtime ownership probe loaded $swift_core_load_count libswiftCore.dylib copies; expected exactly one"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --offline -p swift-abi-core --features apple-runtime --target "$target"
    cargo check --locked --offline -p swift-abi-generated --features apple-runtime --target "$target"
done

printf '%s\n' 'host Rust -> Swift class ownership oracle: passed (64 exact deinit cycles; from_owned_ptr, retain_borrowed, Clone, Drop, into_raw/re-adoption)'
printf '%s\n' 'host compiler oracle: passed (swift_retain(ptr returned) -> ptr; swift_release(ptr) -> void)'
printf '%s\n' 'device compiler oracle: passed (compile-only, arm64-apple-ios15.0)'
printf '%s\n' 'simulator compiler oracle: passed (compile-only, arm64-apple-ios15.0-simulator)'
printf '%s\n' 'default swift-abi-core feature path: passed (no swiftCore linkage or swift_retain/swift_release imports)'
printf '%s\n' 'runtime imports:'
grep -E '_swift_(retain|release)$' "$probe_dir/runtime-imports.txt"
printf '%s\n' 'runtime dependency:'
grep -F '@rpath/libswiftCore.dylib' "$probe_dir/runtime-linkage.txt"
printf '%s\n' 'loaded runtime:'
grep -F 'libswiftCore.dylib' "$probe_dir/runtime-libraries.txt"
printf '%s\n' "runtime LC_RPATH: $swift_runtime_dir"
