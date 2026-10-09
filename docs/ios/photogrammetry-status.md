# iOS Photogrammetry Hardware Status

`ios-photogrammetry-status` exposes two narrow iOS-only RealityFoundation snapshots:

```rust
let supported = ios_photogrammetry_status::photogrammetry_is_supported()?;
let limits = ios_photogrammetry_status::photogrammetry_session_limits()?;
let max_dimension = limits.maximum_input_image_dimension;
let max_images = limits.maximum_number_of_input_images;
```

`Ok(true)` from `photogrammetry_is_supported` means `PhotogrammetrySession.isSupported` reports that the current hardware supports Object Capture processing. `Ok(false)` is the native unsupported-hardware result. The limits function returns `PhotogrammetrySession.Limits` as a Rust-owned `PhotogrammetrySessionLimits`; both fields are signed `i64` values that preserve the native Swift `Int` values on the supported 64-bit iOS targets.

Apple defines `maximum_input_image_dimension` as the maximum allowed input image width or height. Larger images are ignored and produce an `.invalidSample` message. `maximum_number_of_input_images` is the maximum number of images or samples usable for reconstruction; excess images or samples are ignored and produce `.invalidSample`. These are device-specific hardware limits, not a promise that reconstruction will succeed or produce a given quality.

Both APIs are available from iOS 17.0. The package calls their synchronous getters through compiler-derived `swiftcall` C thunks and links `RealityFoundation`. The resilient `PhotogrammetrySession.Limits` value uses metadata-provided size and alignment, and its value-witness destroy function; the bridge does not copy or infer its struct layout. Focused static gates check the Swift compiler's LLVM lowering for device and Simulator, C thunk ABI matches, linked framework/symbol imports, and the iOS 17.0 minimum without running the example.

These snapshots do not create a `PhotogrammetrySession`, read or process images, start reconstruction, capture camera data, present `ObjectCaptureView`, or guarantee that a particular image set can produce a model. They do not expose general RealityKit scene/entity/render support or prove ARKit, camera, or app readiness. `RealityFoundation` and RealityKit remain Swift-facing beyond these snapshots.

Primary API docs: [PhotogrammetrySession](https://developer.apple.com/documentation/realitykit/photogrammetrysession), [PhotogrammetrySession.Limits](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct), [maximumInputImageDimension](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct/maximuminputimagedimension), and [maximumNumberOfInputImages](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct/maximumnumberofinputimages)

The local audit toolchain is Xcode 26.6 build `17F113` with iOS SDK 26.5. The repository's Xcode 27.x baseline caveat remains.
