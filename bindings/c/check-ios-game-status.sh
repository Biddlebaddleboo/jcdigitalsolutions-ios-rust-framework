#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_game_status.rs
header=bindings/c/include/framework_ios_game_status.h

for tool in cargo clang clang++ jq python3 rg rustfmt; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F33 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-game-status.sh
sh -n bindings/c/check-ios-game-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_game_status_is_local_player_authenticated
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'if out_authenticated\.is_null\(\)' "$source"
rg -q 'out_authenticated\.write\(0\)' "$source"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'FrameworkStatus::UNAVAILABLE' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'LocalPlayer::new\(IosGameCenterBackend::new\(\)\)' "$source"
rg -q 'player\.authentication_status\(\)' "$source"
rg -q 'LocalPlayerAuthenticationStatus::Authenticated' "$source"
rg -q 'LocalPlayerAuthenticationStatus::NotAuthenticated' "$source"
rg -q 'valid, properly aligned writable memory' "$source" "$header"
rg -q 'synchronous call' "$source" "$header"
rg -q 'unsynchronized' "$source" "$header"
rg -q 'not retain the' "$source"
rg -q 'pointer is not retained' "$header"
rg -q 'com.apple.developer.game-center' "$header"
rg -q 'does not initialize authentication' "$header"
if rg -n 'authenticateHandler|authenticateWithCompletionHandler|GKPlayerAuthenticationDidChangeNotificationName|GKGameCenterViewController|playerID|teamPlayerID|GKPlayer::' \
    "$source"; then
    echo "out-of-scope Game Center auth, UI, or identity API found in F33 source" >&2
    exit 1
fi

jq -e '
    .optional_capabilities.ios_game_status as $game
    | $game.cargo_feature == "ios-game-status"
    and $game.header == "framework_ios_game_status.h"
    and $game.dependencies == ["framework-game", "ios-game"]
    and $game.symbols == ["framework_ios_game_status_is_local_player_authenticated"]
    and $game.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $game.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($game.api | contains("iOS 4.1 API floor"))
    and ($game.status_mapping.authenticated | contains("output one"))
    and ($game.status_mapping.not_authenticated | contains("output zero"))
    and ($game.status_mapping.unknown_future_portable_status | contains("UNAVAILABLE"))
    and ($game.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($game.status_mapping.panic | contains("PANIC"))
    and ($game.ownership.output | contains("valid, properly aligned writable uint8_t"))
    and ($game.ownership.output | contains("full synchronous call"))
    and ($game.ownership.output | contains("caller prevents unsynchronized access"))
    and ($game.ownership.output | contains("not retained"))
' bindings/c/abi-manifest.json > /dev/null

for file in PLAN_BINDINGS_F33.md docs/bindings/ios-game-status.md "$source" "$header" \
    bindings/c/check-ios-game-status.sh bindings/c/check-ios-game-status-link.sh; do
    if rg -n '[[:blank:]]+$' "$file"; then
        echo "F33 file has trailing whitespace: $file" >&2
        exit 1
    fi
done

cargo tree --locked --no-default-features -p framework-c-api --target aarch64-apple-ios \
    > target/framework-c-ios-game-status-default-tree.txt
if rg -q 'framework-game v|ios-game v|objc2-game-kit v|GameKit.framework' \
    target/framework-c-ios-game-status-default-tree.txt; then
    echo "F33 dependencies leaked into the default iOS graph" >&2
    exit 1
fi
cargo tree --locked --no-default-features --features ios-game-status \
    -p framework-c-api --target x86_64-apple-darwin \
    > target/framework-c-ios-game-status-host-tree.txt
if rg -q 'framework-game v|ios-game v|objc2-game-kit v|GameKit.framework' \
    target/framework-c-ios-game-status-host-tree.txt; then
    echo "F33 dependencies leaked into the host feature graph" >&2
    exit 1
fi
cargo tree --locked --no-default-features --features ios-game-status \
    -p framework-c-api --target aarch64-apple-ios \
    > target/framework-c-ios-game-status-ios-tree.txt
rg -q 'framework-game v' target/framework-c-ios-game-status-ios-tree.txt
rg -q 'ios-game v' target/framework-c-ios-game-status-ios-tree.txt

cat > target/framework-c-ios-game-status-c.c <<'FIXTURE_C'
#include <framework_ios_game_status.h>
int main(void) {
    uint8_t authenticated = 0;
    return (int)framework_ios_game_status_is_local_player_authenticated(&authenticated);
}
FIXTURE_C
cat > target/framework-c-ios-game-status-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_game_status.h>
int main() {
    uint8_t authenticated = 0;
    return static_cast<int>(framework_ios_game_status_is_local_player_authenticated(&authenticated));
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-game-status-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-game-status-cpp.cpp

printf 'F33 static checks passed; no tests, consumers, or probes were executed\n'
