#ifndef FRAMEWORK_IOS_STOREKIT2_STATUS_H
#define FRAMEWORK_IOS_STOREKIT2_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-storekit2-status`. It exposes only B63's synchronous
 * `AppStore.canMakePayments` Boolean. The API and symbol availability floor is
 * iOS 15.0. B63 uses weak imports for StoreKit.framework and the Swift symbol;
 * its Rust link probes use minos 10.0 for device and 14.0 for Simulator. Those
 * are link settings, not the API floor. If the weak symbol is absent, B63
 * returns false without calling it. That fallback is compiler/link evidence
 * only and has not been runtime-tested below iOS 15.0.
 *
 * `out_can_make_payments` is required and writable for one byte. The API checks
 * only nullness: any non-null pointer must actually address valid, properly
 * aligned writable memory for the synchronous call. The caller must prevent
 * unsynchronized concurrent access to that byte. The output is initialized to
 * zero before platform handling and the pointer is not retained. A null pointer
 * returns FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. On iOS, success
 * returns FRAMEWORK_STATUS_OK and writes exactly zero or one from B63. A valid
 * non-iOS call returns FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught
 * Rust panic returns FRAMEWORK_STATUS_PANIC with zero output.
 *
 * `true` means StoreKit reports that the person can authorize purchases;
 * `false` may reflect purchase restrictions or the absent weak symbol. This is
 * not product availability, account identity, entitlement state, or transaction
 * success. No product, transaction, purchase, restore, payment UI, account, or
 * network API is called. This API adds no thread or queue guarantee.
 */
FrameworkStatus framework_ios_storekit2_status_can_make_payments(
    uint8_t *out_can_make_payments);

#ifdef __cplusplus
}
#endif

#endif
