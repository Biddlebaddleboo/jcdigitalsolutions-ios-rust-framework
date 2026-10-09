# iOS strict Foundation URLs

`ios-url` pairs a portable `framework_format::Uri` with an owned `NSURL` for native APIs that need one. The owner retains the exact source `Uri` and exposes it through `source_uri`; `as_ns_url` lends the native object for the owner lifetime

`NativeUrl::new` converts the exact `Uri::as_str()` text to `NSString` and calls only `NSURL::URLWithString_encodingInvalidCharacters` with `false`. Foundation may reject a URI that passed the portable RFC 3986 parser; that result maps to `NativeUrlError::FoundationRejectedUri`. There is no fallback to `URLWithString:`

The strict initializer is available on iOS 17.0 and later. The link/import script sets the minimum to iOS 17.0 for both device and Simulator probes and verifies the linked artifact with `vtool`. The consuming app must also honor the iOS 17.0 API floor

This adapter does not normalize or decode the source text, resolve URI references, infer URL equivalence, open URLs, send network requests, or expose raw pointers. `NSURL` parser output and behavior are not claimed to match the source `Uri` in every detail

`sh platform/ios/ios-url/check-link-imports.sh` builds probes for device and Simulator and inspects their direct imports, Objective-C symbols, initializer selector, and deployment metadata. It does not run either probe. Compile/link evidence does not prove runtime availability, Foundation acceptance, URL-open behavior, parser parity, or performance

## Validation status

The local toolchain, SDK header basis, exact checks, imports, and evidence limits are recorded in `PLAN_IOS_URL.md`
