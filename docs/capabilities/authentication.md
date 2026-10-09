# Local authentication

`framework-auth` defines a portable, one-shot local user-presence contract over a statically
selected `AuthenticationBackend`. Construct `Authenticator::new(backend)`, query policy-specific
availability without prompting, then pass an `AuthenticationRequest` to `authenticate` for an
explicit operation. Availability is synchronous and may block; authentication uses an async
future. The `no_std` contract uses no global registry, hidden initialization, or executor.

## Policy and result

`AuthenticationPolicy::BiometricsOnly` requires biometrics and does not permit local-device-
credential fallback. `AuthenticationPolicy::DeviceOwner` lets the platform use biometrics or its
local device credential, such as a passcode. Availability is reported for the requested policy and
must not display a prompt. The contract does not expose biometric modality, samples, templates, or
any other biometric data.

`AuthenticationRequest::new` takes a borrowed UTF-8 reason string, rejects empty or whitespace-only
text with `AuthenticationError::InvalidReason`, and preserves all other text exactly. The caller
should supply a short explanation in the user's current language. The request borrow lasts through
completion or future drop; a backend that needs native storage beyond that point must own a copy.

`Ok(())` means only that the platform reported success for the selected policy at that operation's
completion. It does not prove a person's identity or promise that a later operation remains
authorized. This facade does not create, store, or retrieve credentials or secrets, issue remote
credentials, apply Keychain access policy, query general privacy permissions, or implement passkeys
or Sign in with Apple.

## Prompt, errors, and cancellation

The native operation starts no earlier than the returned future's first poll. Only an explicit
`authenticate` request may prompt. Dropping a pending future abandons its Rust result and requires
the backend to request native cancellation where supported. The backend must safely handle a
completion that races cancellation, ignore a late or duplicate result, and release callback state
exactly once. The contract does not promise that already-presented system UI is dismissed. No
executor or `Send` requirement is imposed.

`AuthenticationError::InvalidReason` maps to `ErrorKind::InvalidInput` and has no platform code.
`AuthenticationError::Backend(Error)` preserves the backend's `ErrorKind` and optional
`PlatformErrorCode`; the backend maps recognized user/system cancellation to `ErrorKind::Cancelled`.

## iOS backend boundary

The iOS backend is `ios-auth`; its policy mapping, API floors, callback queue, cancellation limits,
error table, and `NSFaceIDUsageDescription` requirement are in [the iOS authentication guide](../ios/authentication.md).
It uses Apple's Objective-C `LAContext` policy-evaluation API and public LocalAuthentication
framework. The portable crate makes no claim that every platform supports either policy, and it does
not set host-app plist values or claim a live prompt result. See Apple's [LAContext](https://developer.apple.com/documentation/localauthentication/lacontext?changes=_9&language=objc),
[policy evaluation](https://developer.apple.com/documentation/localauthentication/lacontext/evaluatepolicy%28_%3Alocalizedreason%3Areply%3A%29?changes=__4),
[context invalidation](https://developer.apple.com/documentation/localauthentication/lacontext/invalidate%28%29?language=objc),
and [NSFaceIDUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsfaceidusagedescription?language=data%2Cdata%2Cdata%2Cdata%2Cdata%2Cdata%2Cdata%2Cdata).
