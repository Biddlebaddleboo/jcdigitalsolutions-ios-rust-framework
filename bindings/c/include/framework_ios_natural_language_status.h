#ifndef FRAMEWORK_IOS_NATURAL_LANGUAGE_STATUS_H
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkIosNaturalLanguageAssetStatus;
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE UINT32_C(0)
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL UINT32_C(2)
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE UINT32_C(3)
#define FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE UINT32_C(4)

/*
 * Opt in with framework-c-api's `ios-natural-language-status` Cargo feature. On iOS this calls
 * B59's English contextual-model asset query. The `NLContextualEmbedding` API floor is iOS 17.0;
 * F29 device and Simulator link probes use their measured minos, recorded in PLAN_BINDINGS_F29.md.
 *
 * On FRAMEWORK_STATUS_OK, `out_status` contains exactly one fixed-width F29 status code:
 * UNAVAILABLE (0) below iOS 17.0, LANGUAGE_UNAVAILABLE (1) if `NLLanguageEnglish` is absent,
 * NO_MODEL (2) if the factory returns no model, ASSETS_NOT_AVAILABLE (3), or ASSETS_AVAILABLE (4).
 * The output value is a framework-owned mapping of B59's five Rust enum variants, not a native
 * NaturalLanguage enum or an unknown raw value. Read it only when FRAMEWORK_STATUS_OK is returned.
 *
 * The query creates a temporary `NLContextualEmbedding` object, reads only `hasAvailableAssets`,
 * and releases the object before return. It does not load a model, accept text, compute vectors,
 * request/download assets, or claim model-load or vector success. It does not use permission, UI,
 * or entitlement APIs.
 *
 * `out_status` is set to zero before platform handling. A null output pointer returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. A valid non-iOS call returns
 * FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught Rust panic returns FRAMEWORK_STATUS_PANIC
 * with zero output. A non-null `out_status` must address valid, properly aligned, writable `uint32_t`
 * storage for the full synchronous call. The caller must prevent unsynchronized concurrent access.
 * The output pointer is caller-owned and not retained; the API checks only nullness and does not
 * retain the output pointer. Calls are synchronous
 * on the caller's thread; no main-thread rule or broader
 * thread-safety promise is added.
 */
FrameworkStatus framework_ios_natural_language_english_contextual_embedding_assets(
    FrameworkIosNaturalLanguageAssetStatus *out_status);

#ifdef __cplusplus
}
#endif

#endif
