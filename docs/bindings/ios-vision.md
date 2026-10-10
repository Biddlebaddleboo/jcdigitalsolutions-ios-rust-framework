# iOS Vision C ABI

F17 adds the opt-in `ios-vision` feature for a synchronous query of Vision text-recognition revision membership

`framework_ios_vision_text_recognition_revision_is_supported` checks whether the current iOS runtime lists the supplied `uint32_t` revision in `VNRecognizeTextRequest.supportedRevisions`. It returns `FRAMEWORK_STATUS_OK` and writes exactly 0 or 1. Any revision value is valid; an unknown revision returns OK with zero

The query is available from iOS 13.0. An earlier iOS runtime returns `FRAMEWORK_STATUS_UNAVAILABLE`; a valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED`. A null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`. The output is initialized to zero before the query. A caught Rust panic returns `FRAMEWORK_STATUS_PANIC`

The output address must point to valid, properly aligned writable `uint8_t` memory for the full synchronous call. The caller must prevent unsynchronized access to the output. The wrapper does not prove memory validity or retain the address. No callback, handle, object, owned buffer, or native error crosses C. The API adds no main-thread rule or thread-safety guarantee

This status does not create `VNRecognizeTextRequest`, create a request handler, read an image, run recognition, check language support, or claim model readiness or recognition success. See [D52](../../PLAN_CAPABILITIES_VISION.md), [B57](../../PLAN_IOS_VISION.md), and [F17](../../PLAN_BINDINGS_COMPLETED_C_ABI.md)

`sh bindings/c/check-ios-vision.sh` passed the host, device, and Simulator build-only gates. Its C/C++ link probes were not executed
