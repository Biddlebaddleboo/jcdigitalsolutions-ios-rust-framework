#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

for tool in awk cargo diff grep nm otool sed sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Sign in with Apple C ABI link/import check" >&2
        exit 1
    fi
done

mkdir -p target
c_source=target/framework-c-ios-sign-in-with-apple-status-c.c
cpp_source=target/framework-c-ios-sign-in-with-apple-status-cpp.cpp
cat > "$c_source" <<'FIXTURE_C'
#include <framework_ios_sign_in_with_apple_status.h>

static void complete(
    void *context,
    FrameworkIosSignInWithAppleCredentialState credential_state,
    uint8_t had_error) {
    (void)context;
    (void)credential_state;
    (void)had_error;
}

int main(void) {
    static const uint8_t user_id_bytes[] = "f21-link-only-user-id";
    FrameworkStr user_id = {user_id_bytes, sizeof(user_id_bytes) - 1};
    return (int)framework_ios_sign_in_with_apple_credential_state_start(
        user_id, complete, NULL);
}
FIXTURE_C

cat > "$cpp_source" <<'FIXTURE_CPP'
#include <framework_ios_sign_in_with_apple_status.h>

static void complete(
    void *context,
    FrameworkIosSignInWithAppleCredentialState credential_state,
    uint8_t had_error) {
    (void)context;
    (void)credential_state;
    (void)had_error;
}

int main() {
    static const uint8_t user_id_bytes[] = "f21-link-only-user-id";
    FrameworkStr user_id{user_id_bytes, sizeof(user_id_bytes) - 1};
    return static_cast<int>(framework_ios_sign_in_with_apple_credential_state_start(
        user_id, complete, nullptr));
}
FIXTURE_CPP

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=13.0
            sdk=iphoneos
            clang_target=arm64-apple-ios13.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            ;;
    esac

    target_dir="$root/target/framework-c-ios-sign-in-with-apple-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" \
        CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p framework-c-api --no-default-features \
        --features ios-sign-in-with-apple-status --target "$target"
    archive="$target_dir/$target/release/libframework_c_api.a"
    if [ ! -f "$archive" ]; then
        echo "missing C ABI archive: $archive" >&2
        exit 1
    fi

    for language in c cpp; do
        case "$language" in
            c)
                compiler=clang
                source="$c_source"
                standard=c11
                expected_imports="$target_dir/c-imports-expected.txt"
                printf '%s\n' AuthenticationServices Foundation libSystem.B.dylib \
                    libobjc.A.dylib | LC_ALL=C sort > "$expected_imports"
                ;;
            cpp)
                compiler=clang++
                source="$cpp_source"
                standard=c++17
                expected_imports="$target_dir/cpp-imports-expected.txt"
                printf '%s\n' AuthenticationServices Foundation libSystem.B.dylib \
                    libc++.1.dylib libobjc.A.dylib | LC_ALL=C sort > "$expected_imports"
                ;;
        esac

        binary="$target_dir/framework-c-sign-in-with-apple-status-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" \
                    -std="$standard" -Wall -Wextra -Werror -pedantic \
                    -I bindings/c/include "$source" "$archive" \
                    -framework AuthenticationServices -framework Foundation \
                    -lSystem -lobjc -o "$binary"
                ;;
            cpp)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" \
                    -std="$standard" -Wall -Wextra -Werror -pedantic \
                    -I bindings/c/include "$source" "$archive" \
                    -framework AuthenticationServices -framework Foundation \
                    -lSystem -lobjc -lc++ -o "$binary"
                ;;
        esac

        imports="$target_dir/$language-$target-imports.txt"
        libraries="$target_dir/$language-$target-libraries.txt"
        symbols="$target_dir/$language-$target-undefined.txt"
        text="$target_dir/$language-$target-strings.txt"
        exports="$target_dir/$language-$target-exports.txt"
        build_info="$target_dir/$language-$target-build.txt"

        otool -L "$binary" > "$imports"
        awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
            | LC_ALL=C sort > "$libraries"
        diff -u "$expected_imports" "$libraries"

        nm -u "$binary" 2>/dev/null > "$symbols"
        for symbol in _objc_getClass _objc_msgSend __Block_copy __Block_release \
            __NSConcreteStackBlock; do
            if ! grep -Fq "$symbol" "$symbols"; then
                echo "$binary is missing Objective-C/block import $symbol" >&2
                exit 1
            fi
        done
        strings "$binary" > "$text"
        grep -Fq 'ASAuthorizationAppleIDProvider' "$text"
        grep -Fq 'getCredentialStateForUserID:completion:' "$text"
        if grep -Eiq \
            'swift|ASAuthorizationController|ASAuthorizationAppleIDRequest|ASAuthorizationPlatformPublicKeyCredential|ASAuthorizationWebBrowser' \
            "$symbols" "$text"; then
            echo "unexpected Swift, authorization-flow, or passkey symbol in $binary" >&2
            exit 1
        fi

        nm -g "$binary" 2>/dev/null \
            | awk '{ print $NF }' \
            | grep -E '^_framework_ios_sign_in_with_apple_[[:alnum:]_]+$' \
            | sed 's/^_//' | LC_ALL=C sort -u \
            > "$exports"
        printf '%s\n' framework_ios_sign_in_with_apple_credential_state_start \
            > "$target_dir/exports-expected.txt"
        diff -u "$target_dir/exports-expected.txt" "$exports"

        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk \
            '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
    done

    printf '%s C11/C++17 exact imports, Objective-C selector/block symbols, export, and iOS %s minos verified; consumers were not executed\n' \
        "$target" "$deployment_target"
done

printf 'F21 native C ABI link/import gate complete; no tests or consumer binaries were executed\n'
