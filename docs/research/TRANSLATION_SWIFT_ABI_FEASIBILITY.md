# Translation Framework Swift ABI Feasibility

Research date: 2026-10-07

## Conclusion

Apple's Translation framework is a useful end-to-end Swift-ABI proof. The current roadmap in
`PLAN_SWIFT_ABI.md` schedules Translation before full StoreKit 2 purchase flows. D58/B63 adds a
narrow synchronous `AppStore.canMakePayments` adapter; its link probes were not executed, and it
does not complete the StoreKit 2 value, purchase, or async path discussed in this research.

It is genuinely Swift-facing, but the core non-UI capability is much simpler than App Intents or SwiftUI:

- `TranslationSession` is a class;
- a session can now be initialized directly for already-installed languages;
- single-string translation is an async throwing method;
- results are plain Swift value types containing strings/languages;
- batch streaming adds `AsyncSequence`, but it is optional for an initial implementation.

That makes Translation a good test of class references + Swift strings/value results + async/throws without compiler registration metadata.

## Public API shape

Apple documents:

```swift
class TranslationSession
```

A non-UI caller can directly initialize a session using:

```swift
convenience init(
    installedSource: Locale.Language,
    target: Locale.Language?
)
```

Apple specifically documents this route for contexts with no UI.

The simplest useful translation method is:

```swift
func translate(String) async throws -> TranslationSession.Response
```

The response is:

```swift
struct TranslationSession.Response
```

and contains, among other properties:

- `sourceLanguage: Locale.Language`
- `targetLanguage: Locale.Language`
- `sourceText: String`
- `targetText: String`
- optional client identifier

References:
https://developer.apple.com/documentation/translation/translationsession
https://developer.apple.com/documentation/translation/translationsession/response

## Why this is preferable to SwiftUI translation APIs

Apple also exposes SwiftUI helpers such as:

- `translationPresentation(...)`
- `translationTask(...)`

The framework should not target those first.

The direct `TranslationSession` initializer means translation capability is not inherently a SwiftUI problem. A UIKit/Rust application can keep its own UI and call the session directly once Swift ABI support exists.

## Minimum ABI surface for initial support

An initial Rust Translation implementation appears to require:

1. resolve/construct `Locale.Language`;
2. call a Swift class initializer;
3. retain/release the Swift class reference correctly;
4. construct/borrow Swift `String`;
5. invoke an async throwing instance method;
6. receive a Swift struct result;
7. read or call accessors for result fields;
8. convert/borrow the target string;
9. destroy returned values correctly.

Compared with StoreKit product lookup, this avoids an immediately generic collection input.

That makes it a valuable validation that the ABI layer is not overfitted specifically to StoreKit generics.

## Class representation question

Do not assume a Swift class is interchangeable with an Objective-C `id` merely because Swift classes on Apple platforms participate in parts of the Objective-C runtime.

Before implementation, inspect:

- whether `TranslationSession` is Objective-C-compatible at the runtime metadata level;
- whether the public class methods are exposed only with Swift calling convention;
- retain/release convention for returned session references;
- whether ordinary `swift_retain`/`swift_release` runtime entrypoints are required rather than Objective-C retain/release.

The Swift ABI metadata docs note that Apple-platform Swift class metadata interoperates with the Objective-C runtime at a foundational level, but that does not make every Swift-only member an Objective-C selector.

## Locale.Language

The API uses Swift Foundation's `Locale.Language`, not an Objective-C `NSLocale` string parameter.

Research should determine the cheapest supported construction path:

- direct Swift initializer from language identifier;
- public Foundation bridge from an existing locale/language representation;
- stable value-metadata construction.

Do not hard-code value layout.

## Async/throws

The single-string translation operation provides a particularly clean async test:

```text
Rust Future/callback state
  -> swiftcall thunk
  -> TranslationSession.translate
  -> Apple's Swift concurrency runtime
  -> completion/resume thunk
  -> Rust result
```

No custom Swift executor should be introduced.

Cancellation needs explicit study because `TranslationSession` itself also provides:

