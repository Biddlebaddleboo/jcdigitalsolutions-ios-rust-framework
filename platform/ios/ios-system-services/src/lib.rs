#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow ClassKit deep-link marker query for caller-owned iOS user activities."]
#![doc = "\n\nThis package only reads `NSUserActivity.isClassKitDeepLink` and returns its Boolean value. It does not read the context identifier path, ClassKit data store, contexts, activities, assignment data, or user identity. Call only on iOS 11.3 or later. The caller retains ownership of the activity and must respect the host app's activity lifecycle and thread requirements."]

#[cfg(target_os = "ios")]
mod classkit;

#[cfg(target_os = "ios")]
pub use classkit::is_classkit_deep_link;
