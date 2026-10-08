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

for tool in xcrun xcode-select xcodebuild sw_vers nm awk grep mktemp otool; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-async-thunk.XXXXXX")
cleanup_probe() {
    if [ "${KEEP_SWIFT_ASYNC_THUNK_PROBE:-0}" = 1 ]; then
        printf 'preserved temporary probe: %s\n' "$probe_dir" >&2
    else
        rm -rf "$probe_dir"
    fi
}
trap cleanup_probe EXIT HUP INT TERM

case "$probe_dir/" in
    "$repo_root/"*) fail 'temporary oracle dir must stay outside the checkout' ;;
esac

host_target_info=$(xcrun swiftc -print-target-info)
host_target=$(printf '%s\n' "$host_target_info" | awk -F '"' '/"triple"/ { print $4; exit }')
case "$host_target" in
    *-apple-macosx*) ;;
    *) fail "unsupported Swift host target: $host_target" ;;
esac

host_sdk=$(xcrun --sdk macosx --show-sdk-path)
device_sdk=$(xcrun --sdk iphoneos --show-sdk-path)
simulator_sdk=$(xcrun --sdk iphonesimulator --show-sdk-path)
host_sdk_version=$(xcrun --sdk macosx --show-sdk-version)
device_sdk_version=$(xcrun --sdk iphoneos --show-sdk-version)
simulator_sdk_version=$(xcrun --sdk iphonesimulator --show-sdk-version)
device_target=arm64-apple-ios17.0
simulator_target=arm64-apple-ios17.0-simulator

mkdir "$probe_dir/host" "$probe_dir/device" "$probe_dir/simulator"

cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public enum OracleFailure: Error { case failed }
public func asyncIdentity(_ value: Int32) async -> Int32 { value }
public func asyncThrowsIdentity(_ value: Int32) async throws -> Int32 { if value < 0 { throw OracleFailure.failed }; return value }
public func asyncCaller(_ value: Int32) async -> Int32 { await asyncIdentity(value) }
public func asyncThrowsCaller(_ value: Int32) async -> Int32 { do { return try await asyncThrowsIdentity(value) } catch { return -1 } }
SWIFT

compile_swift_target() {
    target_name=$1
    target_triple=$2
    target_sdk=$3
    target_dir="$probe_dir/$target_name"
    xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target "$target_triple" -sdk "$target_sdk" -Onone -emit-ir -o "$target_dir/oracle.ll" "$probe_dir/Oracle.swift"
    xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target "$target_triple" -sdk "$target_sdk" -Onone -emit-assembly -o "$target_dir/oracle.s" "$probe_dir/Oracle.swift"
    xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target "$target_triple" -sdk "$target_sdk" -Onone -emit-object -o "$target_dir/oracle.o" "$probe_dir/Oracle.swift"
}

xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -emit-silgen -o "$probe_dir/host/oracle.sil" "$probe_dir/Oracle.swift"
compile_swift_target host "$host_target" "$host_sdk"
compile_swift_target device "$device_target" "$device_sdk"
compile_swift_target simulator "$simulator_target" "$simulator_sdk"
xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target "$host_target" -sdk "$host_sdk" -Onone -emit-library -o "$probe_dir/host/libSwiftAsyncThunkProof.dylib" "$probe_dir/Oracle.swift"

grep -F '$@convention(thin) @async (Int32) -> Int32' "$probe_dir/host/oracle.sil" >/dev/null || fail 'SIL oracle lacks async Int32 lowering'
grep -F '$@convention(thin) @async (Int32) -> (Int32, @error any Error)' "$probe_dir/host/oracle.sil" >/dev/null || fail 'SIL oracle lacks async throws Int32 lowering'

symbol_for() {
    object_path=$1
    function_name=$2
    nm -g "$object_path" | awk -v needle="$function_name" '$2 == "T" && index($3, needle) > 0 { print $3; exit }'
}

identity_symbol=$(symbol_for "$probe_dir/host/oracle.o" asyncIdentity)
throws_symbol=$(symbol_for "$probe_dir/host/oracle.o" asyncThrowsIdentity)
[ -n "$identity_symbol" ] || fail 'Swift oracle lacks asyncIdentity symbol'
[ -n "$throws_symbol" ] || fail 'Swift oracle lacks asyncThrowsIdentity symbol'

for target_name in device simulator; do
    target_identity_symbol=$(symbol_for "$probe_dir/$target_name/oracle.o" asyncIdentity)
    target_throws_symbol=$(symbol_for "$probe_dir/$target_name/oracle.o" asyncThrowsIdentity)
    [ "$target_identity_symbol" = "$identity_symbol" ] || fail "$target_name asyncIdentity symbol differs from host oracle"
    [ "$target_throws_symbol" = "$throws_symbol" ] || fail "$target_name asyncThrowsIdentity symbol differs from host oracle"
