#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Point-in-time MessageUI status queries for iOS"]

#[cfg(target_os = "ios")]
/// Main-thread proof for each UIKit status read
pub use objc2::MainThreadMarker;

#[cfg(target_os = "ios")]
use objc2_message_ui::{MFMailComposeViewController, MFMessageComposeViewController};

/// Reads `MFMailComposeViewController::canSendMail` on the main thread
///
/// The result is Apple's point-in-time setup status only. It does not prove a later send will
/// succeed, identify an account, open compose UI, request permission, create a message, or send one
///
/// This returns `false` below iOS 3.0
#[cfg(target_os = "ios")]
pub fn can_send_mail(mtm: MainThreadMarker) -> bool {
    if !objc2::available!(ios = 3.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 3.0 API floor and `mtm` proves main-thread access
    unsafe { MFMailComposeViewController::canSendMail(mtm) }
}

/// Reads `MFMessageComposeViewController::canSendText` on the main thread
///
/// The result is Apple's point-in-time setup status for text-only messages. It does not prove a
/// later send will succeed, identify an account, open compose UI, request permission, create a
/// message, or send one
///
/// This returns `false` below iOS 4.0
#[cfg(target_os = "ios")]
pub fn can_send_text(mtm: MainThreadMarker) -> bool {
    if !objc2::available!(ios = 4.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 4.0 API floor and `mtm` proves main-thread access
    unsafe { MFMessageComposeViewController::canSendText(mtm) }
}
