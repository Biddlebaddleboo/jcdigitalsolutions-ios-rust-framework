use std::panic::{AssertUnwindSafe, catch_unwind};

use framework_core::{Error, ErrorKind};
use framework_web::{HttpsUrl, NavigationState, WebView, WebViewError};
use ios_runtime::main_thread::MainThread;
use objc2::ffi::NSInteger;
use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{
    MainThreadMarker, MainThreadOnly, define_class, extern_class, extern_methods, extern_protocol,
    msg_send,
};
use objc2_core_foundation::CGRect;
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSURL, NSURLRequest};
use objc2_ui_kit::UIView;
use objc2_web_kit::{
    WKNavigation, WKNavigationAction, WKNavigationResponse, WKWebViewConfiguration,
};

const NAVIGATION_POLICY_CANCEL: NSInteger = 0;
const NAVIGATION_POLICY_ALLOW: NSInteger = 1;

/// A caller-owned iOS web view created for one explicitly supplied initial HTTPS URL.
///
/// The wrapper retains its navigation delegate, is main-thread-affine, and does not expose the
/// native WebKit object. Drop stops loading, clears the weak native delegate, and detaches the
/// view from its parent if attached.
pub struct IosWebView {
    view: Retained<NativeWkWebView>,
    _delegate: Retained<NavigationDelegate>,
    _main_thread: MainThreadMarker,
}

impl IosWebView {
    /// Creates a native web view and issues its first request for `initial_url`.
    ///
    /// This returns when WebKit accepts the request for navigation, not when a page finishes or
    /// becomes visible. Attach the result with [`Self::attach_to`] to display it.
    pub fn new(main_thread: MainThread, initial_url: HttpsUrl<'_>) -> Result<Self, WebViewError> {
        let marker = main_thread.into_objc2();
        autoreleasepool(|_| {
            let url = native_https_url(initial_url)?;
            let request = NSURLRequest::requestWithURL(&url);
            // SAFETY: This uses WKWebViewConfiguration's public NSObject-derived initializer on
            // the required main thread; the returned retained configuration owns its lifetime.
            let configuration = unsafe { WKWebViewConfiguration::new(marker) };
            // SAFETY: The public iOS SDK declares this designated initializer as
            // `initWithFrame:configuration:` with `CGRect` and `WKWebViewConfiguration *`.
            // The objc2 binding crate currently omits WKWebView on iOS; this local declaration
            // mirrors only that public selector and keeps Objective-C ownership typed.
            let view = unsafe {
                NativeWkWebView::initWithFrame_configuration(
                    NativeWkWebView::alloc(marker),
                    CGRect::ZERO,
                    &configuration,
                )
            };
            let delegate = NavigationDelegate::new(marker);
            let protocol_delegate = ProtocolObject::from_ref(&*delegate);
            // SAFETY: WKWebView.navigationDelegate is a weak `id<WKNavigationDelegate>` property;
            // the wrapper stores a strong retain of this main-thread delegate for the view's life.
            unsafe { view.setNavigationDelegate(Some(protocol_delegate)) };
            // SAFETY: This public method takes an NSURLRequest and returns a nullable WKNavigation;
            // both arguments and the native view remain retained for the call.
            let navigation: Option<Retained<WKNavigation>> = unsafe { view.loadRequest(&request) };
            if navigation.is_none() {
                return Err(unavailable());
            }
            Ok(Self {
                view,
                _delegate: delegate,
                _main_thread: marker,
            })
        })
    }

    /// Sets the view's frame and adds it as a child of `parent`.
    ///
    /// UIKit retains the child in its view hierarchy. This wrapper remains responsible for the
    /// child lifetime and detaches it on drop. Calls must remain on the main thread.
    pub fn attach_to(&mut self, parent: &UIView, frame: CGRect) {
        self.view.setFrame(frame);
        parent.addSubview(&self.view);
    }

    /// Sets the view's current frame on the main thread.
    pub fn set_frame(&mut self, frame: CGRect) {
        self.view.setFrame(frame);
    }
}

impl WebView for IosWebView {
    fn navigation_state(&self) -> NavigationState {
        // SAFETY: These read-only WKWebView properties are called on the main thread, as
        // guaranteed by the stored non-sendable marker and public constructor.
        let can_go_back = unsafe { self.view.canGoBack() };
        // SAFETY: Same main-thread and live-object invariant as `canGoBack` above.
        let can_go_forward = unsafe { self.view.canGoForward() };
        NavigationState::new(can_go_back, can_go_forward)
    }

    fn go_back(&mut self) {
        // SAFETY: The methods are called on the main thread. The second message is sent only when
        // WebKit reports that a back item currently exists.
        if unsafe { self.view.canGoBack() } {
            let _navigation = unsafe { self.view.goBack() };
        }
    }

