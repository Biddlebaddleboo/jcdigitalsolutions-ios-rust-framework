use alloc::rc::Rc;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_core::{Availability, Error, ErrorKind};
use framework_sharing::{ClipboardBackend, ClipboardError};
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSDictionary, NSString, NSUTF8StringEncoding};
use objc2_ui_kit::{UIImage, UIPasteboard};

use crate::conversion::decode_utf8;
use crate::operation::{Clear, ClipboardAccess, Deferred, Read, Write};

/// A caller-owned backend for plain-text operations on the iOS general pasteboard.
///
/// Construct, poll, and drop this backend on the main thread. The retained `MainThread` proof and
/// a non-send marker prevent safe Rust code from moving the backend or its operation futures to a
/// worker thread. Operations make synchronous UIKit calls on their first poll.
pub struct IosClipboardBackend {
    pasteboard: Retained<UIPasteboard>,
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
}

impl IosClipboardBackend {
    /// Retains the systemwide general pasteboard for use with a typed main-thread proof.
    pub fn new(main_thread: MainThread) -> Self {
        Self {
            pasteboard: UIPasteboard::generalPasteboard(),
            _main_thread: main_thread,
            _not_send: PhantomData,
        }
    }

    /// Returns whether the general pasteboard currently has a plain-text representation.
    ///
    /// This caller-invoked snapshot checks only `UIPasteboard.hasStrings`; it does not load or
    /// copy string contents. The shared pasteboard may change immediately after this call, and a
    /// `true` result does not guarantee that a later read succeeds or has user approval. Apple
    /// documents this type-presence query among the APIs that avoid user notifications and alerts
    /// when the system has not established user intent to access pasteboard data. Call it on the
    /// same main thread used to construct this backend.
    pub fn has_plain_text(&self) -> bool {
        // SAFETY: The backend retains the main-thread proof and is !Send + !Sync, so safe Rust
        // cannot move this UIKit access to another thread.
        unsafe { self.pasteboard.hasStrings() }
    }

    /// Returns whether the general pasteboard currently contains image data.
    ///
    /// This type-presence query maps to `UIPasteboard.hasImages` and does not fetch image content.
    /// The shared pasteboard may change immediately after this call; this result does not prove
    /// permission or that a later content read will return an image. Call it on the backend's main
    /// thread.
    pub fn has_image(&self) -> bool {
        // SAFETY: The backend retains the main-thread proof and is !Send + !Sync, so safe Rust
        // cannot move this UIKit access to another thread.
        unsafe { self.pasteboard.hasImages() }
    }

    /// Returns the first pasteboard item's image, if present.
    ///
    /// The method checks `UIPasteboard.hasImages` before reading `UIPasteboard.image`; the latter
    /// property returns only the first item's image, so it may return `None` when a later item is
    /// the only item with image data. Reading image content from the general pasteboard may trigger
    /// system privacy UI. Call it on the backend's main thread.
    pub fn read_image(&self) -> Option<Retained<UIImage>> {
        autoreleasepool(|_| {
            // SAFETY: The backend retains the main-thread proof and is !Send + !Sync. UIKit's
            // type-presence query does not fetch image content.
            if !unsafe { self.pasteboard.hasImages() } {
                return None;
            }
            // SAFETY: The same main-thread invariant applies. The getter returns an owned image;
            // the pasteboard can change between the presence query and this content read.
            unsafe { self.pasteboard.image() }
        })
    }

    /// Replaces all general-pasteboard items with the supplied image.
    ///
    /// UIKit copies the image when setting `UIPasteboard.image`. This replaces existing strings,
    /// URLs, and other item representations. Call it on the backend's main thread.
    pub fn write_image(&mut self, image: &UIImage) {
        autoreleasepool(|_| {
            // SAFETY: The backend retains the main-thread proof and is !Send + !Sync, so safe Rust
            // cannot move this UIKit access to another thread. UIKit copies the assigned image.
            unsafe { self.pasteboard.setImage(Some(image)) };
        });
    }
}

