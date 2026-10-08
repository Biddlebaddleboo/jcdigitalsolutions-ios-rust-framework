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

for tool in xcrun xcodebuild rustc nm awk grep mktemp otool; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-async-cxx.XXXXXX")
cleanup_probe() {
    if [ "${KEEP_SWIFT_ASYNC_PROBE:-0}" = 1 ]; then
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
host_swift_target=$(printf '%s\n' "$host_target_info" | awk -F '"' '/"triple"/ { print $4; exit }')
case "$host_swift_target" in
    *-apple-macosx*) ;;
    *) fail "unsupported Swift host target: $host_swift_target" ;;
esac
swiftc_path=$(xcrun --find swiftc)
swift_toolchain_root=${swiftc_path%/usr/bin/swiftc}
[ -f "$swift_toolchain_root/usr/include/swift/bridging" ] || fail 'active toolchain lacks the public Swift C++ bridging header'
xcrun swiftc --help | grep -F -- '-cxx-interoperability-mode' >/dev/null || fail 'active swiftc does not document -cxx-interoperability-mode'
xcrun swiftc -frontend -help | grep -F -- '-emit-clang-header-path' >/dev/null || fail 'active Swift frontend does not document -emit-clang-header-path'

mkdir "$probe_dir/host"
host_dir="$probe_dir/host"

cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public func syncString(_ value:String)->String{value}
public func asyncString(_ value:String) async -> String{value}
public func asyncThrowsString(_ value:String) async throws -> String{value}
SWIFT

if ! xcrun swiftc -parse-as-library -module-name SwiftAsyncProof -cxx-interoperability-mode=default -emit-clang-header-path "$host_dir/SwiftAsyncProof.h" -target "$host_swift_target" -emit-library -o "$host_dir/libSwiftAsyncProof.dylib" "$probe_dir/Oracle.swift" >"$host_dir/compiler.stdout" 2>"$host_dir/compiler.stderr"; then
    cat "$host_dir/compiler.stdout" >&2
    cat "$host_dir/compiler.stderr" >&2
    fail 'public Swift C++ header generation failed'
fi

sync_signature=$(grep -F 'SWIFT_INLINE_THUNK swift::String syncString(const swift::String& value) noexcept' "$host_dir/SwiftAsyncProof.h" || true)
[ -n "$sync_signature" ] || fail 'generated header lacks the synchronous String control API'
async_unavailable="// Unavailable in C++: Swift global function 'asyncString(_:)'."
async_throws_unavailable="// Unavailable in C++: Swift global function 'asyncThrowsString(_:)'."
grep -F "$async_unavailable" "$host_dir/SwiftAsyncProof.h" >/dev/null || fail 'generated header lacks the exact async omission comment'
grep -F "$async_throws_unavailable" "$host_dir/SwiftAsyncProof.h" >/dev/null || fail 'generated header lacks the exact async throws omission comment'
if grep -E 'SWIFT_(INLINE_THUNK|EXTERN).*asyncString|SWIFT_(INLINE_THUNK|EXTERN).*asyncThrowsString' "$host_dir/SwiftAsyncProof.h" >/dev/null; then
    fail 'generated C++ API now exposes an async function; extend the bounded call-path proof before proceeding'
fi

sync_symbol=$(nm -g "$host_dir/libSwiftAsyncProof.dylib" | awk '$2 == "T" && $3 ~ /syncString/ { print $3; exit }')
async_symbol=$(nm -g "$host_dir/libSwiftAsyncProof.dylib" | awk '$2 == "T" && $3 ~ /asyncString/ { print $3; exit }')
async_throws_symbol=$(nm -g "$host_dir/libSwiftAsyncProof.dylib" | awk '$2 == "T" && $3 ~ /asyncThrowsString/ { print $3; exit }')
[ -n "$sync_symbol" ] || fail 'Swift oracle lacks its compiler-emitted synchronous control symbol'
[ -n "$async_symbol" ] || fail 'Swift oracle lacks its compiler-emitted async symbol'
[ -n "$async_throws_symbol" ] || fail 'Swift oracle lacks its compiler-emitted async throws symbol'
nm -g "$host_dir/libSwiftAsyncProof.dylib" | awk '$2 == "T" && ($3 ~ /syncString|asyncString|asyncThrowsString/) { print $3 }' > "$host_dir/oracle-symbols.txt"
nm -u "$host_dir/libSwiftAsyncProof.dylib" > "$host_dir/oracle-undefined-symbols.txt"
otool -L "$host_dir/libSwiftAsyncProof.dylib" > "$host_dir/oracle-linkage.txt"

