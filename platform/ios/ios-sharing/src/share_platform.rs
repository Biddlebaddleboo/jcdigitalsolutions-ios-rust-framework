use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use dispatch2::{DispatchQueue, MainThreadBound};
use framework_core::{Availability, Error, ErrorKind};
use framework_sharing::{ShareBackend, ShareError, ShareRequest};
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Retained, Weak, autoreleasepool};
use objc2::runtime::{AnyObject, Bool};
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_core_foundation::CGRect;
use objc2_foundation::{NSArray, NSError, NSString, NSURL};
use objc2_ui_kit::{
    UIActivityViewController, UIDevice, UIModalPresentationStyle, UIUserInterfaceIdiom, UIView,
    UIViewController,
};

use crate::share_conversion::map_request;
use crate::share_operation::{
    ShareAccess, ShareCompletion, ShareOperation, ShareStartError, accept_preflight,
    map_activity_result, rejected_start,
};

/// A caller-owned iOS system-share backend bound to an explicit presentation context.
///
/// Construct, poll, and drop this backend on the main thread. The stored proof and non-send marker
/// keep safe Rust callers from moving it or its operation futures to another thread.
pub struct IosShareBackend<'ctx> {
    presenter: &'ctx UIViewController,
    source_view: &'ctx UIView,
    source_rect: CGRect,
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
    active_controller: Option<Retained<UIActivityViewController>>,
}

/// An owned, one-operation iOS share session for callback-based callers.
///
/// The session retains its UIKit presentation context and is bound to the main thread. It does not
/// own an executor or dismiss system UI when an operation is cancelled or the session is dropped.
pub struct IosShareSession {
    presenter: Retained<UIViewController>,
    source_view: Retained<UIView>,
    source_rect: CGRect,
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
    active_operation: Option<IosShareOperation>,
}

struct PreparedShare {
    activity: Retained<UIActivityViewController>,
    callback_marker: MainThreadMarker,
}

struct IosShareOperation {
    activity: Retained<UIActivityViewController>,
    completion: ShareCompletion,
}

struct ShareCallbackState {
    completion: ShareCompletion,
    activity: Weak<UIActivityViewController>,
}

impl IosShareOperation {
    fn detach(&self) {
        self.completion.detach();
        autoreleasepool(|_| clear_completion_handler(&self.activity));
    }
}

impl Drop for IosShareOperation {
    fn drop(&mut self) {
        self.detach();
    }
}

impl IosShareSession {
    /// Creates a callback session with retained presenter and popover-anchor objects.
    ///
    /// The source rectangle uses the coordinate system of `source_view`. The presenter and source
    /// view must belong to the same window when sharing starts. Construction and all later session
    /// calls and drops must occur on the main thread.
    pub fn new(
        main_thread: MainThread,
        presenter: Retained<UIViewController>,
        source_view: Retained<UIView>,
        source_rect: CGRect,
    ) -> Self {
        Self {
            presenter,
            source_view,
            source_rect,
            _main_thread: main_thread,
            _not_send: PhantomData,
            active_operation: None,
        }
    }

    /// Reports unknown availability without claiming that the presenter is ready.
    pub const fn availability(&self) -> Availability {
        Availability::Unknown
    }

    /// Starts one owned request and calls `completion` once when UIKit reports its terminal result.
    ///
    /// Input and presentation preflight errors return synchronously with the callback in
    /// `ShareStartError`, so rejected starts do not take callback ownership.
    /// A second start while an operation is active returns `ErrorKind::AlreadyExists`. Starting
    /// work occurs during this call; no future or executor is involved. UIKit has no presentation
    /// error result, so `Ok(())` means presentation was requested, not that visible UI or a terminal
    /// callback is guaranteed. Callback state is detached before its one-shot notification, and a
    /// callback panic is caught in the main-queue work item. UIKit result data goes to main by an
    /// async queue post, so `completion` runs only after `IosShareSession::start` returns
    pub fn start<F>(
        &mut self,
        request: ShareRequest,
        completion: F,
    ) -> Result<(), ShareStartError<F>>
    where
        F: FnOnce(Result<framework_sharing::ShareOutcome, ShareError>) + 'static,
    {
        let Some(main_thread) = MainThread::current() else {
            return Err(rejected_start(unavailable(), completion));
        };
        if self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active())
        {
            return Err(rejected_start(
                ShareError::Backend(Error::new(ErrorKind::AlreadyExists)),
                completion,
            ));
        }
        drop(self.active_operation.take());

