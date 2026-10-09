use framework_format::Uri;
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::{MainThreadMarker, MainThreadOnly, extern_class, extern_methods};
use objc2_foundation::{NSString, NSURL};
use objc2_ui_kit::UIViewController;

/// A bounded reason an HTTPS URI cannot create a Safari controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafariControllerError {
    /// The URI does not use the HTTPS scheme.
    HttpsRequired,
    /// The URI or Foundation URL has no nonempty host.
    HostRequired,
    /// Foundation rejected the caller's URI text.
    FoundationRejectedUri,
}

/// A main-thread-owned `SFSafariViewController` for host-managed presentation.
///
/// The wrapper retains the controller but does not present or dismiss it. Keep it on the main
/// thread and call [`Self::view_controller`] to pass a borrowed base controller to UIKit.
pub struct IosSafariViewController {
    controller: Retained<NativeSafariViewController>,
    _main_thread: MainThreadMarker,
}

impl IosSafariViewController {
    /// Creates one Safari controller for an HTTPS URI with a nonempty host.
    ///
    /// `Ok` confirms only that Foundation and SafariServices constructed a controller. It does not
    /// prove that UIKit presented it, that a URL request began, or that a page loaded.
    pub fn new(main_thread: MainThread, uri: Uri<'_>) -> Result<Self, SafariControllerError> {
        if !uri.scheme().eq_ignore_ascii_case("https") {
            return Err(SafariControllerError::HttpsRequired);
        }
        if uri.authority().is_none_or(str::is_empty) {
            return Err(SafariControllerError::HostRequired);
        }

        let marker = main_thread.into_objc2();
        autoreleasepool(|_| {
            let text = NSString::from_str(uri.as_str());
            let Some(url) = NSURL::URLWithString(&text) else {
                return Err(SafariControllerError::FoundationRejectedUri);
            };
            if url.host().is_none_or(|host| host.length() == 0) {
                return Err(SafariControllerError::HostRequired);
            }

            // SAFETY: The inspected public iOS SDK declares `initWithURL:` as the
            // `SFSafariViewController` designated initializer with an `NSURL *` argument and
            // `instancetype` result. The class is main-thread-only, and `url` stays retained for
            // the initializer call.
            let controller = unsafe {
                NativeSafariViewController::initWithURL(
                    NativeSafariViewController::alloc(marker),
                    &url,
                )
            };
            Ok(Self {
                controller,
                _main_thread: marker,
            })
        })
    }

    /// Returns the Safari controller as a base UIKit controller for host-managed presentation.
    ///
    /// The returned borrow cannot outlive this wrapper. Presentation and dismissal remain the
    /// caller's UIKit responsibility and must occur on the main thread.
    pub fn view_controller(&self) -> &UIViewController {
        &self.controller
    }
}

extern_class!(
    /// Typed local declaration for the public SafariServices controller class.
    #[unsafe(super(UIViewController))]
    #[thread_kind = MainThreadOnly]
    #[name = "SFSafariViewController"]
    struct NativeSafariViewController;
);

#[allow(non_snake_case)]
impl NativeSafariViewController {
    extern_methods!(
        // SAFETY: Mirrors the public iOS SDK declaration `initWithURL:`. The initializer is
        // main-thread-only and takes one retained NSURL pointer under NS_ASSUME_NONNULL.
        #[unsafe(method(initWithURL:))]
        #[unsafe(method_family = init)]
        pub unsafe fn initWithURL(this: Allocated<Self>, url: &NSURL) -> Retained<Self>;
    );
}
