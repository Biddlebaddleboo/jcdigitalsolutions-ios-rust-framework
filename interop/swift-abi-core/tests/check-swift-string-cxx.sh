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

for tool in xcrun rustc nm awk grep mktemp otool; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-string-cxx.XXXXXX")
trap 'rm -rf "$probe_dir"' EXIT HUP INT TERM

case "$probe_dir/" in
    "$repo_root/"*) fail 'temporary oracle dir must stay outside the checkout' ;;
esac

host_target_info=$(xcrun swiftc -print-target-info)
host_swift_target=$(printf '%s\n' "$host_target_info" | awk -F '"' '/"triple"/ { print $4; exit }')
case "$host_swift_target" in
    *-apple-macosx*) ;;
    *) fail "unsupported Swift host target: $host_swift_target" ;;
esac
host_swift_arch=${host_swift_target%%-apple-macosx*}
host_macos_min=${host_swift_target##*-apple-macosx}
rust_host_target=$(rustc -vV | awk -F ': ' '/^host:/ { print $2; exit }')
rust_host_arch=${rust_host_target%%-apple-darwin}
[ "$host_swift_arch" = "$rust_host_arch" ] || fail "Swift and Rust host architectures differ: $host_swift_arch vs $rust_host_arch"

swiftc_path=$(xcrun --find swiftc)
swift_toolchain_root=${swiftc_path%/usr/bin/swiftc}
[ -f "$swift_toolchain_root/usr/include/swift/bridging" ] || fail 'active toolchain lacks the public Swift C++ bridging header'
swift_runtime_dir=/usr/lib/swift
[ -d "$swift_runtime_dir" ] || fail 'system Swift runtime directory is unavailable'
printf '%s\n' "$host_target_info" | grep -F '"/usr/lib/swift"' >/dev/null || fail 'active compiler does not report /usr/lib/swift as a runtime path'

mkdir "$probe_dir/host" "$probe_dir/device" "$probe_dir/simulator"

cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public func makeString(_ bytes:[UInt8])->String{String(decoding:bytes,as:UTF8.self)}
public func roundTrip(_ value:String)->String{value}
SWIFT

cat > "$probe_dir/shim.cpp" <<'CPP'
#include <string>
#include "SwiftStringProof.h"
#include <cstdint>
#include <cstring>
extern "C" int32_t framework_swift_string_round_trip(const uint8_t *input,uint64_t input_len,uint8_t *output,uint64_t output_capacity,uint64_t *output_len){if(output_len==nullptr)return 1;*output_len=0;if(input_len!=0&&input==nullptr)return 2;if(output_capacity!=0&&output==nullptr)return 3;try{auto bytes=swift::Array<uint8_t>::init();for(uint64_t i=0;i<input_len;i++)bytes.append(input[i]);auto input_string=SwiftStringProof::makeString(bytes);swift::String copied(input_string);auto output_string=SwiftStringProof::roundTrip(copied);std::string output_bytes=static_cast<std::string>(output_string);*output_len=static_cast<uint64_t>(output_bytes.size());if(*output_len>output_capacity)return 4;if(*output_len!=0)std::memcpy(output,output_bytes.data(),static_cast<size_t>(*output_len));return 0;}catch(...){return 5;}}
CPP

cat > "$probe_dir/caller.rs" <<'RUST'
use std::ffi::c_uchar;
unsafe extern "C" {
    fn framework_swift_string_round_trip(input: *const c_uchar, input_len: u64, output: *mut c_uchar, output_capacity: u64, output_len: *mut u64) -> i32;
}
fn main() {
    let cases: [(&str, &[u8]); 6] = [
        ("empty", &[]),
        ("ASCII", b"ASCII"),
        ("embedded NUL", b"A\0B"),
        ("non-ASCII BMP", "é".as_bytes()),
        ("combining marks", "e\u{301}".as_bytes()),
        ("multi-scalar emoji", "👩‍💻".as_bytes()),
    ];
    for (name, input) in cases {
        let mut output = [0u8; 128];
        let mut output_len = 0u64;
        let result = unsafe {
            framework_swift_string_round_trip(input.as_ptr(), input.len() as u64, output.as_mut_ptr(), output.len() as u64, &mut output_len)
        };
        assert_eq!(result, 0, "{name}: C ABI status");
        assert_eq!(output_len, input.len() as u64, "{name}: output length");
        assert_eq!(&output[..output_len as usize], input, "{name}: exact UTF-8 bytes");
        println!("pass {name}: {} bytes, exact input/output match", input.len());
    }
}
RUST

cat > "$probe_dir/target-caller.rs" <<'RUST'
#![no_std]
unsafe extern "C" {
    fn framework_swift_string_round_trip(input: *const u8, input_len: u64, output: *mut u8, output_capacity: u64, output_len: *mut u64) -> i32;
}
#[no_mangle]
pub extern "C" fn rust_swift_string_probe(input: *const u8, input_len: u64, output: *mut u8, output_capacity: u64, output_len: *mut u64) -> i32 {
    unsafe { framework_swift_string_round_trip(input, input_len, output, output_capacity, output_len) }
}
RUST

host_dir="$probe_dir/host"
xcrun swiftc -parse-as-library -module-name SwiftStringProof -cxx-interoperability-mode=default -emit-clang-header-path "$host_dir/SwiftStringProof.h" -target "$host_swift_target" -emit-library -o "$host_dir/libSwiftStringProof.dylib" "$probe_dir/Oracle.swift"
grep -F 'swift::String makeString(const swift::Array<uint8_t>& bytes)' "$host_dir/SwiftStringProof.h" >/dev/null || fail 'generated header lacks the byte-array String constructor API'
grep -F 'swift::String roundTrip(const swift::String& value)' "$host_dir/SwiftStringProof.h" >/dev/null || fail 'generated header lacks the String round-trip API'
grep -F 'vwTable->initializeWithCopy' "$host_dir/SwiftStringProof.h" >/dev/null || fail 'generated String copy does not use the compiler-provided value witness'
grep -F 'vwTable->destroy(_getOpaquePointer(), metadata._0);' "$host_dir/SwiftStringProof.h" >/dev/null || fail 'generated String destruction does not use the compiler-provided value witness'

host_sdk=$(xcrun --sdk macosx --show-sdk-path)
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -S -emit-llvm -o "$host_dir/shim.ll" "$probe_dir/shim.cpp"
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -S -o "$host_dir/shim.s" "$probe_dir/shim.cpp"
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -c -o "$host_dir/shim.o" "$probe_dir/shim.cpp"
grep -F 'StringC1ERKS0_' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks an exercised String copy'
grep -F 'StringD1Ev' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks exercised String destruction'

host_make_symbol=$(nm -g "$host_dir/libSwiftStringProof.dylib" | awk '$2 == "T" && $3 ~ /makeB0/ { print $3; exit }')
host_round_trip_symbol=$(nm -g "$host_dir/libSwiftStringProof.dylib" | awk '$2 == "T" && $3 ~ /roundTrip/ { print $3; exit }')
[ -n "$host_make_symbol" ] || fail 'host Swift library lacks the compiler-emitted makeString symbol'
[ -n "$host_round_trip_symbol" ] || fail 'host Swift library lacks the compiler-emitted roundTrip symbol'
nm -u "$host_dir/shim.o" | grep -F "$host_make_symbol" >/dev/null || fail 'host C++ object lacks the compiler-emitted makeString reference'
nm -u "$host_dir/shim.o" | grep -F "$host_round_trip_symbol" >/dev/null || fail 'host C++ object lacks the compiler-emitted roundTrip reference'

rustc --edition=2024 --crate-name swift_string_probe "$probe_dir/caller.rs" \
    -C "link-arg=$host_dir/shim.o" \
    -C "link-arg=$host_dir/libSwiftStringProof.dylib" \
    -C link-arg=-lc++ \
    -C link-arg=-L/usr/lib/swift \
    -C link-arg=-lswiftCore \
    -C "link-arg=-Wl,-rpath,$swift_runtime_dir" \
    -C "link-arg=-Wl,-rpath,$host_dir" \
    -C "link-arg=-mmacosx-version-min=$host_macos_min" \
    -o "$host_dir/rust-host-probe"
DYLD_PRINT_LIBRARIES=1 "$host_dir/rust-host-probe" 2> "$host_dir/runtime-loads.txt"
swift_core_load_count=$(grep -F 'libswiftCore.dylib' "$host_dir/runtime-loads.txt" | wc -l | tr -d ' ')
[ "$swift_core_load_count" -eq 1 ] || fail "host proof loaded $swift_core_load_count Swift core runtimes; expected one"
grep -F '/usr/lib/swift/libswiftCore.dylib' "$host_dir/runtime-loads.txt" >/dev/null || fail 'host proof did not load the system Swift core runtime'
swift_core_link_count=$(otool -L "$host_dir/rust-host-probe" | grep -F 'libswiftCore.dylib' | wc -l | tr -d ' ')
[ "$swift_core_link_count" -eq 1 ] || fail "host executable links $swift_core_link_count Swift core runtimes; expected one"

build_apple_target() {
    target_name=$1
    sdk_name=$2
    target_triple=$3
    rust_target=$4
    target_dir="$probe_dir/$target_name"
    sdk_path=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun swiftc -parse-as-library -module-name SwiftStringProof -cxx-interoperability-mode=default -emit-clang-header-path "$target_dir/SwiftStringProof.h" -target "$target_triple" -sdk "$sdk_path" -emit-object -o "$target_dir/oracle.o" "$probe_dir/Oracle.swift"
    xcrun --sdk "$sdk_name" clang++ -target "$target_triple" -isysroot "$sdk_path" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$target_dir" -c -o "$target_dir/shim.o" "$probe_dir/shim.cpp"
    rustc --edition=2021 --crate-type lib --target "$rust_target" --emit=obj "$probe_dir/target-caller.rs" -o "$target_dir/rust-caller.o"
    grep -F 'swift::String makeString(const swift::Array<uint8_t>& bytes)' "$target_dir/SwiftStringProof.h" >/dev/null || fail "$target_name generated header lacks the byte-array String constructor API"
    grep -F 'swift::String roundTrip(const swift::String& value)' "$target_dir/SwiftStringProof.h" >/dev/null || fail "$target_name generated header lacks the String round-trip API"
    target_make_symbol=$(nm -g "$target_dir/oracle.o" | awk '$2 == "T" && $3 ~ /makeB0/ { print $3; exit }')
    target_round_trip_symbol=$(nm -g "$target_dir/oracle.o" | awk '$2 == "T" && $3 ~ /roundTrip/ { print $3; exit }')
    [ "$target_make_symbol" = "$host_make_symbol" ] || fail "$target_name makeString symbol differs from host oracle"
    [ "$target_round_trip_symbol" = "$host_round_trip_symbol" ] || fail "$target_name roundTrip symbol differs from host oracle"
    nm -u "$target_dir/shim.o" | grep -F "$target_make_symbol" >/dev/null || fail "$target_name C++ object lacks the compiler-emitted makeString reference"
    nm -u "$target_dir/shim.o" | grep -F "$target_round_trip_symbol" >/dev/null || fail "$target_name C++ object lacks the compiler-emitted roundTrip reference"
    nm -u "$target_dir/rust-caller.o" | grep -F '_framework_swift_string_round_trip' >/dev/null || fail "$target_name Rust object lacks the fixed-width C boundary reference"
}

build_apple_target device iphoneos arm64-apple-ios17.0 aarch64-apple-ios
build_apple_target simulator iphonesimulator arm64-apple-ios17.0-simulator aarch64-apple-ios-sim

printf 'Swift host target: %s\n' "$host_swift_target"
printf 'Compiler-emitted Swift symbols: %s %s\n' "$host_make_symbol" "$host_round_trip_symbol"
printf '%s\n' 'host Swift C++ header, generated wrapper, Rust C ABI caller: passed'
printf '%s\n' 'host round-trip fixtures: passed (empty, ASCII, embedded NUL, BMP, combining marks, multi-scalar emoji)'
printf '%s\n' 'host C++ ownership: exercised generated String copy and destruction; IR confirms both operations'
printf '%s\n' 'host runtime: exactly one /usr/lib/swift/libswiftCore.dylib loaded and linked'
printf '%s\n' 'device Swift, C++, and Rust objects: passed (compile-only, iOS 17.0 target)'
printf '%s\n' 'simulator Swift, C++, and Rust objects: passed (compile-only, iOS 17.0 target)'