cat > "$probe_dir/load-oracle.c" <<'C'
#include <dlfcn.h>
#include <stdio.h>
int main(int argc,char **argv){if(argc!=2)return 2;void *handle=dlopen(argv[1],RTLD_NOW);if(handle==NULL){fprintf(stderr,"%s\n",dlerror());return 1;}return dlclose(handle)!=0;}
C
xcrun clang -target "$host_swift_target" -isysroot "$(xcrun --sdk macosx --show-sdk-path)" "$probe_dir/load-oracle.c" -o "$host_dir/load-oracle"
DYLD_PRINT_LIBRARIES=1 "$host_dir/load-oracle" "$host_dir/libSwiftAsyncProof.dylib" > "$host_dir/load.stdout" 2> "$host_dir/runtime-loads.txt"
swift_core_load_count=$(grep -F '/usr/lib/swift/libswiftCore.dylib' "$host_dir/runtime-loads.txt" | wc -l | tr -d ' ')
[ "$swift_core_load_count" -eq 1 ] || fail "host loader saw $swift_core_load_count libswiftCore path loads; expected one"
swift_concurrency_load_count=$(grep -F '/usr/lib/swift/libswift_Concurrency.dylib' "$host_dir/runtime-loads.txt" | wc -l | tr -d ' ')
libcxx_load_count=$(grep -F '/usr/lib/libc++.1.dylib' "$host_dir/runtime-loads.txt" | wc -l | tr -d ' ')

printf 'Xcode: %s\n' "$(xcodebuild -version | tr '\n' ' ')"
printf 'Swift: %s\n' "$(xcrun swiftc --version | tr '\n' ' ')"
printf 'Clang: %s\n' "$(xcrun clang++ --version | sed -n '1,2p' | tr '\n' ' ')"
printf 'Rust: %s\n' "$(rustc --version --verbose | sed -n '1,7p' | tr '\n' ' ')"
printf 'Host Swift target: %s\n' "$host_swift_target"
printf 'macOS SDK: %s (%s)\n' "$(xcrun --sdk macosx --show-sdk-path)" "$(xcrun --sdk macosx --show-sdk-version)"
printf 'iPhoneOS SDK: %s (%s)\n' "$(xcrun --sdk iphoneos --show-sdk-path)" "$(xcrun --sdk iphoneos --show-sdk-version)"
printf 'iPhoneSimulator SDK: %s (%s)\n' "$(xcrun --sdk iphonesimulator --show-sdk-path)" "$(xcrun --sdk iphonesimulator --show-sdk-version)"
printf 'Generated sync signature: %s\n' "$sync_signature"
printf 'Generated async omission: %s\n' "$async_unavailable"
printf 'Generated async throws omission: %s\n' "$async_throws_unavailable"
printf 'Compiler stdout bytes: %s; stderr bytes: %s\n' "$(wc -c < "$host_dir/compiler.stdout" | tr -d ' ')" "$(wc -c < "$host_dir/compiler.stderr" | tr -d ' ')"
printf 'Swift oracle symbols: %s %s %s\n' "$sync_symbol" "$async_symbol" "$async_throws_symbol"
printf 'Host Swift library linkage:\n'
cat "$host_dir/oracle-linkage.txt"
printf 'Host async/task-related undefined symbols:\n'
if grep -Ei 'swift_task|swift_continuation|swift_async' "$host_dir/oracle-undefined-symbols.txt"; then :; else printf '%s\n' '(none in this Swift library undefined-symbol list)'; fi
printf 'Host loader path loads: libswiftCore=%s, libswift_Concurrency=%s, libc++=%s\n' "$swift_core_load_count" "$swift_concurrency_load_count" "$libcxx_load_count"
printf '%s\n' 'Swift-generated C++ API omits both async declarations; C++/Rust async call-path probe stopped as specified'