```swift
func cancel()
```

A future Rust API may map cancellation or Drop behavior to session cancellation where semantically appropriate, but must not cancel unrelated concurrent work sharing the same session.

## Download/preparation behavior

Apple exposes:

```swift
func prepareTranslation() async throws
```

to request permission/download preparation.

The directly initialized installed-language session throws if the requested language pair is not already installed.

A robust Rust API should separate:

- checking availability;
- preparing/downloading languages;
- installed-only fast path;
- actual translation.

Do not hide possible user interaction behind a supposedly pure background call.

## Batch APIs

Apple exposes two useful batch shapes.

All-at-once:

```swift
func translations(
    from: [TranslationSession.Request]
) async throws -> [TranslationSession.Response]
```

Streaming:

```swift
func translate(
    batch: [TranslationSession.Request]
) -> TranslationSession.BatchResponse
```

`BatchResponse` conforms to `AsyncSequence`.

### Recommendation

Implement in this order:

1. single-string translation;
2. all-at-once array translation;
3. streaming `BatchResponse`.

This lets StoreKit and Translation share reusable ABI primitives for:

- Swift String;
- Array;
- async/throws;
- result storage/destruction;

before implementing general AsyncSequence iteration.

References:
https://developer.apple.com/documentation/translation/translationsession/translations(from:)
https://developer.apple.com/documentation/translation/translationsession/translate(batch:)
https://developer.apple.com/documentation/translation/translationsession/batchresponse

## Runtime overhead target

A successful implementation should not add a language bridge runtime.

Desired path:

```text
Rust
 -> thin ABI thunk
 -> TranslationSession
 -> Apple on-device Translation implementation
```

The translation/model work itself will dominate the handful of native ABI instructions needed to cross into the framework.

Avoid:

- serializing the source text into JSON;
- an Objective-C proxy object solely to call Swift;
- copying text more than required by Swift String ownership/construction;
- a dedicated worker process;
- a custom task executor.

## Why Translation is architecturally useful

StoreKit exercises highly generic StoreKit value types and transaction streams.

Translation tests a different but simpler surface:

- public Swift class;
- direct non-UI initialization;
- Swift Foundation value types;
- async throwing methods;
- simple result struct;
- optional later AsyncSequence.

If the same internal Swift ABI primitives can support both frameworks cleanly, confidence increases that the design is reusable rather than StoreKit-specific.

## Feasibility experiments

After the common Swift-call proof from the StoreKit research:

### T1 — session construction

- identify exact ABI-public initializer symbol from the current SDK;
- construct supported `Locale.Language` values;
- initialize an installed-language session;
- retain/destroy correctly.

### T2 — synchronous property access

Before async translation, read:

- `isReady`;
- `canRequestDownloads`;
- source/target language properties.

Use this to validate Swift class instance calling without async complexity.

### T3 — single translation

Call `translate(String)` and return `targetText` to Rust.

Validate:

- correct Unicode;
- empty string;
- long string;
- error path;
- repeated calls;
- teardown.

### T4 — cancellation

Start a sufficiently long operation, invoke `cancel()`, and characterize:

- error/result delivered;
- whether session remains reusable;
- callback/resume behavior;
- Rust state lifetime.

### T5 — all-at-once batch

Construct `[Request]`, call `translations(from:)`, iterate `[Response]`.

### T6 — streaming batch

Only after the previous tests, bridge `BatchResponse.AsyncIterator` into a Rust stream/callback API.

## App Store compliance

Only documented public Translation framework declarations should be called.

Do not use private language-model services or daemon protocols as a shortcut.

Using the public Swift ABI to invoke a documented public Translation API is the intended research direction; final shipping validation still needs an Xcode archive/App Store-compatible link test.

## Preliminary priority

**P1, once the shared Layer-1 primitives and a supported C6 task-entry contract exist; ahead of full StoreKit 2 purchase flows.**

The current roadmap schedules Translation as the first end-to-end Swift value/async framework proof because it is useful, non-UI-capable, and exercises a smaller ABI surface than App Intents.
