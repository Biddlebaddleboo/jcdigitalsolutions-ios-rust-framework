#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_speech_status.rs
header=bindings/c/include/framework_ios_speech_status.h
guide=docs/bindings/ios-speech-status.md
plan=PLAN_BINDINGS_F28.md

for tool in cargo clang clang++ diff jq python3 rg rustfmt sh; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F28 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-speech-status.sh
sh -n bindings/c/check-ios-speech-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_speech_status_authorization_status
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNAVAILABLE' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_raw_status.write\(0\)' "$source"
rg -q 'valid, writable, properly aligned `int64_t` storage' "$source"
rg -q 'for this synchronous call' "$source"
rg -q 'prevent unsynchronized concurrent access' "$source"
rg -q 'does not retain the pointer' "$source"
rg -q 'ios_speech_status::authorization_status\(\)' "$source"
if rg -n 'requestAuthorization|recognitionTask|SFSpeechAudioBufferRecognitionRequest|SFSpeechURLRecognitionRequest' "$source"; then
    echo "Speech operation or permission-request API leaked into F28 source" >&2
    exit 1
fi
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'Unknown native' "$header"
rg -q 'zero before platform' "$header"
rg -q 'valid aligned writable storage' "$header"
rg -q 'pointer is caller-owned and not retained' "$header"
rg -q 'prevent' "$header"
rg -q 'unsynchronized concurrent access' "$header"
for contract_file in "$guide" "$plan"; do
    rg -q 'valid' "$contract_file"
    rg -q 'writable' "$contract_file"
    rg -q 'aligned' "$contract_file"
    rg -q 'synchronous' "$contract_file"
    rg -q 'call' "$contract_file"
    rg -q 'unsynchronized concurrent access' "$contract_file"
    rg -q 'retain' "$contract_file"
done
rg -q 'zero before platform' "$guide"
rg -q 'output remains zero' "$plan"

jq -e '
    .optional_capabilities.ios_speech_status as $speech
    | $speech.cargo_feature == "ios-speech-status"
    and $speech.header == "framework_ios_speech_status.h"
    and $speech.symbols == ["framework_ios_speech_status_authorization_status"]
    and ($speech.authorization_raw_values | keys | sort) == [
        "FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED",
        "FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED",
        "FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED",
        "FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED"
    ]
    and ($speech.authorization_raw_values | to_entries | map(.value) | sort) == [0, 1, 2, 3]
    and $speech.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $speech.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($speech.api | contains("iOS 10.0 API floor"))
    and ($speech.status_mapping.success | contains("unchanged"))
    and ($speech.status_mapping.unavailable | contains("UNAVAILABLE"))
    and ($speech.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($speech.status_mapping.panic | contains("PANIC"))
    and ($speech.status_mapping.unknown_native_value | contains("unchanged"))
    and ($speech.ownership.output | contains("int64_t"))
    and (($speech.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and ($speech.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "Speech", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($speech.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "Speech", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

if rg -n '[[:blank:]]+$' \
    "$plan" PLAN_VALIDATION_C_ABI_SPEECH_STATUS.md \
    "$guide" "$source" "$header" \
    bindings/c/check-ios-speech-status.sh bindings/c/check-ios-speech-status-link.sh; then
    echo "F28 file has trailing whitespace" >&2
    exit 1
fi

cat > target/framework-c-ios-speech-status-c.c <<'FIXTURE_C'
#include <framework_ios_speech_status.h>
_Static_assert(sizeof(FrameworkIosSpeechAuthorizationStatus) == sizeof(int64_t), "raw status width");
_Static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED == 0, "not determined code");
_Static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED == 1, "denied code");
_Static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED == 2, "restricted code");
_Static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED == 3, "authorized code");
int main(void) {
    FrameworkIosSpeechAuthorizationStatus status = INT64_MIN;
    return (int)framework_ios_speech_status_authorization_status(&status);
}
FIXTURE_C

cat > target/framework-c-ios-speech-status-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_speech_status.h>
static_assert(sizeof(FrameworkIosSpeechAuthorizationStatus) == sizeof(int64_t), "raw status width");
static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED == 0, "not determined code");
static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED == 1, "denied code");
static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED == 2, "restricted code");
static_assert(FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED == 3, "authorized code");
int main() {
    FrameworkIosSpeechAuthorizationStatus status = INT64_MIN;
    return static_cast<int>(framework_ios_speech_status_authorization_status(&status));
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-speech-status-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-speech-status-cpp.cpp

expected_symbols=target/framework-c-ios-speech-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_speech_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"
rg -o 'framework_ios_speech_status_[A-Za-z0-9_]+' "$header" | LC_ALL=C sort -u \
    > target/framework-c-ios-speech-status-header-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-speech-status-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-speech-status-default-tree.txt
if rg -q 'ios-speech-status v|objc2-speech v|Speech.framework' \
    target/framework-c-ios-speech-status-default-tree.txt; then
    echo "Speech dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-speech-status \
    > target/framework-c-ios-speech-status-ios-tree.txt
rg -q 'ios-speech-status v' target/framework-c-ios-speech-status-ios-tree.txt
rg -q 'objc2-speech feature "SFSpeechRecognizer"' \
    target/framework-c-ios-speech-status-ios-tree.txt
if rg -q 'objc2-speech feature "block2"|objc2-speech feature "default"' \
    target/framework-c-ios-speech-status-ios-tree.txt; then
    echo "Speech block/default features leaked into F28" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-speech-status \
    > target/framework-c-ios-speech-status-host-tree.txt
if rg -q 'ios-speech-status v|objc2-speech v|Speech.framework' \
    target/framework-c-ios-speech-status-host-tree.txt; then
    echo "iOS Speech dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-speech-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-speech-status -- -D warnings
cargo doc --locked -p framework-c-api --no-deps
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-speech-status --target aarch64-apple-ios
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-speech-status --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-speech-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-speech-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-c-api --no-deps --target aarch64-apple-ios

printf 'F28 static/build gate passed; no tests, C/C++ consumers, or link probes were built or run\n'
