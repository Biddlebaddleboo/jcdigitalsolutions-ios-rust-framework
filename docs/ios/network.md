# iOS foreground HTTP

`ios-network` implements the portable [`framework-network`](../capabilities/app-data.md) `HttpBackend` contract with a private Foundation `NSURLSession` and `NSMutableURLRequest`. It is iOS-only and imports Foundation; it does not link `Network.framework`, UIKit, or Swift runtime support.

```rust,ignore
use framework_network::{Header, HttpClient, HttpMethod, HttpRequest, HttpUrl};
use ios_network::IosHttpBackend;

async fn fetch_status() -> Result<u16, framework_network::NetworkError> {
    let method = HttpMethod::new("GET")?;
    let url = HttpUrl::new("https://api.example.test/v1")?;
    let headers = [Header::new("Accept", b"application/json")?];
    let request = HttpRequest::new(method, url, &headers, None);
    let mut client = HttpClient::new(IosHttpBackend::new());
    let response = client.send(request).await?;
    Ok(response.status().get())
}
```

The caller supplies its own executor or polls the Rust future directly. Creating `HttpClient` does no network work. `HttpBackend::send` copies request values when called; the returned future creates and resumes a URL-session data task on its first poll. The future borrows the backend, so it cannot outlive the session owner. There is no Tokio/async-std dependency, executor discovery, global session registry, retry loop, background task, or framework-owned connection pool.

## Request and response semantics

- HTTP and HTTPS URLs and method tokens pass through Foundation string conversion and URL parsing. A URL that `NSURL URLWithString:` cannot construct, or that has no host, returns `NetworkError::InvalidUrl`. Foundation and URLSession can parse, canonicalize, redirect, or serialize URL components; this backend does not promise wire-level preservation of the original URL string.
- Method spelling is passed to `NSMutableURLRequest.setHTTPMethod:`. Request bodies are copied from the borrowed portable slice into an owned Rust `Vec<u8>`, then transferred into immutable `NSData` with `NSData::from_vec`; `setHTTPBody:` gives the request its own Foundation value. No text encoding is applied to body bytes.
- Header names and values become `NSString`. Request header values with any non-ASCII byte are rejected as `InvalidHeaderOrMethod`; converting those bytes to a string would not preserve their octets. The portable contract accepts these bytes, so this is a deliberate iOS-backend restriction.
- The URL Loading System reserves `Content-Length`, `Authorization`, `Connection`, `Host`, `Proxy-Authenticate`, `Proxy-Authorization`, and `WWW-Authenticate`; Apple documents that values for these fields may be ignored, overwritten, or omitted. This backend rejects them case-insensitively as `InvalidHeaderOrMethod` instead of sending a silently altered request. `Content-Length` is derived from the body when possible.
- Duplicate request fields are submitted in caller order with `addValue:forHTTPHeaderField:`. Foundation appends a prior field value with a comma, and field names are case-insensitive. Thus duplicates become a comma-combined value in the request dictionary; the URL Loading System controls final field ordering and wire serialization. This cannot preserve separate duplicate field lines or every field's semantics.
- Response status values from 100 through 599, including non-success statuses, are ordinary `HttpResponse` values. A missing/non-HTTP response or a response header value that cannot satisfy the portable `ResponseHeader` rules becomes a platform backend error.
- Response body bytes are copied from the callback's borrowed `NSData` into the owned `Vec<u8>` before the callback returns. A missing body with a valid response is treated as empty.
- Foundation exposes response headers through `NSHTTPURLResponse.allHeaderFields`, an `NSDictionary`, not an ordered multi-map. Duplicate response fields and their original order are therefore not promised; URL Loading System may also canonicalize some field names. The backend copies the entries Foundation exposes and converts `NSString` values to UTF-8 bytes.

These conversions preserve method, body bytes, and representable single header values as far as the selected URL Loading System APIs permit. They are not a raw-socket HTTP implementation or a claim of byte-for-byte HTTP parity.

## Errors, completion, and drop

`NSURLSession` completion runs on its delegate queue, not necessarily the main thread. The callback copies the response body and header strings before it returns. A completion may arrive during the first poll, before that poll saves a waker; the cell keeps the result for that poll. The cell unlocks before a wake, so a reentrant poll can read the result without a lock cycle. Panics in native request setup or response conversion within `catch_unwind` map to `NetworkError::Backend` with `ErrorKind::Internal`; no panic crosses the Objective-C block boundary. The block captures only `Arc<CompletionCell<...>>`; the cell uses a mutex and the block holds no Objective-C object or borrowed request data.

The completion cell accepts one terminal result, wakes the currently registered Rust task after releasing its lock, and ignores any later completion. Dropping the future detaches its waker and result interest, then calls `NSURLSessionTask.cancel()` when a task has started. Cancellation is a request, not proof the transfer stopped immediately; URLSession can finish cancellation asynchronously and may deliver its cancellation completion after the Rust future is gone. The callback-owned state remains alive until the native block is released. Dropping an unpolled future creates no task. Dropping after native completion but before polling the result discards that result and requests task cancellation if the task handle remains.

