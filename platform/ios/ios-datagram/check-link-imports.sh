#!/bin/sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/ios-datagram-imports.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

expected_imports=$(printf '%s\n' libSystem.B.dylib libiconv.2.dylib | LC_ALL=C sort)
system_imports() {
    artifact_name=$1
    awk -v artifact_name="$artifact_name" \
        'NR > 1 && NF {path = $1; sub(/^.*\//, "", path); if (path != artifact_name) print path}' \
        | LC_ALL=C sort
}
matches_expected_imports() {
    artifact_name=$1
    output=$2
    actual=$(printf '%s\n' "$output" | system_imports "$artifact_name")
    [ "$actual" = "$expected_imports" ]
}
has_direct_iconv_symbol() {
    grep -Eqi '(^|[[:space:]])_+iconv(_|$)' "$1"
}
has_exact_symbol() {
    grep -Fxq "$1" "$2"
}
has_forbidden_link_reference() {
    pattern='swift_|nw_listener|nw_connection|Network.framework|objc_msgSend|objc_getClass'
    if grep -Eqi "$pattern" "$1"; then
        return 0
    fi
    printf '%s\n' "$2" | grep -Eqi "$pattern"
}

probe_name=libios_datagram_import_probe.dylib
probe_imports=$(cat <<EOF
/tmp/$probe_name:
    @rpath/$probe_name (compatibility version 0.0.0, current version 0.0.0)
    /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1351.0.0)
    /usr/lib/libiconv.2.dylib (compatibility version 7.0.0, current version 7.0.0)
EOF
)
if ! matches_expected_imports "$probe_name" "$probe_imports"; then
    printf 'link-import parser regression: probe install name was not excluded\n' >&2
    exit 1
fi
if matches_expected_imports "$probe_name" "$probe_imports
    /System/Library/Frameworks/Unexpected.framework/Unexpected"; then
    printf 'link-import parser regression: unexpected dylib was accepted\n' >&2
    exit 1
fi
printf '                 U _iconv_open\n' >"$scratch/iconv-symbols.txt"
if ! has_direct_iconv_symbol "$scratch/iconv-symbols.txt"; then
    printf 'symbol parser regression: direct iconv symbol was not detected\n' >&2
    exit 1
fi
printf '%s\n' _socket _socketpair >"$scratch/nm-j-symbols.txt"
if ! has_exact_symbol _socket "$scratch/nm-j-symbols.txt" \
    || has_exact_symbol _bind "$scratch/nm-j-symbols.txt"; then
    printf 'symbol parser regression: nm -j exact-symbol matching failed\n' >&2
    exit 1
fi
printf '                 U _swift_allocObject\n' >"$scratch/prohibited-symbols.txt"
if ! has_forbidden_link_reference "$scratch/prohibited-symbols.txt" "$expected_imports"; then
    printf 'link policy regression: prohibited Swift symbol was not detected\n' >&2
    exit 1
fi
printf '%s\n' _socket >"$scratch/prohibited-symbols.txt"
if ! has_forbidden_link_reference "$scratch/prohibited-symbols.txt" \
    '/System/Library/Frameworks/Network.framework/Network'; then
    printf 'link policy regression: prohibited framework import was not detected\n' >&2
    exit 1
fi
if has_forbidden_link_reference "$scratch/prohibited-symbols.txt" "$expected_imports"; then
    printf 'link policy regression: allowed system imports were rejected\n' >&2
    exit 1
fi

cat >"$scratch/Cargo.toml" <<EOF
[package]
name = "ios-datagram-import-probe"
version = "0.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
path = "$repo_root/platform/ios/ios-datagram/examples/link_probe.rs"

[dependencies]
ios-datagram = { path = "$repo_root/platform/ios/ios-datagram" }
EOF

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = "aarch64-apple-ios" ]; then
        sdk=iphoneos
        deployment_target=12.0
    else
        sdk=iphonesimulator
        deployment_target=14.0
    fi
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$scratch/target" \
        cargo build --manifest-path "$scratch/Cargo.toml" --release --target "$target"
    artifact="$scratch/target/$target/release/libios_datagram_import_probe.dylib"
    artifact_name=${artifact##*/}
    otool_output=$(otool -L "$artifact")
    imports=$(printf '%s\n' "$otool_output" | system_imports "$artifact_name")
    if ! matches_expected_imports "$artifact_name" "$otool_output"; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$imports" >&2
        exit 1
    fi
    symbols="$scratch/$target-undefined.txt"
    nm -u -j "$artifact" >"$symbols"
    if has_direct_iconv_symbol "$symbols"; then
        printf 'unexpected direct iconv symbol reference in %s\n' "$artifact" >&2
        exit 1
    fi
    for symbol in _socket _setsockopt _bind _sendto _recvfrom _poll _pipe _read _write _fcntl _getsockname _close; do
        if ! has_exact_symbol "$symbol" "$symbols"; then
            printf 'missing public BSD socket symbol %s in %s\n' "$symbol" "$symbols" >&2
            exit 1
        fi
    done
    if has_forbidden_link_reference "$symbols" "$imports"; then
        printf 'unexpected Swift, Objective-C, or Network.framework linkage in %s\n' "$artifact" >&2
        exit 1
    fi
    build_info="$scratch/$target-build.txt"
    vtool -show-build "$artifact" >"$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        printf '%s has deployment target %s; expected %s\n' \
            "$target" "${actual_deployment_target:-unknown}" "$deployment_target" >&2
        exit 1
    fi
    printf '%s\n' "$target imports:" "$imports"
done
