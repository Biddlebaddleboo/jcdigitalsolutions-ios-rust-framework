#ifndef FRAMEWORK_IOS_EXTENSION_SUPPORT_H
#define FRAMEWORK_IOS_EXTENSION_SUPPORT_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkIosExtensionMetadataError;
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE UINT32_C(0)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH UINT32_C(1)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE UINT32_C(2)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE UINT32_C(3)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY UINT32_C(4)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE UINT32_C(5)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER UINT32_C(6)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE UINT32_C(7)
#define FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER UINT32_C(8)

/*
 * Opt in with framework-c-api's ios-extension-support Cargo feature. On iOS this reads only
 * NSExtension.NSExtensionPointIdentifier from one caller-supplied absolute UTF-8 .appex path.
 * The Foundation API floor is iOS 4.0. The focused Release link probes use device minos 12.0 and
 * Simulator minos 14.0; those probe minima do not raise the API floor.
 *
 * The input is borrowed for this synchronous call. Its bytes must be readable, immutable UTF-8;
 * null data is valid only with zero length. Null outputs, malformed pointer/length metadata, and
 * overlap between input and output slots return FRAMEWORK_STATUS_INVALID_ARGUMENT without a write.
 * With valid disjoint outputs, both slots are zeroed before UTF-8 and platform handling; invalid
 * UTF-8 returns FRAMEWORK_STATUS_INVALID_ARGUMENT with zero outputs. The path shape and bundle
 * metadata errors are returned in out_error while the function returns FRAMEWORK_STATUS_OK.
 *
 * On FRAMEWORK_STATUS_OK and out_error == NONE, out_identifier contains the copied identifier as
 * length-delimited UTF-8 in a FrameworkOwnedBuffer. Destroy that original, unchanged descriptor
 * once with framework_owned_buffer_destroy. On FRAMEWORK_STATUS_OK with a non-NONE error code,
 * out_identifier remains empty. A valid non-iOS call returns FRAMEWORK_STATUS_UNSUPPORTED with
 * both outputs initialized to zero. A caught Rust panic returns FRAMEWORK_STATUS_PANIC with both
 * outputs zero. FRAMEWORK_STATUS_RESOURCE_EXHAUSTED means the Rust-owned result could not be
 * represented by the ABI buffer.
 *
 * Both outputs are required, writable, aligned, mutually disjoint, and disjoint from input. The
 * identifier output must not hold a live framework allocation on entry. The API retains no input,
 * output, Foundation object, callback, or native pointer. It does not load extension code or prove
 * installation, registration, approval, entitlement, launchability, or host compatibility.
 */
FrameworkStatus framework_ios_extension_support_read_extension_point_identifier(
    FrameworkStr bundle_path,
    FrameworkIosExtensionMetadataError *out_error,
    FrameworkOwnedBuffer *out_identifier);

#ifdef __cplusplus
}
#endif

#endif