done

identity_ir_symbol=${identity_symbol#_}
throws_ir_symbol=${throws_symbol#_}

for target_name in host device simulator; do
    target_ir="$probe_dir/$target_name/oracle.ll"
    grep -F "define swifttailcc void @\"$identity_ir_symbol\"(ptr swiftasync %0, i32 %1)" "$target_ir" >/dev/null || fail "$target_name Swift IR asyncIdentity signature differs from compiler-derived signature"
    grep -F "define swifttailcc void @\"$throws_ir_symbol\"(ptr swiftasync %0, i32 %1)" "$target_ir" >/dev/null || fail "$target_name Swift IR async throws signature differs from compiler-derived signature"
    grep -F "musttail call swifttailcc void @\"$identity_ir_symbol\"(ptr swiftasync" "$target_ir" >/dev/null || fail "$target_name Swift caller oracle lacks asyncIdentity call"
    grep -F "musttail call swifttailcc void @\"$throws_ir_symbol\"(ptr swiftasync" "$target_ir" >/dev/null || fail "$target_name Swift caller oracle lacks asyncThrowsIdentity call"
done

grep -F 'call swiftcc ptr @swift_task_alloc' "$probe_dir/host/oracle.ll" >/dev/null || fail 'Swift caller oracle lacks compiler-emitted task allocation'
grep -F 'call swiftcc void @swift_task_dealloc' "$probe_dir/host/oracle.ll" >/dev/null || fail 'Swift caller oracle lacks compiler-emitted task deallocation'
grep -F 'musttail call swifttailcc void @swift_task_switch' "$probe_dir/host/oracle.ll" >/dev/null || fail 'async throws oracle lacks compiler-emitted task switch'
grep -E '^define internal swifttailcc void @".*asyncCaller.*FTQ0_"\(ptr swiftasync %0, i32 %1\)' "$probe_dir/host/oracle.ll" >/dev/null || fail 'Swift async caller resume signature was not found'
grep -E '^define internal swifttailcc void @".*asyncThrowsCaller.*FTQ0_"\(ptr swiftasync %0, i32 %1, ptr swiftself %2\)' "$probe_dir/host/oracle.ll" >/dev/null || fail 'Swift async throws caller resume/error signature was not found'

cat > "$probe_dir/Thunk.c" <<'C'
#include <stdint.h>
#if !__has_attribute(swiftasynccall)
#error "active Clang lacks swiftasynccall"
#endif
#if !__has_attribute(swift_async_context)
#error "active Clang lacks swift_async_context"
#endif
#if !__has_extension(swiftasynccc)
#error "active Clang target lacks swiftasynccc"
#endif
C
printf 'extern void swift_async_identity(void *context __attribute__((swift_async_context)), int32_t value) __attribute__((swiftasynccall)) __asm__("%s");\n' "$identity_symbol" >> "$probe_dir/Thunk.c"
printf 'extern void swift_async_throws_identity(void *context __attribute__((swift_async_context)), int32_t value) __attribute__((swiftasynccall)) __asm__("%s");\n' "$throws_symbol" >> "$probe_dir/Thunk.c"
cat >> "$probe_dir/Thunk.c" <<'C'
__attribute__((swiftasynccall)) void clang_async_identity(void *context __attribute__((swift_async_context)), int32_t value) { return swift_async_identity(context, value); }
__attribute__((swiftasynccall)) void clang_async_throws_identity(void *context __attribute__((swift_async_context)), int32_t value) { return swift_async_throws_identity(context, value); }
C

compile_clang_target() {
    target_name=$1
    target_triple=$2
    sdk_name=$3
    target_sdk=$4
    target_dir="$probe_dir/$target_name"
    xcrun --sdk "$sdk_name" clang -std=c11 -target "$target_triple" -isysroot "$target_sdk" -S -emit-llvm -O2 -o "$target_dir/clang.ll" "$probe_dir/Thunk.c"
    xcrun --sdk "$sdk_name" clang -std=c11 -target "$target_triple" -isysroot "$target_sdk" -S -O2 -o "$target_dir/clang.s" "$probe_dir/Thunk.c"
    xcrun --sdk "$sdk_name" clang -std=c11 -target "$target_triple" -isysroot "$target_sdk" -c -O2 -o "$target_dir/clang.o" "$probe_dir/Thunk.c"
    grep -F 'define swifttailcc void @clang_async_identity(ptr noundef swiftasync %0, i32 noundef %1)' "$target_dir/clang.ll" >/dev/null || fail "$target_name Clang asyncIdentity forwarding signature differs"
    grep -F 'define swifttailcc void @clang_async_throws_identity(ptr noundef swiftasync %0, i32 noundef %1)' "$target_dir/clang.ll" >/dev/null || fail "$target_name Clang async throws forwarding signature differs"
    grep -F "musttail call swifttailcc void @\"\\01$identity_symbol\"(ptr noundef swiftasync %0, i32 noundef %1)" "$target_dir/clang.ll" >/dev/null || fail "$target_name Clang asyncIdentity call differs from compiler-derived ABI"
    grep -F "musttail call swifttailcc void @\"\\01$throws_symbol\"(ptr noundef swiftasync %0, i32 noundef %1)" "$target_dir/clang.ll" >/dev/null || fail "$target_name Clang async throws call differs from compiler-derived ABI"
    if [ "$target_name" = host ]; then
        awk -v symbol="$identity_symbol" '$1 == "jmp" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/clang.s" || fail 'host Clang asyncIdentity assembly lacks tail jump to Swift symbol'
        awk -v symbol="$throws_symbol" '$1 == "jmp" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/clang.s" || fail 'host Clang async throws assembly lacks tail jump to Swift symbol'
        awk -v symbol="$identity_symbol" '$1 == "jmp" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/oracle.s" || fail 'host Swift async caller assembly lacks tail jump to asyncIdentity'
        awk -v symbol="$throws_symbol" '$1 == "jmp" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/oracle.s" || fail 'host Swift async caller assembly lacks tail jump to asyncThrowsIdentity'
    else
        awk -v symbol="$identity_symbol" '$1 == "b" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/clang.s" || fail "$target_name Clang asyncIdentity assembly lacks tail branch to Swift symbol"
        awk -v symbol="$throws_symbol" '$1 == "b" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/clang.s" || fail "$target_name Clang async throws assembly lacks tail branch to Swift symbol"
        awk -v symbol="$identity_symbol" '$1 == "b" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/oracle.s" || fail "$target_name Swift async caller assembly lacks tail branch to asyncIdentity"
        awk -v symbol="$throws_symbol" '$1 == "b" && $2 == symbol { found = 1 } END { exit !found }' "$target_dir/oracle.s" || fail "$target_name Swift async caller assembly lacks tail branch to asyncThrowsIdentity"
    fi
    nm -u "$target_dir/clang.o" | grep -F "$identity_symbol" >/dev/null || fail "$target_name Clang object lacks asyncIdentity symbol reference"
    nm -u "$target_dir/clang.o" | grep -F "$throws_symbol" >/dev/null || fail "$target_name Clang object lacks asyncThrowsIdentity symbol reference"
}

compile_clang_target host "$host_target" macosx "$host_sdk"
compile_clang_target device "$device_target" iphoneos "$device_sdk"
compile_clang_target simulator "$simulator_target" iphonesimulator "$simulator_sdk"

printf 'Selected Xcode: %s\n' "$(xcode-select -p)"
printf 'macOS: %s (build %s)\n' "$(sw_vers -productVersion)" "$(sw_vers -buildVersion)"
printf 'Xcode: %s\n' "$(xcodebuild -version | tr '\n' ' ')"
printf 'Swift: %s\n' "$(xcrun swiftc --version | tr '\n' ' ')"
printf 'Clang: %s\n' "$(xcrun clang --version | sed -n '1,2p' | tr '\n' ' ')"
printf 'Host: %s; macOS SDK %s (%s)\n' "$host_target" "$host_sdk_version" "$host_sdk"
printf 'Device: %s; iPhoneOS SDK %s (%s)\n' "$device_target" "$device_sdk_version" "$device_sdk"
printf 'Simulator: %s; iPhoneSimulator SDK %s (%s)\n' "$simulator_target" "$simulator_sdk_version" "$simulator_sdk"
printf 'Swift flags: -parse-as-library -module-name SwiftAsyncThunkProof -target <target> -sdk <SDK> -Onone -emit-ir|-emit-assembly|-emit-object\n'
printf 'Clang flags: -std=c11 -target <target> -isysroot <SDK> -O2 -emit-llvm|-S|-c\n'
printf 'Swift asyncIdentity symbol: %s\n' "$identity_symbol"
printf 'Swift asyncThrowsIdentity symbol: %s\n' "$throws_symbol"
printf '%s\n' 'Swift IR lowering: swifttailcc void(ptr swiftasync, i32) for both entry functions; Clang IR and host/device/simulator tail branches match'
printf '%s\n' 'Swift caller oracle: task allocation/deallocation, async resume entries, task switch, and throws continuation/error register observed'
printf 'Host Swift runtime imports:\n'
nm -u "$probe_dir/host/libSwiftAsyncThunkProof.dylib" | awk '/swift_task_(alloc|dealloc|switch)/ { print $NF }'
printf 'Host Swift library dependencies:\n'
otool -L "$probe_dir/host/libSwiftAsyncThunkProof.dylib"
printf '%s\n' 'No Swift async entry was invoked: the compiler oracle does not establish a public Rust/C task-context allocation, lifetime, resume, executor, and error-delivery contract'
