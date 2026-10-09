# App Tracking Transparency status

`framework-auth` exposes a portable status enum and a static backend trait for App Tracking Transparency (ATT) only. It is not a general privacy authorization facade.

The status values map to Apple's ATT states for the calling app: `NotDetermined`, `Restricted`, `Denied`, and `Authorized`. `Unknown` preserves a native status value this version does not recognize. The status is a query-time value and can change when the user or system changes tracking authorization.

`Authorized` reports only the ATT authorization state. It does not report authorization for another privacy feature, account consent, advertising-identifier availability, or whether a data use is allowed by law or policy. This crate does not request authorization, access an identifier, or perform tracking.

The iOS status-only adapter is described in the [iOS tracking authorization guide](../ios/tracking-authorization.md). The capability manifest row is `023-security-auth-privacy-authorization`; it records partial support for this ATT-only slice, not general privacy authorization.
