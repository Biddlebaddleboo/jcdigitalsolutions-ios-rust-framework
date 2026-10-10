#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

source=bindings/c/src/ios_sign_in_with_apple_status.rs
header=bindings/c/include/framework_ios_sign_in_with_apple_status.h

sh -n bindings/c/check-ios-sign-in-with-apple-status.sh
rustfmt --check --edition 2024 "$source"

source_symbol=$(rg -o 'framework_ios_sign_in_with_apple_credential_state_start' "$source" | sort -u)
header_symbol=$(rg -o 'framework_ios_sign_in_with_apple_credential_state_start' "$header" | sort -u)
if [ "$source_symbol" != "$header_symbol" ]; then
    echo "F21 Rust export and C header symbol do not match" >&2
    exit 1
fi

if rg -n '[[:blank:]]+$' \
    docs/bindings/ios-sign-in-with-apple-status.md \
    "$source" "$header" bindings/c/check-ios-sign-in-with-apple-status.sh; then
    echo "F21 file has trailing whitespace" >&2
    exit 1
fi

cat <<'FIXTURE_C' | clang -std=c11 -Wall -Wextra -Werror -pedantic \
    -I bindings/c/include -x c -fsyntax-only -
#include <framework_ios_sign_in_with_apple_status.h>
static void complete(void *context, FrameworkIosSignInWithAppleCredentialState state, uint8_t had_error) {
    (void)context; (void)state; (void)had_error;
}
FrameworkIosSignInWithAppleCredentialStateCompletion callback = complete;
FrameworkStatus (*start_query)(FrameworkStr, FrameworkIosSignInWithAppleCredentialStateCompletion, void *) = framework_ios_sign_in_with_apple_credential_state_start;
_Static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED == 0, "revoked tag");
_Static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED == 1, "authorized tag");
_Static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND == 2, "not-found tag");
_Static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED == 3, "transferred tag");
FIXTURE_C

cat <<'FIXTURE_CPP' | clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
    -I bindings/c/include -x c++ -fsyntax-only -
#include <framework_ios_sign_in_with_apple_status.h>
static void complete(void *context, FrameworkIosSignInWithAppleCredentialState state, uint8_t had_error) {
    (void)context; (void)state; (void)had_error;
}
FrameworkIosSignInWithAppleCredentialStateCompletion callback = complete;
FrameworkStatus (*start_query)(FrameworkStr, FrameworkIosSignInWithAppleCredentialStateCompletion, void *) = framework_ios_sign_in_with_apple_credential_state_start;
static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED == 0, "revoked tag");
static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED == 1, "authorized tag");
static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND == 2, "not-found tag");
static_assert(FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED == 3, "transferred tag");
FIXTURE_CPP

printf 'F21 static checks complete; no Cargo build, link, test, or probe was run\n'
