use objc2_class_kit::NSUserActivityCLSDeepLinks;
use objc2_foundation::NSUserActivity;

/// Return the ClassKit deep-link marker from a caller-owned user activity.
///
/// This reads only the `isClassKitDeepLink` Boolean property. It does not read
/// `contextIdentifierPath`, access the ClassKit data store, or validate assignment content. Call
/// only on iOS 11.3 or later. The synchronous getter does not retain or store `activity` and does
/// not hop threads; the host must use it while the supplied `NSUserActivity` is valid and in
/// accordance with the host activity lifecycle.
pub fn is_classkit_deep_link(activity: &NSUserActivity) -> bool {
    // SAFETY: `activity` is a borrowed, live Foundation `NSUserActivity`, the generated binding
    // names the read-only ClassKit category property with the matching BOOL ABI, and callers must
    // provide an iOS 11.3+ deployment/runtime environment where the selector is available.
    unsafe { activity.isClassKitDeepLink() }
}
