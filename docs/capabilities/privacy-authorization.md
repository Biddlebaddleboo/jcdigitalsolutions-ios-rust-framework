# App Tracking Transparency status

`framework-auth` exposes a portable status enum and a statically dispatched, synchronous backend trait for App Tracking Transparency (ATT) only. The query reports the current status for the calling app; it does not query another app. This is not a general privacy-authorization facade.

The portable values reflect Apple's [ATT authorization-status definitions](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/authorizationstatus) and [calling-application status property](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/trackingauthorizationstatus):

- `NotDetermined`: the app cannot determine the authorization status for tracking-related data; Apple also reports this before the device receives an authorization request. It is neither approval nor denial.
- `Restricted`: authorization to access tracking-related data is restricted. Apple notes this can be system-managed before a prompt; it is distinct from a user denial.
- `Denied`: the user denies authorization to access app-related data that may be used to track the user or device.
- `Authorized`: the user authorizes access to app-related data that may be used to track the user or device.
- `Unknown`: the backend received a native status value this version does not recognize. Backends preserve it as `Unknown`, not as authorization.

This is a query-time status for the calling app and can change when the user or system changes tracking authorization. The query is synchronous and does not request authorization or show a prompt.

`Authorized` reports only the ATT state. It is not general privacy consent, account consent, data-access status, or advertising-identifier availability, and it does not guarantee that tracking is lawful or otherwise allowed. This crate does not request authorization, access an identifier, or perform tracking.

The iOS status-only adapter is described in the [iOS tracking authorization guide](../ios/tracking-authorization.md). The capability manifest row is `023-security-auth-privacy-authorization`; it records partial support for this ATT-only slice, not general privacy authorization.
