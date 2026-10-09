# iOS App Tracking Transparency status

`ios-auth` reads only `ATTrackingManager.trackingAuthorizationStatus` and maps Apple's four documented values to `AppTrackingAuthorizationStatus`. The iOS SDK declares App Tracking Transparency available from iOS 14.0. The host app must use an iOS deployment target of 14.0 or later before calling the backend; this crate cannot enforce the host target. Mac Catalyst is excluded.

The query is status-only. Apple documents the separate `requestTrackingAuthorizationWithCompletionHandler:` method as the authorization request. This adapter does not call it, enable the binding's `block2` feature, instantiate `ATTrackingManager`, access IDFA, or perform tracking. A `NotDetermined` result is not a grant or denial; `Restricted` is distinct from `Denied`.

Apple's App Tracking Transparency and `NSUserTrackingUsageDescription` documentation states that an app using this API must provide the usage-description key. The adapter therefore requires the host to configure `NSUserTrackingUsageDescription` in `Info.plist`. Reading the status property does not itself display the authorization prompt; only the separate request API does so. This adapter does not configure or validate entitlements, app privacy declarations, or tracking policy.

The status belongs to the calling app and covers ATT only. `Authorized` does not establish authorization for another privacy feature, availability of an advertising identifier, or permission for a particular data use. No live device/simulator status, prompt, user choice, or tracking operation is claimed.

## Binding and safety audit

The backend pins `objc2-app-tracking-transparency` 0.3.2 with `default-features = false`. This omits its optional `block2` feature and the generated block-based request method. The generated status getter has the exact public selector and returns a copyable `ATTrackingManagerAuthorizationStatus` scalar. The binding links `AppTrackingTransparency.framework`. The private wrapper documents the iOS 14 API-floor invariant and scalar return; no raw pointer, callback, ownership transfer, native object, or request call crosses the boundary.

## Apple references

- [ATTrackingManager.trackingAuthorizationStatus](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/trackingauthorizationstatus)
- [ATTrackingManager.AuthorizationStatus](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/authorizationstatus)
- [App Tracking Transparency](https://developer.apple.com/documentation/apptrackingtransparency)
- [NSUserTrackingUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsusertrackingusagedescription)