impl ClipboardAccess for IosClipboardBackend {
    fn read(&mut self) -> Result<Option<alloc::string::String>, ClipboardError> {
        autoreleasepool(|_| {
            // SAFETY: The backend is constructed with `MainThread`, is `!Send`, and this call is
            // synchronous on that same main thread. The UIKit property is a type-presence query.
            if !unsafe { self.pasteboard.hasStrings() } {
                return Ok(None);
            }
            // SAFETY: The same main-thread invariant applies. The array is retained before the
            // autorelease pool ends; another app may race the preceding type-presence query.
            let Some(values) = (unsafe { self.pasteboard.strings() }) else {
                return Err(backend_error(ErrorKind::Platform));
            };
            let Some(value) = values.firstObject() else {
                return Err(backend_error(ErrorKind::Platform));
            };
            let Some(data) = value.dataUsingEncoding(NSUTF8StringEncoding) else {
                return Err(backend_error(ErrorKind::InvalidInput));
            };
            decode_utf8(data.to_vec()).map(Some)
        })
    }

    fn write(&mut self, text: &str) -> Result<(), ClipboardError> {
        autoreleasepool(|_| {
            let value = NSString::from_str(text);
            // SAFETY: The backend is constructed with `MainThread`, is `!Send`, and this call is
            // synchronous on that same main thread. UIKit copies the assigned string value.
            unsafe { self.pasteboard.setString(Some(&value)) };
            Ok(())
        })
    }

    fn clear(&mut self) -> Result<(), ClipboardError> {
        autoreleasepool(|_| {
            let items: Retained<NSArray<NSDictionary<NSString, AnyObject>>> =
                NSArray::from_slice(&[]);
            // SAFETY: The backend is constructed with `MainThread`, is `!Send`, and this call is
            // synchronous on that same main thread. The empty array has the required item type;
            // assigning it replaces all pasteboard items.
            unsafe { self.pasteboard.setItems(&items) };
            Ok(())
        })
    }
}

impl ClipboardBackend for IosClipboardBackend {
    fn availability(&self) -> Availability {
        // UIKit exposes no query for current programmatic-read usability or approval. `Unknown`
        // does not mean that permission is required.
        Availability::Unknown
    }

    type ReadFuture<'a>
        = IosClipboardReadFuture<'a>
    where
        Self: 'a;

    fn read<'a>(&'a mut self) -> Self::ReadFuture<'a> {
        IosClipboardReadFuture {
            inner: Deferred::new(self, Read),
        }
    }

    type WriteFuture<'a>
        = IosClipboardWriteFuture<'a>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, text: &'a str) -> Self::WriteFuture<'a> {
        IosClipboardWriteFuture {
            inner: Deferred::new(self, Write(text)),
        }
    }

    type ClearFuture<'a>
        = IosClipboardClearFuture<'a>
    where
        Self: 'a;

    fn clear<'a>(&'a mut self) -> Self::ClearFuture<'a> {
        IosClipboardClearFuture {
            inner: Deferred::new(self, Clear),
        }
    }
}

/// A non-send future that reads one owned plain-text value on first poll.
pub struct IosClipboardReadFuture<'a> {
    inner: Deferred<'a, IosClipboardBackend, Read>,
}

impl Future for IosClipboardReadFuture<'_> {
    type Output = Result<Option<alloc::string::String>, ClipboardError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(context)
    }
}

/// A non-send future that writes borrowed plain text on first poll.
pub struct IosClipboardWriteFuture<'a> {
    inner: Deferred<'a, IosClipboardBackend, Write<'a>>,
}

impl Future for IosClipboardWriteFuture<'_> {
    type Output = Result<(), ClipboardError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(context)
    }
}

/// A non-send future that clears the general pasteboard on first poll.
pub struct IosClipboardClearFuture<'a> {
    inner: Deferred<'a, IosClipboardBackend, Clear>,
}

impl Future for IosClipboardClearFuture<'_> {
    type Output = Result<(), ClipboardError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(context)
    }
}

fn backend_error(kind: ErrorKind) -> ClipboardError {
    ClipboardError::Backend(Error::new(kind))
}
