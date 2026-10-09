#ifndef FRAMEWORK_IOS_GAME_STATUS_H
#define FRAMEWORK_IOS_GAME_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature `ios-game-status`.
 * It exposes only B55's synchronous point-in-time read of
 * `GKLocalPlayer.localPlayer.isAuthenticated`. The API floor is iOS 4.1; F33's
 * Release link probes use minos 10.0 for device and 14.0 for Simulator. Those
 * are link settings, not the API floor.
 *
 * `out_authenticated` is required and writable for one byte. The API checks
 * only nullness: any non-null pointer must address valid, properly aligned
 * writable memory for the synchronous call. The caller must prevent
 * unsynchronized concurrent access to that byte. The output is zero before
 * platform handling and the pointer is not retained. A null pointer returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. On iOS, success returns
 * FRAMEWORK_STATUS_OK and writes exactly zero or one. A valid non-iOS call
 * returns FRAMEWORK_STATUS_UNSUPPORTED with zero output. A future unrecognized
 * portable status returns FRAMEWORK_STATUS_UNAVAILABLE with zero output. A
 * caught Rust panic returns FRAMEWORK_STATUS_PANIC with zero output.
 *
 * False is a current snapshot only; it can include an offline temporary player
 * or an uninitialized Game Center service and does not prove that no account
 * exists. This read does not initialize authentication, install an auth
 * handler, present UI, observe status changes, or expose player identity/data.
 * A configured Game Center app still needs the signed
 * `com.apple.developer.game-center` entitlement. No usage-description key is
 * needed for this read. No main-thread or thread-safety promise is added.
 */
FrameworkStatus framework_ios_game_status_is_local_player_authenticated(
    uint8_t *out_authenticated);

#ifdef __cplusplus
}
#endif

#endif
