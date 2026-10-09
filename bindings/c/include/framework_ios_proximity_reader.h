#ifndef FRAMEWORK_IOS_PROXIMITY_READER_H
#define FRAMEWORK_IOS_PROXIMITY_READER_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint8_t FrameworkIosProximityReaderBoolean;

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-proximity-reader`. It queries Apple's public
 * PaymentCardReader.isSupported property and reports only whether the current
 * iPhone model supports Tap to Pay on iPhone. The API floor and linked target
 * minimum are iOS 15.4 for both device and Simulator.
 *
 * `out_supported` must address valid, properly aligned writable uint8_t memory
 * for the full synchronous call. The caller must prevent unsynchronized access
 * to the output. The wrapper does not retain the output address. It initializes
 * the output to zero before the query. On FRAMEWORK_STATUS_OK, it is exactly
 * zero or one. A null output
 * returns FRAMEWORK_STATUS_INVALID_ARGUMENT. On non-iOS targets, a valid call
 * returns FRAMEWORK_STATUS_UNSUPPORTED with output zero. If the Rust backend
 * reports no value on an iOS target, the wrapper returns
 * FRAMEWORK_STATUS_UNAVAILABLE with output zero. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC with output zero.
 *
 * This is not an operating-system compatibility, entitlement, merchant,
 * region, account, payment service provider, reader, NFC, or transaction
 * readiness check. It does not create a reader, start a session, access NFC,
 * show UI, or process a payment. The API adds no main-thread or thread-safety
 * guarantee. The linked probes are build-only and are not executed.
 */
FrameworkStatus framework_ios_proximity_reader_tap_to_pay_device_model_supported(
    FrameworkIosProximityReaderBoolean *out_supported);

#ifdef __cplusplus
}
#endif

#endif
