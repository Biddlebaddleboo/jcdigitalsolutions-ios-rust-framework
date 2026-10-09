# iOS Sign in with Apple credential-state query

This package makes one query through `ASAuthorizationAppleIDProvider.getCredentialStateForUserID:completion:` for a user ID that the host previously received from a successful Sign in with Apple authorization. It does not start an authorization request, show UI, authenticate a user, validate a server session, or report passkey readiness.

The operation is available from iOS 13.0 and links `AuthenticationServices.framework`. The host must enable the Sign in with Apple capability and provide `com.apple.developer.applesignin` with the normal `Default` value. Apple describes that entitlement as enabling Sign in with Apple but does not say whether this query alone is exempt, so this package takes the conservative entitlement-scoped position. No separate usage-description key was found in the reviewed Apple docs.

`get_credential_state` copies the supplied ID to an `NSString`. Its callback receives a Rust-owned `CredentialStateSnapshot` with all four Apple states, an `Unknown(i64)` fallback, and an independent `had_error` bit. `NotFound` can arrive with an error, so both fields must be kept. The bit records error presence only; it omits error domain, code, and localized text. The callback may run on any queue and may run before the call returns. The framework has no cancellation operation; abandoning downstream work does not cancel the query. The callback receives no ID, `NSError`, identity token, authorization code, or Apple object.

The result is a point-in-time credential state for this one identifier only. It is not a server-session check, general Sign in with Apple readiness, passkey state, or an offline-availability promise. The package has no portable contract or root capability facade/feature wiring.

## Focused compile gate

Run `./check.sh` from this directory. It runs compile, strict Clippy, and rustdoc gates for iOS device and arm64 Simulator targets, then builds and inspects release Mach-O link probes for both targets. The gate does not run tests or execute either probe. Its expected import set is in `link-imports-expected.txt`.
