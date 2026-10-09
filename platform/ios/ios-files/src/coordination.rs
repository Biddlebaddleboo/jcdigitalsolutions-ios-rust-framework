use core::marker::PhantomData;
use core::ptr::NonNull;
use std::cell::RefCell;
use std::rc::Rc;

use block2::RcBlock;
use framework_core::ErrorKind;
use framework_files::FileError;
use ios_runtime::ffi::catch_unwind;
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::{
    NSError, NSFileCoordinator, NSFileCoordinatorReadingOptions, NSFileCoordinatorWritingOptions,
    NSURL,
};

use super::{backend_error, foundation_error};

/// Synchronous coordination for caller-supplied file URLs.
///
/// This value owns one Foundation coordinator with no file presenter. It is bound to the thread
/// where it is created and can be used for coordinated reads and writes without a UIKit main
/// thread requirement.
pub struct IosFileCoordinator {
    native: Retained<NSFileCoordinator>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl IosFileCoordinator {
    /// Creates a coordinator with no registered file presenter.
    pub fn new() -> Self {
        Self {
            native: NSFileCoordinator::new(),
            _thread_bound: PhantomData,
        }
    }

    /// Runs a synchronous coordinated read of a caller-supplied file URL.
    ///
    /// The accessor receives Foundation's coordinated URL, which may differ from `url`. Use that
    /// URL only during the accessor. This method blocks the calling thread until the accessor
    /// returns or Foundation reports a coordination error; it starts no task or executor. It
    /// borrows `self` mutably to serialize calls through this coordinator. Do not begin another
    /// coordinated operation for the same URL from the accessor, or block on work that needs the
    /// same file.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` for a non-file URL. If Foundation reports an `NSError`, it takes
    /// precedence and its code is preserved when it fits the portable 32-bit platform-code field;
    /// otherwise the accessor's `FileError` is returned as supplied. The portable error does not
    /// retain the Foundation error domain or user info. If Foundation returns without an error
    /// and does not invoke the accessor, the method returns `Internal`. An accessor panic is caught
    /// and mapped to `Internal`; the panic hook still runs, and `panic=abort` remains fatal.
    pub fn coordinate_read<R, F>(
        &mut self,
        url: &NSURL,
        options: NSFileCoordinatorReadingOptions,
        accessor: F,
    ) -> Result<R, FileError>
    where
        F: FnOnce(&NSURL) -> Result<R, FileError>,
    {
        if !url.isFileURL() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        autoreleasepool(|_| {
            let action = Rc::new(RefCell::new(Some(accessor)));
            let output = Rc::new(RefCell::new(None));
            let action_for_block = Rc::clone(&action);
            let output_for_block = Rc::clone(&output);
            let reader = RcBlock::new(move |coordinated_url: NonNull<NSURL>| {
                let accessor = match action_for_block.try_borrow_mut() {
                    Ok(mut action) => action.take(),
                    Err(_) => None,
                };
                let Some(accessor) = accessor else {
                    return;
                };
                // SAFETY: Foundation passes a non-null URL valid for the synchronous accessor.
                // The reference is passed only to `accessor` and cannot escape its higher-ranked
                // borrowed argument.
                let coordinated_url = unsafe { coordinated_url.as_ref() };
                let result = match catch_unwind(|| accessor(coordinated_url)) {
                    Ok(result) => result,
                    Err(_) => Err(backend_error(ErrorKind::Internal, None)),
                };
                if let Ok(mut output) = output_for_block.try_borrow_mut() {
                    *output = Some(result);
                }
            });
            let mut native_error: Option<Retained<NSError>> = None;
            self.native
                .coordinateReadingItemAtURL_options_error_byAccessor(
                    url,
                    options,
                    Some(&mut native_error),
                    &reader,
                );
            drop(reader);

            if let Some(error) = native_error {
                return Err(foundation_error(&error));
            }
            output
                .borrow_mut()
                .take()
                .unwrap_or_else(|| Err(backend_error(ErrorKind::Internal, None)))
        })
    }

    /// Runs a synchronous coordinated write of a caller-supplied file URL.
    ///
    /// The accessor receives Foundation's coordinated URL, which may differ from `url`. Use that
    /// URL only during the accessor. This method blocks the calling thread until the accessor
    /// returns or Foundation reports a coordination error; it starts no task or executor. It
    /// borrows `self` mutably to serialize calls through this coordinator. Do not begin another
    /// coordinated operation for the same URL from the accessor, or block on work that needs the
    /// same file.
    ///
    /// The caller selects Foundation's standard write options. This method does not infer whether
    /// the accessor replaces, moves, or deletes the item, and it does not perform file I/O itself.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` for a non-file URL. If Foundation reports an `NSError`, it takes
    /// precedence and its code is preserved when it fits the portable 32-bit platform-code field;
    /// otherwise the accessor's `FileError` is returned as supplied. The portable error does not
    /// retain the Foundation error domain or user info. If Foundation returns without an error
    /// and does not invoke the accessor, the method returns `Internal`. An accessor panic is caught
    /// and mapped to `Internal`; the panic hook still runs, and `panic=abort` remains fatal.
    pub fn coordinate_write<R, F>(
        &mut self,
        url: &NSURL,
        options: NSFileCoordinatorWritingOptions,
        accessor: F,
    ) -> Result<R, FileError>
    where
        F: FnOnce(&NSURL) -> Result<R, FileError>,
    {
        if !url.isFileURL() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        autoreleasepool(|_| {
            let action = Rc::new(RefCell::new(Some(accessor)));
            let output = Rc::new(RefCell::new(None));
            let action_for_block = Rc::clone(&action);
            let output_for_block = Rc::clone(&output);
            let writer = RcBlock::new(move |coordinated_url: NonNull<NSURL>| {
                let accessor = match action_for_block.try_borrow_mut() {
                    Ok(mut action) => action.take(),
                    Err(_) => None,
                };
                let Some(accessor) = accessor else {
                    return;
                };
                // SAFETY: Foundation passes a non-null URL valid for the synchronous accessor.
                // The reference is passed only to `accessor` and cannot escape its higher-ranked
                // borrowed argument.
                let coordinated_url = unsafe { coordinated_url.as_ref() };
                let result = match catch_unwind(|| accessor(coordinated_url)) {
                    Ok(result) => result,
                    Err(_) => Err(backend_error(ErrorKind::Internal, None)),
                };
                if let Ok(mut output) = output_for_block.try_borrow_mut() {
                    *output = Some(result);
                }
            });
            let mut native_error: Option<Retained<NSError>> = None;
            self.native
                .coordinateWritingItemAtURL_options_error_byAccessor(
                    url,
                    options,
                    Some(&mut native_error),
                    &writer,
                );
            drop(writer);

            if let Some(error) = native_error {
                return Err(foundation_error(&error));
            }
            output
                .borrow_mut()
                .take()
                .unwrap_or_else(|| Err(backend_error(ErrorKind::Internal, None)))
        })
    }
}

impl Default for IosFileCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
