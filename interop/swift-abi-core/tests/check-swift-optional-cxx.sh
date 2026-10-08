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
probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-optional-cxx.XXXXXX")
cleanup_probe() {
    if [ "${KEEP_SWIFT_OPTIONAL_PROBE:-0}" = 1 ]; then
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
xcrun swiftc --help | grep -F -- '-cxx-interoperability-mode' >/dev/null || fail 'active swiftc does not document -cxx-interoperability-mode'
xcrun swiftc -frontend -help | grep -F -- '-emit-clang-header-path' >/dev/null || fail 'active swift frontend does not document -emit-clang-header-path'

mkdir "$probe_dir/host" "$probe_dir/device" "$probe_dir/simulator"

cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public func makeOptional(_ bytes:[UInt8],_ present:Bool)->String?{guard present else{return nil};return String(decoding:bytes,as:UTF8.self)}
public func roundTrip(_ value:String?)->String?{value}
SWIFT

cat > "$probe_dir/shim.cpp" <<'CPP'
#include <string>
#include "SwiftOptionalProof.h"
#include <cstdint>
#include <cstring>
extern "C" int32_t framework_swift_optional_string_round_trip(uint32_t input_has_value,const uint8_t *input,uint64_t input_len,uint8_t *output,uint64_t output_capacity,uint32_t *output_has_value,uint64_t *output_len){if(output_has_value==nullptr||output_len==nullptr)return 1;*output_has_value=0;*output_len=0;if(input_has_value>1)return 2;if(input_len!=0&&input==nullptr)return 3;if(output_capacity!=0&&output==nullptr)return 4;try{auto bytes=swift::Array<uint8_t>::init();for(uint64_t i=0;i<input_len;i++)bytes.append(input[i]);auto input_optional=SwiftOptionalProof::makeOptional(bytes,input_has_value!=0);swift::Optional<swift::String> copied(input_optional);auto output_optional=SwiftOptionalProof::roundTrip(copied);if(output_optional.isNone())return 0;*output_has_value=1;auto output_string=output_optional.getSome();swift::String copied_string(output_string);std::string output_bytes=static_cast<std::string>(copied_string);*output_len=static_cast<uint64_t>(output_bytes.size());if(*output_len>output_capacity)return 5;if(*output_len!=0)std::memcpy(output,output_bytes.data(),static_cast<size_t>(*output_len));return 0;}catch(...){return 6;}}
CPP

cat > "$probe_dir/caller.rs" <<'RUST'
unsafe extern "C" {
    fn framework_swift_optional_string_round_trip(input_has_value: u32, input: *const u8, input_len: u64, output: *mut u8, output_capacity: u64, output_has_value: *mut u32, output_len: *mut u64) -> i32;
}
fn main() {
    let cases: [(&str, bool, &[u8]); 7] = [
        ("none", false, &[]),
        ("some empty", true, &[]),
        ("ASCII", true, b"ASCII"),
        ("embedded NUL", true, b"A\0B"),
        ("non-ASCII BMP", true, "é".as_bytes()),
        ("combining marks", true, "e\u{301}".as_bytes()),
        ("multi-scalar emoji", true, "👩‍💻".as_bytes()),
    ];
    for (name, has_value, input) in cases {
        let mut output = [0u8; 128];
        let mut output_has_value = u32::MAX;
        let mut output_len = u64::MAX;
        let input_pointer = if has_value { input.as_ptr() } else { core::ptr::null() };
        let result = unsafe {
            framework_swift_optional_string_round_trip(has_value as u32, input_pointer, input.len() as u64, output.as_mut_ptr(), output.len() as u64, &mut output_has_value, &mut output_len)
        };
        assert_eq!(result, 0, "{name}: C ABI status");
        assert_eq!(output_has_value, has_value as u32, "{name}: optional state");
        assert_eq!(output_len, input.len() as u64, "{name}: output length");
        assert_eq!(&output[..output_len as usize], input, "{name}: exact UTF-8 bytes");
        println!("pass {name}: optional={}, {} bytes, exact input/output match", output_has_value, output_len);
    }
    let mut output_has_value = u32::MAX;
    let mut output_len = u64::MAX;
    let invalid = unsafe {
        framework_swift_optional_string_round_trip(2, core::ptr::null(), 0, core::ptr::null_mut(), 0, &mut output_has_value, &mut output_len)
    };
    assert_eq!(invalid, 2, "invalid optional-state status");
}
RUST

cat > "$probe_dir/target-caller.rs" <<'RUST'
#![no_std]
unsafe extern "C" {
    fn framework_swift_optional_string_round_trip(input_has_value: u32, input: *const u8, input_len: u64, output: *mut u8, output_capacity: u64, output_has_value: *mut u32, output_len: *mut u64) -> i32;
}
#[no_mangle]
pub extern "C" fn rust_swift_optional_string_probe(input_has_value: u32, input: *const u8, input_len: u64, output: *mut u8, output_capacity: u64, output_has_value: *mut u32, output_len: *mut u64) -> i32 {
    unsafe { framework_swift_optional_string_round_trip(input_has_value, input, input_len, output, output_capacity, output_has_value, output_len) }
}
RUST

host_dir="$probe_dir/host"
xcrun swiftc -parse-as-library -module-name SwiftOptionalProof -cxx-interoperability-mode=default -emit-clang-header-path "$host_dir/SwiftOptionalProof.h" -target "$host_swift_target" -emit-library -o "$host_dir/libSwiftOptionalProof.dylib" "$probe_dir/Oracle.swift"
grep -F 'swift::Optional<swift::String> makeOptional(const swift::Array<uint8_t>& bytes, bool present)' "$host_dir/SwiftOptionalProof.h" >/dev/null || fail 'generated header lacks the Optional<String> byte-array constructor API'
grep -F 'swift::Optional<swift::String> roundTrip(const swift::Optional<swift::String>& value)' "$host_dir/SwiftOptionalProof.h" >/dev/null || fail 'generated header lacks the Optional<String> round-trip API'
awk 'index($0,"class SWIFT_SYMBOL(\"s:Sq\") Optional final"){capture=1} capture{print} capture && $0=="};"{exit}' "$host_dir/SwiftOptionalProof.h" > "$host_dir/optional-class.txt"
grep -F 'vwTable->initializeWithCopy(_getOpaquePointer(), const_cast<char *>(other._getOpaquePointer()), metadata._0);' "$host_dir/optional-class.txt" >/dev/null || fail 'generated Optional copy does not use its compiler-provided value witness'
grep -F 'vwTable->destroy(_getOpaquePointer(), metadata._0);' "$host_dir/optional-class.txt" >/dev/null || fail 'generated Optional destruction does not use its compiler-provided value witness'
grep -F 'T_0_0 getSome() const;' "$host_dir/optional-class.txt" >/dev/null || fail 'generated Optional wrapper lacks its compiler-generated some projection'
grep -F 'bool isNone() const;' "$host_dir/optional-class.txt" >/dev/null || fail 'generated Optional wrapper lacks its compiler-generated none query'
awk 'index($0,"class SWIFT_SYMBOL(\"s:SS\") String final"){capture=1} capture{print} capture && $0=="};"{exit}' "$host_dir/SwiftOptionalProof.h" > "$host_dir/string-class.txt"
grep -F 'vwTable->initializeWithCopy(_getOpaquePointer(), const_cast<char *>(other._getOpaquePointer()), metadata._0);' "$host_dir/string-class.txt" >/dev/null || fail 'generated String copy does not use its compiler-provided value witness'
grep -F 'vwTable->destroy(_getOpaquePointer(), metadata._0);' "$host_dir/string-class.txt" >/dev/null || fail 'generated String destruction does not use its compiler-provided value witness'

host_sdk=$(xcrun --sdk macosx --show-sdk-path)
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -S -emit-llvm -o "$host_dir/shim.ll" "$probe_dir/shim.cpp"
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -S -o "$host_dir/shim.s" "$probe_dir/shim.cpp"
xcrun clang++ -target "$host_swift_target" -isysroot "$host_sdk" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$host_dir" -c -o "$host_dir/shim.o" "$probe_dir/shim.cpp"
grep -F 'framework_swift_optional_string_round_trip' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks the fixed-width C boundary function'
grep -F '_ZN5swift8OptionalINS_6StringEEC2ERKS2_' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks the exercised Optional<String> copy constructor'
grep -F '_ZN5swift8OptionalINS_6StringEED2Ev' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks the exercised Optional<String> destructor'
grep -F '_ZN5swift6StringC1ERKS0_' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks the exercised String copy constructor'
grep -F '_ZN5swift6StringD1Ev' "$host_dir/shim.ll" >/dev/null || fail 'host C++ IR lacks the exercised String destructor'

host_make_symbol=$(nm -g "$host_dir/libSwiftOptionalProof.dylib" | awk '$2 == "T" && $3 ~ /makeB0/ { print $3; exit }')
host_round_trip_symbol=$(nm -g "$host_dir/libSwiftOptionalProof.dylib" | awk '$2 == "T" && $3 ~ /roundTrip/ { print $3; exit }')
[ -n "$host_make_symbol" ] || fail 'host Swift library lacks the compiler-emitted makeOptional symbol'
[ -n "$host_round_trip_symbol" ] || fail 'host Swift library lacks the compiler-emitted Optional roundTrip symbol'
nm -u "$host_dir/shim.o" | grep -F "$host_make_symbol" >/dev/null || fail 'host C++ object lacks the compiler-emitted makeOptional reference'
nm -u "$host_dir/shim.o" | grep -F "$host_round_trip_symbol" >/dev/null || fail 'host C++ object lacks the compiler-emitted Optional roundTrip reference'
nm -u "$host_dir/shim.o" > "$host_dir/shim-undefined-symbols.txt"
grep -F '_$sSqMa' "$host_dir/shim-undefined-symbols.txt" >/dev/null || fail 'host C++ object lacks Swift Optional metadata access'
grep -F '_$sSSMa' "$host_dir/shim-undefined-symbols.txt" >/dev/null || fail 'host C++ object lacks Swift String metadata access'

rustc --edition=2024 --crate-name swift_optional_probe "$probe_dir/caller.rs" \
    -C "link-arg=$host_dir/shim.o" \
    -C "link-arg=$host_dir/libSwiftOptionalProof.dylib" \
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
    xcrun swiftc -parse-as-library -module-name SwiftOptionalProof -cxx-interoperability-mode=default -emit-clang-header-path "$target_dir/SwiftOptionalProof.h" -target "$target_triple" -sdk "$sdk_path" -emit-object -o "$target_dir/oracle.o" "$probe_dir/Oracle.swift"
    xcrun --sdk "$sdk_name" clang++ -target "$target_triple" -isysroot "$sdk_path" -std=c++17 -O0 -I "$swift_toolchain_root/usr/include" -I "$target_dir" -c -o "$target_dir/shim.o" "$probe_dir/shim.cpp"
    rustc --edition=2021 --crate-type lib --target "$rust_target" --emit=obj "$probe_dir/target-caller.rs" -o "$target_dir/rust-caller.o"
    grep -F 'swift::Optional<swift::String> makeOptional(const swift::Array<uint8_t>& bytes, bool present)' "$target_dir/SwiftOptionalProof.h" >/dev/null || fail "$target_name generated header lacks the Optional<String> byte-array constructor API"
    grep -F 'swift::Optional<swift::String> roundTrip(const swift::Optional<swift::String>& value)' "$target_dir/SwiftOptionalProof.h" >/dev/null || fail "$target_name generated header lacks the Optional<String> round-trip API"
    target_make_symbol=$(nm -g "$target_dir/oracle.o" | awk '$2 == "T" && $3 ~ /makeB0/ { print $3; exit }')
    target_round_trip_symbol=$(nm -g "$target_dir/oracle.o" | awk '$2 == "T" && $3 ~ /roundTrip/ { print $3; exit }')
    [ "$target_make_symbol" = "$host_make_symbol" ] || fail "$target_name makeOptional symbol differs from host oracle"
    [ "$target_round_trip_symbol" = "$host_round_trip_symbol" ] || fail "$target_name Optional roundTrip symbol differs from host oracle"
    nm -u "$target_dir/shim.o" | grep -F "$target_make_symbol" >/dev/null || fail "$target_name C++ object lacks the compiler-emitted makeOptional reference"
    nm -u "$target_dir/shim.o" | grep -F "$target_round_trip_symbol" >/dev/null || fail "$target_name C++ object lacks the compiler-emitted Optional roundTrip reference"
    nm -u "$target_dir/rust-caller.o" | grep -F '_framework_swift_optional_string_round_trip' >/dev/null || fail "$target_name Rust object lacks the fixed-width C ABI function reference"
}

build_apple_target device iphoneos arm64-apple-ios17.0 aarch64-apple-ios
build_apple_target simulator iphonesimulator arm64-apple-ios17.0-simulator aarch64-apple-ios-sim

printf 'Swift host target: %s\n' "$host_swift_target"
printf 'Compiler-emitted Swift symbols: %s %s\n' "$host_make_symbol" "$host_round_trip_symbol"
printf '%s\n' 'host Swift C++ Optional<String> header, generated wrappers, Rust C ABI caller: passed'
printf '%s\n' 'host optional UTF-8 fixtures: passed (none, some empty, ASCII, embedded NUL, BMP, combining marks, multi-scalar emoji)'
printf '%s\n' 'host Optional<String> and String copy/destruction: generated header value witnesses and IR/object references verified'
printf '%s\n' 'host runtime: exactly one /usr/lib/swift/libswiftCore.dylib loaded and linked'
printf '%s\n' 'device Swift, C++, and Rust objects: passed (compile-only, iOS 17.0 target)'
printf '%s\n' 'simulator Swift, C++, and Rust objects: passed (compile-only, iOS 17.0 target)'