    fn go_forward(&mut self) {
        // SAFETY: The methods are called on the main thread. The second message is sent only when
        // WebKit reports that a forward item currently exists.
        if unsafe { self.view.canGoForward() } {
            let _navigation = unsafe { self.view.goForward() };
        }
    }

    fn reload(&mut self) {
        // SAFETY: The view is live and this command is main-thread-affine.
        let _navigation = unsafe { self.view.reload() };
    }

    fn stop_loading(&mut self) {
        // SAFETY: The view is live and this command is main-thread-affine.
        unsafe { self.view.stopLoading() };
    }
}

impl Drop for IosWebView {
    fn drop(&mut self) {
        // SAFETY: `IosWebView` is !Send through `MainThreadMarker`, so its destructor remains on
        // the thread that created and owns this UIKit/WebKit view.
        unsafe { self.view.stopLoading() };
        self.view.removeFromSuperview();
        // SAFETY: Clearing the weak property is valid while both the view and delegate are alive;
        // this prevents the native view from retaining a stale Rust-owned delegate address.
        unsafe { self.view.setNavigationDelegate(None) };
    }
}

fn native_https_url(url: HttpsUrl<'_>) -> Result<Retained<NSURL>, WebViewError> {
    let text = NSString::from_str(url.as_str());
    let Some(native_url) = NSURL::URLWithString(&text) else {
        return Err(WebViewError::InvalidUrl);
    };
    if native_url_is_https_with_host(&native_url) {
        Ok(native_url)
    } else {
        Err(WebViewError::InvalidUrl)
    }
}

fn native_url_is_https_with_host(url: &NSURL) -> bool {
    let Some(scheme) = url.scheme() else {
        return false;
    };
    ascii_case_equal(&scheme, b"https") && url.host().is_some_and(|host| host.length() != 0)
}

fn ascii_case_equal(value: &NSString, expected: &[u8]) -> bool {
    if value.length() != expected.len() as _ {
        return false;
    }
    expected.iter().enumerate().all(|(index, byte)| {
        let actual = value.characterAtIndex(index as _);
        actual == u16::from(*byte) || actual == u16::from(byte.to_ascii_uppercase())
    })
}

fn action_is_allowed(action: &WKNavigationAction) -> bool {
    // SAFETY: WebKit supplies a live WKFrameInfo for a non-new-window navigation action.
    let has_target_frame = unsafe { action.targetFrame() }.is_some();
    if !has_target_frame {
        return false;
    }
    // SAFETY: WebKit supplies a live NSURLRequest for every WKNavigationAction.
    let request = unsafe { action.request() };
    request
        .URL()
        .is_some_and(|url| native_url_is_https_with_host(&url))
}

fn response_is_allowed(response: &WKNavigationResponse) -> bool {
    // SAFETY: WKNavigationResponse.response returns a retained response for this live callback.
    let native_response = unsafe { response.response() };
    native_response
        .URL()
        .is_some_and(|url| native_url_is_https_with_host(&url))
}

extern_protocol!(
    #[allow(clippy::missing_safety_doc)]
    /// The minimal iOS `WKNavigationDelegate` protocol surface used by this adapter.
    ///
    /// # Safety
    ///
    /// This name must identify Apple's runtime `WKNavigationDelegate` protocol, inherit
    /// `NSObjectProtocol`, and use the exact selector argument encodings declared below.
    #[name = "WKNavigationDelegate"]
    unsafe trait IosWkNavigationDelegate: NSObjectProtocol + MainThreadOnly {
        #[optional]
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        #[unsafe(method_family = none)]
        unsafe fn decide_navigation_action(
            &self,
            web_view: &NativeWkWebView,
            action: &WKNavigationAction,
            decision_handler: &block2::DynBlock<dyn Fn(NSInteger)>,
        );

        #[optional]
        #[unsafe(method(webView:decidePolicyForNavigationResponse:decisionHandler:))]
        #[unsafe(method_family = none)]
        unsafe fn decide_navigation_response(
            &self,
            web_view: &NativeWkWebView,
            response: &WKNavigationResponse,
            decision_handler: &block2::DynBlock<dyn Fn(NSInteger)>,
        );
    }
);

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    struct NavigationDelegate;

    // SAFETY: This NSObject subclass has no additional state and its callbacks are main-thread-only.
    unsafe impl NSObjectProtocol for NavigationDelegate {}

    // SAFETY: This class conforms to the runtime WKNavigationDelegate protocol, whose two
    // implemented selectors and ABI types mirror the inspected public iOS SDK header.
    #[allow(non_snake_case)]
    unsafe impl IosWkNavigationDelegate for NavigationDelegate {
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        unsafe fn decide_navigation_action(
            &self,
            _web_view: &NativeWkWebView,
            action: &WKNavigationAction,
            decision_handler: &block2::DynBlock<dyn Fn(NSInteger)>,
        ) {
            let allowed =
                catch_unwind(AssertUnwindSafe(|| action_is_allowed(action))).unwrap_or(false);
            let policy = if allowed {
                NAVIGATION_POLICY_ALLOW
            } else {
                NAVIGATION_POLICY_CANCEL
            };
            decision_handler.call((policy,));
        }

        #[unsafe(method(webView:decidePolicyForNavigationResponse:decisionHandler:))]
        unsafe fn decide_navigation_response(
            &self,
            _web_view: &NativeWkWebView,
            response: &WKNavigationResponse,
            decision_handler: &block2::DynBlock<dyn Fn(NSInteger)>,
        ) {
            let allowed =
                catch_unwind(AssertUnwindSafe(|| response_is_allowed(response))).unwrap_or(false);
            let policy = if allowed {
                NAVIGATION_POLICY_ALLOW
            } else {
                NAVIGATION_POLICY_CANCEL
            };
            decision_handler.call((policy,));
        }
    }
);

