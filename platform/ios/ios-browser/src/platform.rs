use core::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use block2::RcBlock;
use framework_format::Uri;
use ios_runtime::main_thread::MainThread;
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject, Bool};
use objc2_foundation::{NSDictionary, NSString, NSURL};
use objc2_ui_kit::{UIApplication, UIApplicationOpenExternalURLOptionsKey};

/// A bounded reason an HTTPS URI cannot be sent to the system URL handler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenError {
    /// The URI does not use the HTTPS scheme.
    HttpsRequired,
    /// The URI has no nonempty host.
    HostRequired,
    /// Foundation rejected the otherwise syntactically valid URI.
    FoundationRejectedUri,
}

/// Request that iOS open a validated HTTPS URI with its system URL handler.
///
/// `uri` is an absolute RFC 3986 URI from `framework-format`; its exact text is preserved until
/// Foundation constructs an `NSURL`. This function accepts only HTTPS and requires a nonempty
/// Foundation host. `Ok(())` means only that `UIApplication.openURL:options:completionHandler:` was
/// called. The asynchronous `completion` receives UIKit's URL-open result: `true` means iOS
/// reports that it opened the URL with a handler, and `false` means it did not. Neither result
/// proves that a remote page loaded or was viewed, and a Universal Link may open an associated app
/// instead of a browser. Safari is not guaranteed.
///
/// UIKit and the completion are main-thread-affine. The system invokes the completion
/// asynchronously on the app's main queue. Panics from the callback are caught and discarded so
/// they cannot unwind through the Objective-C block boundary. The caller must keep any captured
/// state valid for the asynchronous callback. This API does not use `canOpenURL`, create a
/// presenter, or expose a UIKit handle.
pub fn open_external_https_uri<F>(
    main_thread: MainThread,
    uri: Uri<'_>,
    completion: F,
) -> Result<(), OpenError>
where
    F: FnOnce(bool) + 'static,
{
    if !uri.scheme().eq_ignore_ascii_case("https") {
        return Err(OpenError::HttpsRequired);
    }
    if uri.authority().is_none_or(str::is_empty) {
        return Err(OpenError::HostRequired);
    }

    autoreleasepool(|_| {
        let text = NSString::from_str(uri.as_str());
        let Some(url) = NSURL::URLWithString(&text) else {
            return Err(OpenError::FoundationRejectedUri);
        };
        if url.host().is_none_or(|host| host.length() == 0) {
            return Err(OpenError::HostRequired);
        }

        let callback = Rc::new(RefCell::new(Some(completion)));
        let handler = RcBlock::new(move |opened: Bool| {
            let callback = callback.borrow_mut().take();
            if let Some(callback) = callback {
                let _ = catch_unwind(AssertUnwindSafe(|| callback(opened.as_bool())));
            }
        });
        let application = UIApplication::sharedApplication(main_thread.into_objc2());
        let options =
            NSDictionary::<UIApplicationOpenExternalURLOptionsKey, AnyObject>::dictionary();
        // SAFETY: `options` is an empty dictionary with the exact key/value types required by the
        // public UIKit binding; no option values are supplied.
        unsafe {
            application.openURL_options_completionHandler(&url, &options, Some(&handler));
        }
        Ok(())
    })
}
