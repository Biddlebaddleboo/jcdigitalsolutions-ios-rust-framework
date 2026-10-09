#ifndef FRAMEWORK_IOS_MODELIO_STATUS_H
#define FRAMEWORK_IOS_MODELIO_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-modelio-status`. It asks whether ModelIO can read asset files with the
 * supplied extension by calling +[MDLAsset canImportFileExtension:], whose
 * class API floor is iOS 9.0. The linked Release probes use iOS deployment
 * minima 10.0 for device and 14.0 for Simulator; these probe minima are
 * distinct from the API floor.
 *
 * `extension` is a borrowed UTF-8 FrameworkStr and is forwarded without
 * normalization. Null data is valid only with zero length; empty extension
 * text is passed through. The bytes must remain readable and immutable for
 * this synchronous call. `out_supported` must address valid, properly aligned
 * writable memory for one byte for the full synchronous call and must be
 * disjoint from nonempty input. The caller must prevent unsynchronized access
 * to either region. The wrapper checks range arithmetic and input/output
 * overlap, but cannot prove that memory is valid, aligned, writable, or live.
 * It retains neither pointer. The wrapper initializes a structurally
 * valid, disjoint output to zero before UTF-8 or platform handling. A null
 * output, malformed span metadata, or overlap returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without writing output. Invalid UTF-8
 * returns FRAMEWORK_STATUS_INVALID_ARGUMENT
 * with zero output.
 *
 * FRAMEWORK_STATUS_OK writes exactly zero or one. An unknown or unsupported
 * extension is a successful false result. A valid non-iOS call returns
 * FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC with zero output. The API adds no main-thread rule
 * or thread-safety guarantee. It does not create an asset, access a URL or
 * file data, or claim that a specific asset parses, renders, or uses a GPU.
 */
FrameworkStatus framework_ios_modelio_can_import_file_extension(
    FrameworkStr extension,
    uint8_t *out_supported);

#ifdef __cplusplus
}
#endif

#endif