impl NavigationDelegate {
    fn new(marker: MainThreadMarker) -> Retained<Self> {
        let allocated: Allocated<Self> = marker.alloc();
        let allocated = allocated.set_ivars(());
        // SAFETY: NavigationDelegate derives directly from NSObject, and its empty ivars are set
        // before NSObject's designated initializer runs.
        unsafe { msg_send![super(allocated), init] }
    }
}

extern_class!(
    /// Private iOS declaration for the generated-binding gap in objc2-web-kit 0.3.2.
    #[unsafe(super(UIView))]
    #[thread_kind = MainThreadOnly]
    #[name = "WKWebView"]
    struct NativeWkWebView;
);

#[allow(non_snake_case)]
impl NativeWkWebView {
    extern_methods!(
        // SAFETY: Mirrors the public iOS WebKit header's designated initializer. The class is
        // main-thread-only and both the configuration and frame use the SDK-declared ABI types.
        #[unsafe(method(initWithFrame:configuration:))]
        #[unsafe(method_family = init)]
        pub unsafe fn initWithFrame_configuration(
            this: Allocated<Self>,
            frame: CGRect,
            configuration: &WKWebViewConfiguration,
        ) -> Retained<Self>;

        // SAFETY: Mirrors the public nullable `WKNavigation *` result of `loadRequest:`.
        #[unsafe(method(loadRequest:))]
        #[unsafe(method_family = none)]
        pub unsafe fn loadRequest(&self, request: &NSURLRequest) -> Option<Retained<WKNavigation>>;

        // SAFETY: Mirrors the public weak `id<WKNavigationDelegate>` property setter. The supplied
        // protocol object remains retained by IosWebView for the view's lifetime.
        #[unsafe(method(setNavigationDelegate:))]
        #[unsafe(method_family = none)]
        pub unsafe fn setNavigationDelegate(
            &self,
            navigation_delegate: Option<&ProtocolObject<dyn IosWkNavigationDelegate>>,
        );

        // SAFETY: Mirrors the public read-only BOOL property getter.
        #[unsafe(method(canGoBack))]
        #[unsafe(method_family = none)]
        pub unsafe fn canGoBack(&self) -> bool;

        // SAFETY: Mirrors the public read-only BOOL property getter.
        #[unsafe(method(canGoForward))]
        #[unsafe(method_family = none)]
        pub unsafe fn canGoForward(&self) -> bool;

        // SAFETY: Mirrors the public nullable WKNavigation return type.
        #[unsafe(method(goBack))]
        #[unsafe(method_family = none)]
        pub unsafe fn goBack(&self) -> Option<Retained<WKNavigation>>;

        // SAFETY: Mirrors the public nullable WKNavigation return type.
        #[unsafe(method(goForward))]
        #[unsafe(method_family = none)]
        pub unsafe fn goForward(&self) -> Option<Retained<WKNavigation>>;

        // SAFETY: Mirrors the public nullable WKNavigation return type.
        #[unsafe(method(reload))]
        #[unsafe(method_family = none)]
        pub unsafe fn reload(&self) -> Option<Retained<WKNavigation>>;

        // SAFETY: Mirrors the public void stopLoading selector.
        #[unsafe(method(stopLoading))]
        #[unsafe(method_family = none)]
        pub unsafe fn stopLoading(&self);
    );
}

fn unavailable() -> WebViewError {
    WebViewError::Backend(Error::new(ErrorKind::Unavailable))
}
