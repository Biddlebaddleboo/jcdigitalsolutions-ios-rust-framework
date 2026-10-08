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

for tool in xcrun rustc nm awk grep mktemp; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-rust-thunk.XXXXXX")
trap 'rm -rf "$probe_dir"' EXIT HUP INT TERM

case "$probe_dir/" in
    "$repo_root/"*) fail 'temporary oracle dir must stay outside the checkout' ;;
esac

cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public func add(_ lhs: Int32, _ rhs: Int32) -> Int32 { lhs &+ rhs }
SWIFT

cat > "$probe_dir/caller.rs" <<'RUST'
unsafe extern "C" {
    fn framework_swift_abi_add(lhs: i32, rhs: i32) -> i32;
}

fn main() {
    let actual = unsafe { framework_swift_abi_add(19, 23) };
    assert_eq!(actual, 42);
}
RUST

cat > "$probe_dir/cross-caller.rs" <<'RUST'
#![no_std]
unsafe extern "C" {
    fn framework_swift_abi_add(lhs: i32, rhs: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn rust_swiftcall_probe() -> i32 {
    unsafe { framework_swift_abi_add(19, 23) }
}
RUST

xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProbe -o "$probe_dir/oracle-host.ll" "$probe_dir/Oracle.swift"
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProbe -o "$probe_dir/oracle-host.o" "$probe_dir/Oracle.swift"

swift_symbol=$(nm -g "$probe_dir/oracle-host.o" | awk '$2 == "T" { count++; symbol = $3 } END { if (count != 1) exit 1; print symbol }') || fail 'expected one Swift fixture function symbol'
case "$swift_symbol" in
    _\$s*) ;;
    *) fail "unexpected Swift symbol from nm: $swift_symbol" ;;
esac