        let mut backend = IosShareBackend::new(
            main_thread,
            &self.presenter,
            &self.source_view,
            self.source_rect,
        );
        let (prepared, completion) = accept_preflight(backend.prepare(request), completion)?;
        let callback = ShareCompletion::with_callback(completion);
        let activity = backend.present_prepared(prepared, callback.clone());
        self.active_operation = Some(IosShareOperation {
            activity,
            completion: callback,
        });
        Ok(())
    }

    /// Detaches the active result callback without dismissing UIKit share UI.
    ///
    /// Returns `true` only when an accepted operation was still awaiting a terminal callback.
    /// Returns `false` if no operation is active or it already completed.
    pub fn cancel(&mut self) -> bool {
        let active = self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active());
        drop(self.active_operation.take());
        active
    }
}

impl Drop for IosShareSession {
    fn drop(&mut self) {
        drop(self.active_operation.take());
    }
}

impl<'ctx> IosShareBackend<'ctx> {
    /// Creates a backend from a typed main-thread proof and caller-supplied UI context.
    ///
    /// The source rectangle uses the coordinate system of `source_view`. On iPad it is applied as
    /// the required popover anchor. The presenter and source view must remain alive for the
    /// backend's lifetime and must belong to the same window when a share starts.
    pub fn new(
        main_thread: MainThread,
        presenter: &'ctx UIViewController,
        source_view: &'ctx UIView,
        source_rect: CGRect,
    ) -> Self {
        Self {
            presenter,
            source_view,
            source_rect,
            _main_thread: main_thread,
            _not_send: PhantomData,
            active_controller: None,
        }
    }
}

impl ShareAccess for IosShareBackend<'_> {
    fn present(
        &mut self,
        request: ShareRequest,
        completion: ShareCompletion,
    ) -> Result<(), ShareError> {
        let prepared = self.prepare(request)?;
        self.present_prepared(prepared, completion);
        Ok(())
    }

    fn detach(&mut self) {
        if let Some(activity) = self.active_controller.take() {
            autoreleasepool(|_| clear_completion_handler(&activity));
        }
    }
}

