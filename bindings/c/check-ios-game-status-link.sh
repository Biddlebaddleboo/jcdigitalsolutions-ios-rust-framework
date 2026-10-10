#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

rustc_sysroot=$(rustc --print sysroot)
rustc_host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$rustc_sysroot/lib/rustlib/$rustc_host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for Rust archive scans: $llvm_nm" >&2
    exit 1
fi

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort strings xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F33 link/import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-game-status.sh

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-game-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-game-status -- -D warnings
cargo doc --locked --no-deps -p framework-c-api --no-default-features --features ios-game-status
cargo build --locked --release -p framework-c-api --no-default-features
cargo build --locked --release -p framework-c-api --no-default-features --features ios-game-status

expected_symbols=target/framework-c-ios-game-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_game_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"
host_archive=target/release/libframework_c_api.a
for language in c cpp; do
    case "$language" in
        c) compiler=clang; source=target/framework-c-ios-game-status-c.c; standard=c11 ;;
        cpp) compiler=clang++; source=target/framework-c-ios-game-status-cpp.cpp; standard=c++17 ;;
    esac
    binary="target/framework-c-ios-game-status-$language-host"
    "$compiler" -std="$standard" -Wall -Wextra -Werror -pedantic -nostdlib++ \
        -I bindings/c/include "$source" "$host_archive" -o "$binary"
    nm -u "$binary" 2>/dev/null > "target/framework-c-ios-game-status-$language-host-undefined.txt"
    if rg -q 'GameKit|GKLocalPlayer|objc_|OBJC_CLASS|swift_' \
        "target/framework-c-ios-game-status-$language-host-undefined.txt"; then
        echo "GameKit, Objective-C, or Swift import leaked into host $language consumer" >&2
        exit 1
    fi
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-game-status-$language-host-libraries.txt"
    printf '%s\n' 'libSystem.B.dylib' \
        > "target/framework-c-ios-game-status-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-game-status-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-game-status-$language-host-libraries.txt"
done
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_game_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u > target/framework-c-ios-game-status-host-archive-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-game-status-host-archive-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=10.0
            sdk=iphoneos
            clang_target=arm64-apple-ios10.0
            rust_min_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            rust_min_flag=-mios-simulator-version-min=14.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-game-status --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-game-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-game-status --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-game-status-c.c; standard=c11; cpp_only_flags= ;;
            cpp) compiler=clang++; source=target/framework-c-ios-game-status-cpp.cpp; standard=c++17; cpp_only_flags=-nostdinc++ ;;
        esac
        binary="target/framework-c-ios-game-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -nostdlib++ $cpp_only_flags -I bindings/c/include "$source" "$archive" \
            -framework GameKit -framework Foundation -lobjc -o "$binary"
        imports="target/framework-c-ios-game-status-$language-$target-imports.txt"
        otool -L "$binary" > "$imports"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "target/framework-c-ios-game-status-$language-$target-libraries.txt"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_game_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-game-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-game-status-$language-imports-expected.txt" \
            "target/framework-c-ios-game-status-$language-$target-libraries.txt"
        symbols="target/framework-c-ios-game-status-$language-$target-undefined.txt"
        nm -m "$binary" 2>/dev/null > "$symbols"
        strings -a "$binary" > "target/framework-c-ios-game-status-$language-$target-strings.txt"
        rg -q 'GKLocalPlayer' "$symbols"
        rg -q 'localPlayer' "target/framework-c-ios-game-status-$language-$target-strings.txt"
        rg -q 'isAuthenticated' "target/framework-c-ios-game-status-$language-$target-strings.txt"
        if rg -q 'authenticateHandler|authenticateWithCompletionHandler|GKPlayerAuthenticationDidChangeNotificationName|GKGameCenterViewController|playerID|teamPlayerID|swift_' \
            "$symbols" "target/framework-c-ios-game-status-$language-$target-strings.txt"; then
            echo "out-of-scope Game Center auth, identity, UI, or Swift API in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-game-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports %s; minos %s; consumer not executed\n' \
            "$target" "$language" "$(tr '\n' ' ' < "target/framework-c-ios-game-status-$language-$target-libraries.txt")" "$actual_deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_game_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-game-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" "target/framework-c-ios-game-status-$target-archive-symbols.txt"
done

printf 'F33 link/import gate complete; no tests, consumers, probes, or Game Center calls were executed\n'
