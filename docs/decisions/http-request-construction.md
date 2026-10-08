# HTTP Request Construction: No Rust Replacement Candidate

## Decision

Keep Foundation `NSURLRequest` construction and `NSURLSession` as the iOS foreground HTTP path. This review found no bounded Apple behavior that a Rust implementation can replace while preserving the current contract and offering a credible performance-win hypothesis. Do not add a Rust HTTP/URL implementation, parity suite, or benchmark workload from this review. No performance claim is made; representative-device Release measurements have not been run.

This is a defer/no-candidate decision, not evidence that URLSession is faster in every request-construction microbenchmark. The native backend remains the default because it is the system HTTP implementation and there is no proven, semantically equivalent replacement candidate.

## Baseline and code boundary

Review baseline: repository `main` at `51f9d8e` (`docs(plan): scope HTTP replacement review`). The portable contract and iOS backend reviewed are [framework-network](../../crates/framework-network/src/lib.rs) and [ios-network](../../platform/ios/ios-network/src).

| Stage | Existing behavior | Evidence |
| --- | --- | --- |
| Portable validation and storage | `HttpUrl::new` checks scheme, authority presence, and ASCII whitespace/control bytes; it neither canonicalizes nor fully parses the URL. `HttpMethod` and `Header` validate tokens/values. `HttpRequest` borrows its method, URL, header slice, and optional body without hidden allocation. | [`HttpUrl`, `HttpMethod`, `Header`, and `HttpRequest`](../../crates/framework-network/src/lib.rs#L53-L203) |
| Async contract | `HttpBackend::send` returns a concrete, statically selected future; the facade starts/attaches to work on first poll. No executor or transport is prescribed. | [`HttpBackend` and `HttpClient`](../../crates/framework-network/src/lib.rs#L307-L364) |
| Escaping request ownership | The iOS backend copies borrowed method, URL, header names/values, and body into owned Rust storage before its future can be polled or detached. Header values must be ASCII and must not use the listed URL Loading System reserved fields. | [`OwnedRequest::from_request`](../../platform/ios/ios-network/src/conversion.rs#L10-L56) |
| Native request construction | On first poll, Rust creates Foundation strings and `NSURL`, checks for a host, creates `NSMutableURLRequest`, sets method and headers, and supplies body `NSData`. It then creates/resumes a URL-session data task. | [`IosSendFuture::start` and `make_request`](../../platform/ios/ios-network/src/platform.rs#L95-L205) |
| Native response boundary | The completion callback receives Foundation data/response/error values, reads `NSHTTPURLResponse` status and `allHeaderFields`, converts exposed headers, and copies response body bytes into Rust-owned storage before callback return. | [`response_from_callback`](../../platform/ios/ios-network/src/platform.rs#L207-L241) |

The owned staging values are not a replacement for Foundation. They establish ownership across an asynchronous operation whose callback may outlive the caller-facing future; the callback must not rely on borrowed request memory after that future is dropped. The response body is likewise copied before URLSession's callback-lifetime data is released. This review does not claim those copies are free or measure their cost.

## Exact Apple behavior retained

`URLSession` accepts a Foundation URL request, not the portable borrowed Rust view. Apple's documentation distinguishes the request object, which represents URL/method/headers/body and policies, from `URLSession`, which sends it. The current B3 path therefore must create Foundation request values even if portable validation remains in Rust. A Rust URL parser or serializer would not remove the Foundation request required by this backend.

The request object is also only one part of the behavior. The URL Loading System owns URL handling and transport policy, including reserved-header handling, cache/cookie/credential configuration, proxy and authentication behavior, redirects, and system network execution. For example:

- `NSMutableURLRequest.addValue:forHTTPHeaderField:` appends repeated values with a comma, and header names are case-insensitive; Apple says reserved headers should not be set through this method. The backend uses this documented behavior and rejects its known reserved-field set rather than claiming separate duplicate wire fields.
- `NSURLRequest` documents that URL Loading System behavior may ignore, overwrite, or omit reserved header values; it derives `Content-Length` from a known-length body.
- `NSHTTPURLResponse.allHeaderFields` exposes a dictionary. Dictionary keys are unique, and Apple documents canonicalization of some field names; the current response boundary cannot promise original duplicate-field multiplicity or ordering.
- The default URL-session configuration uses persistent disk caching, the user's Keychain credentials, and the shared cookie store by default. This is observable system behavior, not work performed by the portable request validator.
- The URL-session completion handler runs on its delegate queue and supplies Foundation response/data/error objects. Replacing that callback surface entails changing the transport integration, not replacing a small Rust algorithm.

These semantics and current limitations are documented in [the iOS HTTP guide](../ios/network.md). Apple references: [URLSession data-task request and callback](https://developer.apple.com/documentation/foundation/urlsession/datatask%28with%3Acompletionhandler%3A%29-e6xv), [NSURLRequest and reserved headers](https://developer.apple.com/documentation/foundation/nsurlrequest?changes=_2), [NSMutableURLRequest header addition](https://developer.apple.com/documentation/foundation/nsmutableurlrequest/addvalue%28_%3Aforhttpheaderfield%3A%29), [NSHTTPURLResponse headers](https://developer.apple.com/documentation/foundation/httpurlresponse/allheaderfields), and [default session configuration](https://developer.apple.com/documentation/foundation/urlsessionconfiguration/default?changes=__4).

## Candidate assessment

| Candidate | Why it is not an E1 replacement candidate |
| --- | --- |
| Reimplement URL parsing in Rust | The portable facade deliberately performs only shallow validation. The backend must still obtain an `NSURL`/request usable by URLSession; doing full parsing first would add work and would not eliminate Foundation's acceptance/normalization boundary. No equivalent-input/output benchmark hypothesis is established. |
| Reimplement request serialization or HTTP transport in Rust | This would replace URL Loading System transport and its system-managed behavior, not the bounded construction of a URLSession request. It would change the behavioral contract and requires correctness, TLS/trust, proxy/authentication, redirects, cache/cookie, policy, and performance evidence that does not exist. A raw-socket or third-party HTTP comparison would not be apples-to-apples with the current URLSession behavior. |
| Remove Rust staging allocations around Foundation conversion | This may be considered as a separately scoped adapter optimization, but it replaces no Apple behavior. It would need to preserve the escaping callback/drop ownership rules and compare the same Foundation request/result contract. This source review does not establish an end-to-end or device-level win, so it is not a Rust replacement proposal and no benchmark is added here. |

There is no candidate semantic subset to claim and no Rust candidate implementation to compare. Do not compare a borrowed `HttpRequest` value with an `NSURLRequest` submitted to URLSession: they differ in parsing, ownership, policies, and the native work performed.

## Parity, performance, and default

- Apple baseline: Foundation `NSURL`/`NSMutableURLRequest` plus URLSession's default foreground configuration, as implemented in B3. Existing host conversion fixtures test Rust-owned values and error mapping; they are not Apple differential tests.
- Parity evidence for a Rust replacement: none; no candidate exists, and no Apple URLSession differential runner was run by E1.
- Performance evidence: none; no Release timing, allocation count, device measurement, or linkage comparison was made by E1.
- Default decision: keep Foundation/URLSession. Reconsider only after identifying a narrower Foundation behavior that is both replaceable without changing the observable contract and supported by a credible cost hypothesis. Any Rust replacement would then require deterministic Apple differential parity plus representative physical-device Release measurements before selection.

No files under `benchmarks/replacements/http-construction/**` or candidate parity fixtures are added because they would compare no candidate. A benchmark harness without a viable, equivalent workload would not constitute evidence.
