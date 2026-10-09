#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F29 link-import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-natural-language-status.sh

cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-natural-language-status
host_archive=target/release/libframework_c_api.a
jq -r '.optional_capabilities.ios_natural_language_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u \
    > target/framework-c-ios-natural-language-status-expected-symbols.txt

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-natural-language-status-c.c "$host_archive" \
    -o target/framework-c-ios-natural-language-status-c-host
clang++ -nostdlib++ -std=c++17 -Wall -Wextra -Werror -pedantic \
    -I bindings/c/include target/framework-c-ios-natural-language-status-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-natural-language-status-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_natural_language_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-natural-language-status-host-symbols.txt
diff -u target/framework-c-ios-natural-language-status-expected-symbols.txt \
    target/framework-c-ios-natural-language-status-host-symbols.txt
nm -u "$host_archive" 2>/dev/null > target/framework-c-ios-natural-language-status-host-undefined.txt
if rg -q 'NLContextualEmbedding|NLLanguageEnglish|objc_|OBJC_CLASS|NaturalLanguage|swift_' \
    target/framework-c-ios-natural-language-status-host-undefined.txt; then
    echo "Natural Language, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi

for language in c cpp; do
    case "$language" in
        c) binary=target/framework-c-ios-natural-language-status-c-host ; expected=libSystem.B.dylib ;;
        cpp) binary=target/framework-c-ios-natural-language-status-cpp-host ; expected=libSystem.B.dylib ;;
    esac
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-natural-language-status-$language-host-libraries.txt"
    printf '%s\n' "$expected" | LC_ALL=C sort \
        > "target/framework-c-ios-natural-language-status-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-natural-language-status-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-natural-language-status-$language-host-libraries.txt"
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=17.0
            sdk=iphoneos
            clang_target=arm64-apple-ios17.0
            rust_min_flag=-miphoneos-version-min=17.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=17.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios17.0-simulator
            rust_min_flag=-mios-simulator-version-min=17.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-natural-language-status \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-natural-language-status-c.c; standard=c11; cxx_flag= ;;
            cpp) compiler=clang++; source=target/framework-c-ios-natural-language-status-cpp.cpp; standard=c++17; cxx_flag=-nostdlib++ ;;
        esac
        binary="target/framework-c-ios-natural-language-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework NaturalLanguage -framework Foundation -lobjc $cxx_flag -o "$binary"
        libraries="target/framework-c-ios-natural-language-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_natural_language_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-natural-language-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-natural-language-status-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-natural-language-status-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        rg -q '_NLLanguageEnglish' "$symbols"
        strings_file="target/framework-c-ios-natural-language-status-$language-$target-strings.txt"
        strings "$binary" > "$strings_file"
        rg -q 'NLContextualEmbedding' "$strings_file"
        rg -q 'contextualEmbeddingWithLanguage' "$strings_file"
        rg -q 'hasAvailableAssets' "$strings_file"
        if rg -qi 'requestEmbeddingAssets|loadWithError|embeddingResultForString|requestAssets|swift_' \
            "$symbols" "$strings_file"; then
            echo "asset request, model load/text API, or Swift behavior leaked into $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-natural-language-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports: %s; minos %s\n' \
            "$target" "$language" "$(tr '\n' ' ' < "$libraries")" "$actual_deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_natural_language_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-natural-language-status-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-natural-language-status-expected-symbols.txt \
        "target/framework-c-ios-natural-language-status-$target-archive-symbols.txt"
done

printf 'F29 link-import gate passed; C/C++ consumers and probes were not executed\n'
