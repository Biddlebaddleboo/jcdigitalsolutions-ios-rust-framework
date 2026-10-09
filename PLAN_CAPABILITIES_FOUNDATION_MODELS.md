# PLAN_CAPABILITIES_FOUNDATION_MODELS.md — D69: Foundation Models C-surface feasibility

## Objective

Audit row `067-ml-vision-language-foundation-models-residual-through-c-where-available` for a public C or Objective-C implementation path. Do not assume model execution or an undocumented Swift ABI. This is an audit only; no manifest, crate, source, CI, or global plan change is in scope

## Status

No public C or Objective-C Foundation Models interface is present in the inspected iOS SDK or generated Rust binding cache. Keep row 067 unsupported (`X`) until an explicit supported Swift-to-C boundary is separately approved and validated. No Foundation Models implementation or portable contract is added by D69

## Smallest candidate and blocker

The narrowest meaningful status-only candidate is a point-in-time snapshot of `SystemLanguageModel.default.isAvailable` or `.availability`. Apple defines `.available` as the system being ready for requests and supplies unavailable reasons (`deviceNotEligible`, `appleIntelligenceNotEnabled`, `modelNotReady`). Such a snapshot would not start a session, run inference, or prove a later request succeeds

That property is a Swift API, not a C or Objective-C entry point. Rust cannot call it through the existing C surface without a Swift wrapper or a deliberately supported Swift ABI layer. The SDK exports Swift-mangled symbols, not a documented C function for the status query; manually calling those symbols would assume Swift calling convention, type representation, and ownership rules. No smallest direct Rust-accessible slice is therefore supportable under the requested constraints

If this capability is prioritized later, first establish a supported wrapper boundary that exports only fixed C scalar status values and maps the Apple enum explicitly. Keep wrapper/compiler ABI, error mapping, weak availability, device architecture, and link behavior as acceptance questions. Do not add a Swift shim or call mangled symbols as part of this audit

## Local SDK and binding evidence

Inspection used Xcode 26.6, iOS and iOS Simulator SDK 26.5, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- The iPhoneOS SDK contains `FoundationModels.framework/Modules/FoundationModels.swiftmodule/arm64e-apple-ios.swiftinterface`, its `.swiftdoc`, and `FoundationModels.tbd`; it contains no framework `Headers` directory, C header, or module map
- The public interface marks the framework declarations available from iOS 26.0 and unavailable on tvOS/watchOS. The alternate `Generable` overload with `representNilExplicitlyInGeneratedContent` is iOS 26.4. `LanguageModelSession.respond` is async/throws and its structured forms use Swift `Generable`, `GenerationSchema`, and generated-content values
- `SystemLanguageModel` is declared as `final public class SystemLanguageModel : Swift.Sendable`; its `default`, `isAvailable`, and `availability` members are Swift declarations. It is not declared `@objc`; the interface's `@objc deinit` does not make these APIs callable through Objective-C
- The iPhoneOS stub lists `arm64e-ios` and `swift-abi-version: 7`; its Foundation Models exports use mangled Swift names. The Simulator stub lists `arm64-ios-simulator` and `x86_64-ios-simulator`. This is metadata only; no link or runtime probe was performed, and the device architecture mismatch against the repository's common `aarch64-apple-ios` target remains unverified
- The local Cargo registry contains no `objc2-foundation-models` crate or generated Foundation Models bindings. Existing `objc2` bindings cover Objective-C frameworks and do not provide this Swift API
- The installed SDK is Xcode 26.6 / iOS 26.5, below the repository's Xcode 27.x baseline. No minimum OS below the public API floor iOS 26.0 is implied

## Apple primary documentation

- [Foundation Models framework](https://developer.apple.com/documentation/foundationmodels)
- [SystemLanguageModel](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel)
- [SystemLanguageModel availability](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel/availability-swift.property)
- [SystemLanguageModel availability cases](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel/availability-swift.enum)
- [LanguageModelSession](https://developer.apple.com/documentation/foundationmodels/languagemodelsession)
- [Apple Developer: Foundation Models is a native Swift API](https://developer.apple.com/machine-learning/whats-new/)

## Limits and next step

- Do not infer eligibility, Apple Intelligence enrollment, model readiness, successful inference, or continued availability from a compile/link check or a prior snapshot
- Do not populate entitlement, plist, permission, device-only, or parity fields from this audit; they remain unverified for any eventual wrapper-backed implementation
- Do not add model sessions, prompts, generation, streaming, tools, macros, or response values through C
- Next step: leave row 067 as `X`; if scope changes, create a separate Swift-ABI/C-wrapper feasibility task and first prove a supported scalar C export plus device/Simulator linkability on intended architectures
- No tests, builds, link probes, or runtime calls were run