ir_symbol=${swift_symbol#_}
oracle_signature=$(grep -F "define swiftcc i32 @\"$ir_symbol\"(" "$probe_dir/oracle-host.ll" || true)
[ -n "$oracle_signature" ] || fail 'Swift compiler IR lacks the expected swiftcc function declaration'
printf '%s\n' "$oracle_signature" | grep -Eq '\(i32 [^,]+, i32 [^)]+\)' || fail 'Swift compiler IR lacks the expected direct i32 argument pair'

cat > "$probe_dir/thunk.c" <<C
#include <stdint.h>
extern int32_t swift_abi_add(int32_t lhs, int32_t rhs) __asm__("$swift_symbol") __attribute__((swiftcall));
__attribute__((visibility("default"))) int32_t framework_swift_abi_add(int32_t lhs, int32_t rhs) {
    return swift_abi_add(lhs, rhs);
}
C

cat > "$probe_dir/target-caller.rs" <<'RUST'
#![no_std]
unsafe extern "C" {
    fn framework_swift_abi_add(lhs: i32, rhs: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn rust_swiftcall_probe() -> i32 {
    unsafe { framework_swift_abi_add(19, 23) }
}
RUST

xcrun clang -S -emit-llvm -O2 -o "$probe_dir/thunk-host.ll" "$probe_dir/thunk.c"
xcrun clang -S -O2 -o "$probe_dir/thunk-host.s" "$probe_dir/thunk.c"
xcrun clang -c -O2 -o "$probe_dir/thunk-host.o" "$probe_dir/thunk.c"
grep -E 'define i32 @framework_swift_abi_add\(i32 [^,]+, i32 [^)]+\)' "$probe_dir/thunk-host.ll" >/dev/null || fail 'Clang IR lacks the two-argument C-ABI wrapper'
swiftcall_line=$(grep -F 'call swiftcc i32' "$probe_dir/thunk-host.ll" | grep -F "$swift_symbol" || true)
[ -n "$swiftcall_line" ] || fail 'Clang IR lacks the compiler-derived swiftcall symbol'
printf '%s\n' "$swiftcall_line" | grep -Eq 'call swiftcc i32 .*\(i32 [^,]+, i32 [^)]+\)' || fail 'Clang IR call does not match the compiler-derived i32 signature'

rustc --edition=2024 --crate-name swiftcall_scalar_probe "$probe_dir/caller.rs" \
    -C "link-arg=$probe_dir/thunk-host.o" \
    -C "link-arg=$probe_dir/oracle-host.o" \
    -o "$probe_dir/rust-host-probe"
"$probe_dir/rust-host-probe"

device_sdk=$(xcrun --sdk iphoneos --show-sdk-path)
simulator_sdk=$(xcrun --sdk iphonesimulator --show-sdk-path)

xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProbe -target arm64-apple-ios17.0 -sdk "$device_sdk" -o "$probe_dir/oracle-device.ll" "$probe_dir/Oracle.swift"
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProbe -target arm64-apple-ios17.0 -sdk "$device_sdk" -o "$probe_dir/oracle-device.o" "$probe_dir/Oracle.swift"
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot "$device_sdk" -S -emit-llvm -O2 -o "$probe_dir/thunk-device.ll" "$probe_dir/thunk.c"
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot "$device_sdk" -S -O2 -o "$probe_dir/thunk-device.s" "$probe_dir/thunk.c"
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot "$device_sdk" -c -O2 -o "$probe_dir/thunk-device.o" "$probe_dir/thunk.c"
rustc --edition=2021 --crate-type lib --target aarch64-apple-ios --emit=obj "$probe_dir/target-caller.rs" -o "$probe_dir/rust-device.o"

xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProbe -target arm64-apple-ios17.0-simulator -sdk "$simulator_sdk" -o "$probe_dir/oracle-simulator.ll" "$probe_dir/Oracle.swift"
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProbe -target arm64-apple-ios17.0-simulator -sdk "$simulator_sdk" -o "$probe_dir/oracle-simulator.o" "$probe_dir/Oracle.swift"
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot "$simulator_sdk" -S -emit-llvm -O2 -o "$probe_dir/thunk-simulator.ll" "$probe_dir/thunk.c"
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot "$simulator_sdk" -S -O2 -o "$probe_dir/thunk-simulator.s" "$probe_dir/thunk.c"
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot "$simulator_sdk" -c -O2 -o "$probe_dir/thunk-simulator.o" "$probe_dir/thunk.c"
rustc --edition=2021 --crate-type lib --target aarch64-apple-ios-sim --emit=obj "$probe_dir/target-caller.rs" -o "$probe_dir/rust-simulator.o"

for target in device simulator; do
    oracle_object="$probe_dir/oracle-$target.o"
    oracle_ir="$probe_dir/oracle-$target.ll"
    thunk_object="$probe_dir/thunk-$target.o"
    thunk_ir="$probe_dir/thunk-$target.ll"
    thunk_assembly="$probe_dir/thunk-$target.s"
    rust_object="$probe_dir/rust-$target.o"

    target_symbol=$(nm -g "$oracle_object" | awk '$2 == "T" { count++; symbol = $3 } END { if (count != 1) exit 1; print symbol }') || fail "$target Swift object lacks one fixture symbol"
    [ "$target_symbol" = "$swift_symbol" ] || fail "$target Swift symbol differs from host oracle"
    target_signature=$(grep -F "define swiftcc i32 @\"$ir_symbol\"(" "$oracle_ir" || true)
    [ -n "$target_signature" ] || fail "$target Swift IR lacks the host swiftcc signature"
    printf '%s\n' "$target_signature" | grep -Eq '\(i32 [^,]+, i32 [^)]+\)' || fail "$target Swift IR lacks the direct i32 argument pair"
    swiftcall_line=$(grep -F 'call swiftcc i32' "$thunk_ir" | grep -F "$swift_symbol" || true)
    [ -n "$swiftcall_line" ] || fail "$target Clang IR lacks the compiler-derived swiftcall symbol"
    printf '%s\n' "$swiftcall_line" | grep -Eq 'call swiftcc i32 .*\(i32 [^,]+, i32 [^)]+\)' || fail "$target Clang IR call does not match the compiler-derived i32 signature"
    nm -g "$thunk_object" | awk -v symbol="$swift_symbol" '$1 == "U" && $2 == symbol { found = 1 } END { exit !found }' || fail "$target Clang object lacks the Swift symbol reference"
    grep -F "$swift_symbol" "$thunk_assembly" >/dev/null || fail "$target optimized assembly lacks the Swift call target"
    nm -u "$rust_object" | grep -F '_framework_swift_abi_add' >/dev/null || fail "$target Rust object lacks the C-ABI wrapper reference"
done

printf 'host Rust -> C swiftcall -> Swift fixture: passed (symbol %s)\n' "$swift_symbol"
printf '%s\n' 'device Swift, Clang, and Rust objects: passed (compile-only, iOS 17.0 target)'
printf '%s\n' 'simulator Swift, Clang, and Rust objects: passed (compile-only, iOS 17.0 target)'