`NSError` from `NSURLErrorDomain` maps `NSURLErrorCancelled` to `Cancelled`, `NSURLErrorTimedOut` to `Timeout`, and the documented host/connectivity errors `NSURLErrorCannotFindHost`, `NSURLErrorCannotConnectToHost`, `NSURLErrorNetworkConnectionLost`, and `NSURLErrorNotConnectedToInternet` to `Unavailable`. Other native failures map to `Platform`. The signed native code is retained when it fits the current `PlatformErrorCode(i32)` contract; an out-of-range `NSInteger` is not truncated and has no portable code. The domain is used for category selection but is not retained because current `framework_core::Error` has no native-domain field. The portable contract would need a bounded iOS detail extension before callers can retrieve the original domain string.

## Defaults, privacy, and availability

`IosHttpBackend::new()` creates a private session from `defaultSessionConfiguration`, not `sharedSession`. Foundation's default configuration uses the shared URL cache, persistent credential storage, and shared cookie store; cookie handling is enabled by default. Request construction leaves the URL request cache policy and 60-second request timeout at Foundation's defaults. URL Loading System redirect, authentication, proxy, and transport behavior remains native. The backend adds no cookie jar, retry policy, redirect policy, or pool. When the backend is dropped, it calls `finishTasksAndInvalidate`, which prevents new tasks while allowing existing tasks to finish. Dropping a request future first calls `cancel`; its callback may still arrive asynchronously. Use `IosHttpBackend::with_session` to supply a differently configured caller-owned session; the backend does not invalidate that session, so its caller remains responsible for session lifecycle. `native_session()` provides the iOS-specific escape hatch.

App Transport Security applies to URL Loading System requests. Apps linked against the iOS 9 SDK or later require HTTPS by default; an `http` URL can fail unless the app has an appropriate narrow `NSAppTransportSecurity` exception. Access to local-network addresses is subject to Local Network privacy on iOS 14 and later and requires an `NSLocalNetworkUsageDescription` entry plus user authorization. This backend does not browse Bonjour or use multicast, so it does not require `NSBonjourServices` or the multicast entitlement. Ordinary internet URLSession requests require no framework-added usage string or entitlement.

The installed iOS 26.5 SDK headers mark `NSURLSession` and `NSURLSessionConfiguration` available from iOS 7.0; the backend's minimum API floor is therefore iOS 7.0. This is API availability, not a claim that every server, ATS configuration, permission state, or deployment target can perform every request.

## Dependencies and linkage

- `objc2` and `objc2-foundation` provide typed public Objective-C/Foundation bindings with only the URL, request, response, session, data, string, dictionary, and error modules enabled.
- `block2` supplies the owned escaping completion block and the `NSData::from_vec` deallocator callback. The URLSession binding marks its selector unsafe because block sendability cannot be expressed in its Rust type. The callback captures only `Arc<CompletionCell<...>>`, whose contents are synchronized; it does not use `Rc`, `RefCell`, or Objective-C values across threads.
- Foundation is the smallest public Apple boundary for URLSession. `Network.framework` is not used because this contract is foreground HTTP, not low-level connection/listener access.
- The binding crates and block helper are behind this iOS crate; they do not enter the portable `framework-network` API. Replacing `objc2-foundation` or `block2` requires an internal adapter change, not a portable API change.

## Validation status

| Gate | Recorded result | Evidence supports | Does not establish |
| --- | --- | --- | --- |
| Host unit tests | 10 deterministic tests passed | Owned request conversion, response conversion/error mapping, and completion/detach races | Foundation/URLSession runtime behavior, HTTP parity, or performance |
| iOS device and Simulator `cargo check` / strict Clippy | Pass in the recorded Xcode 26.6 / iOS 26.5 SDK environment | Generated Apple bindings type-check for both Rust targets | A task starts, sends a request, or receives a callback |
| Release link/import probe | Pass; probes are link-only and not executed | Direct imports `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; the script's selected Swift/Python/runtime and unrelated capability symbol patterns are rejected | URLSession runtime behavior, local-server parity, or performance |
| URLSession local-fixture differential | Not run; this crate has no integration-test app/runner | No Apple reference result is available | HTTP parity |
| Representative-device Release measurement | Not run; E1 found no replacement candidate | No performance result is available | A performance win or default change |

The recorded Xcode version is below the repository's Xcode 27.x baseline. No live external endpoint
is used. A future parity record must follow the artifact requirements in
[`PLAN_VALIDATION_IOS_NETWORK.md`](../../PLAN_VALIDATION_IOS_NETWORK.md); compile and link results do
not substitute for an Apple runtime reference.

References: [URLSession completion handler and delegate queue](https://developer.apple.com/documentation/foundation/urlsession/datatask%28with%3Acompletionhandler%3A%29-e6xv), [URLSession invalidation lifecycle](https://developer.apple.com/documentation/foundation/urlsession/finishtasksandinvalidate%28%29?language=objc), [NSURLRequest reserved headers](https://developer.apple.com/documentation/foundation/nsurlrequest), [NSMutableURLRequest duplicate header behavior](https://developer.apple.com/documentation/foundation/nsmutableurlrequest/addvalue%28_%3Aforhttpheaderfield%3A), [NSHTTPURLResponse headers](https://developer.apple.com/documentation/foundation/httpurlresponse/allheaderfields), [default session configuration](https://developer.apple.com/documentation/foundation/urlsessionconfiguration/default), [App Transport Security](https://developer.apple.com/documentation/bundleresources/information-property-list/nsapptransportsecurity), and [local network privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy).