impl IosShareBackend<'_> {
    fn prepare(&self, request: ShareRequest) -> Result<PreparedShare, ShareError> {
        autoreleasepool(|_| {
            let items: Vec<Retained<AnyObject>> = map_request(
                request,
                |value| Retained::<AnyObject>::from(NSString::from_str(&value)),
                |value| {
                    let value = NSString::from_str(&value);
                    let url = NSURL::URLWithString(&value).ok_or_else(invalid_input)?;
                    if url.isFileURL() {
                        return Err(invalid_input());
                    }
                    Ok(Retained::<AnyObject>::from(url))
                },
            )?;
            let activity_items: Retained<NSArray<AnyObject>> = NSArray::from_retained_slice(&items);

            self.preflight()?;
            let marker = MainThreadMarker::new().ok_or_else(unavailable)?;
            let is_ipad =
                UIDevice::currentDevice(marker).userInterfaceIdiom() == UIUserInterfaceIdiom::Pad;
            let allocation_marker = MainThreadMarker::new().ok_or_else(unavailable)?;

            // SAFETY: `activity_items` is non-empty and contains only owned NSString/NSURL
            // instances, both valid NSObject-backed activity items. No custom activities are
            // supplied, so the generic NSArray element invariant required by UIKit holds.
            let activity = unsafe {
                UIActivityViewController::initWithActivityItems_applicationActivities(
                    UIActivityViewController::alloc(allocation_marker),
                    &activity_items,
                    None,
                )
            };

            if is_ipad {
                activity.setModalPresentationStyle(UIModalPresentationStyle::Popover);
                let popover = activity
                    .popoverPresentationController()
                    .ok_or_else(unavailable)?;
                popover.setSourceView(Some(self.source_view));
                popover.setSourceRect(self.source_rect);
            }

            let callback_marker = MainThreadMarker::new().ok_or_else(unavailable)?;
            Ok(PreparedShare {
                activity,
                callback_marker,
            })
        })
    }

    fn present_prepared(
        &mut self,
        prepared: PreparedShare,
        completion: ShareCompletion,
    ) -> Retained<UIActivityViewController> {
        autoreleasepool(|_| {
            let PreparedShare {
                activity,
                callback_marker,
            } = prepared;
            let callback_state = Arc::new(MainThreadBound::new(
                ShareCallbackState {
                    completion: completion.clone(),
                    activity: Weak::from(&activity),
                },
                callback_marker,
            ));
            let callback_claimed = Arc::new(AtomicBool::new(false));
            let handler = block2::RcBlock::new(
                move |_activity_type: *mut NSString,
                      completed: Bool,
                      _returned_items: *mut NSArray,
                      activity_error: *mut NSError| {
                    let _ = ios_runtime::ffi::catch_unwind(|| {
                        if callback_claimed.swap(true, Ordering::AcqRel) {
                            return;
                        }
                        let native_result = ios_runtime::ffi::catch_unwind(|| {
                            autoreleasepool(|_| {
                                // SAFETY: UIKit documents this callback argument as either null or a
                                // valid NSError pointer for the duration of the callback.
                                let native_error = unsafe { activity_error.as_ref() };
                                let native_code =
                                    native_error.and_then(|error| i32::try_from(error.code()).ok());
                                (completed.as_bool(), native_error.is_some(), native_code)
                            })
                        });
                        let callback_state = Arc::clone(&callback_state);
                        DispatchQueue::main().exec_async(move || {
                            let _ = ios_runtime::ffi::catch_unwind(|| {
                                // SAFETY: this closure runs on DispatchQueue::main, so this marker is valid
                                let marker = unsafe { MainThreadMarker::new_unchecked() };
                                let (completion, activity) = {
                                    let state = callback_state.get(marker);
                                    (state.completion.clone(), state.activity.clone())
                                };
                                let result = match native_result {
                                    Ok((completed, has_error, native_code)) => {
                                        map_activity_result(completed, has_error, native_code)
                                    }
                                    Err(_) => {
                                        Err(ShareError::Backend(Error::new(ErrorKind::Internal)))
                                    }
                                };
                                completion.complete_with(result, || {
                                    clear_weak_completion_handler(&activity)
                                });
                            });
                        });
                    });
                },
            );

            // SAFETY: `handler` is a valid Blocks closure with the generated UIKit signature.
            // UIKit copies this property; `RcBlock` remains alive through the setter call.
            unsafe { activity.setCompletionWithItemsHandler(block2::RcBlock::as_ptr(&handler)) };
            self.active_controller = Some(activity.clone());
            self.presenter
                .presentViewController_animated_completion(&activity, true, None);
            activity
        })
    }

    fn preflight(&self) -> Result<(), ShareError> {
        if self.presenter.isBeingPresented()
            || self.presenter.isBeingDismissed()
            || self.presenter.presentedViewController().is_some()
        {
            return Err(unavailable());
        }
        let presenter_view = self.presenter.viewIfLoaded().ok_or_else(unavailable)?;
        let presenter_window = presenter_view.window().ok_or_else(unavailable)?;
        let source_window = self.source_view.window().ok_or_else(unavailable)?;
        if !core::ptr::eq(
            Retained::as_ptr(&presenter_window),
            Retained::as_ptr(&source_window),
        ) {
            return Err(invalid_input());
        }
        Ok(())
    }
}

fn clear_completion_handler(activity: &UIActivityViewController) {
    // SAFETY: Callers run on the main thread. UIKit accepts null to clear the copied completion
    // handler, detaching callback state without dismissing the presented controller.
    unsafe { activity.setCompletionWithItemsHandler(core::ptr::null_mut()) };
}

fn clear_weak_completion_handler(activity: &Weak<UIActivityViewController>) {
    if let Some(activity) = activity.load() {
        clear_completion_handler(&activity);
    }
}

impl<'ctx> ShareBackend for IosShareBackend<'ctx> {
    fn availability(&self) -> Availability {
        Availability::Unknown
    }

    type ShareFuture<'a>
        = IosShareFuture<'a, 'ctx>
    where
        Self: 'a;

    fn share<'a>(&'a mut self, request: ShareRequest) -> Self::ShareFuture<'a> {
        IosShareFuture {
            inner: ShareOperation::new(self, request),
        }
    }
}

/// A non-send future that presents one system-share request on its first poll.
pub struct IosShareFuture<'a, 'ctx> {
    inner: ShareOperation<'a, IosShareBackend<'ctx>>,
}

impl Future for IosShareFuture<'_, '_> {
    type Output = Result<framework_sharing::ShareOutcome, ShareError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(context)
    }
}

fn invalid_input() -> ShareError {
    ShareError::Backend(Error::new(ErrorKind::InvalidInput))
}

fn unavailable() -> ShareError {
    ShareError::Backend(Error::new(ErrorKind::Unavailable))
}
