use core::ptr::NonNull;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use objc2::rc::autoreleasepool;
use objc2_file_provider::{NSFileProviderDomain, NSFileProviderManager};
use objc2_foundation::{NSArray, NSError};

use crate::completion::Completion;
use crate::{FileProviderQueryError, NativeFileProviderError, RegisteredDomainPresence};

pub(crate) fn start(completion: Arc<Completion>) {
    if !objc2::available!(ios = 11.0, ..) {
        completion.complete(Err(FileProviderQueryError::ApiUnavailable));
        return;
    }

    let handler = RcBlock::new(
        move |domains: NonNull<NSArray<NSFileProviderDomain>>, error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: The nullable NSError is borrowed only for this callback invocation
                    if let Some(error) = unsafe { error.as_ref() } {
                        return Err(FileProviderQueryError::Native(
                            NativeFileProviderError::new(error.domain().to_string(), error.code()),
                        ));
                    }

                    // SAFETY: Apple declares the returned domains array nonnull on success
                    let has_registered_domains = unsafe { domains.as_ref() }.count() != 0;
                    Ok(RegisteredDomainPresence::new(has_registered_domains))
                })
            }))
            .unwrap_or(Err(FileProviderQueryError::CallbackPanicked));
            completion.complete(result);
        },
    );

    // SAFETY: `handler` is a heap block owned for the call. The Objective-C API accepts an
    // escaping completion block and copies it for the asynchronous result. It may invoke the
    // block on an arbitrary queue; its only capture is `Arc<Completion>`, whose callback state
    // contains Rust-owned `Send + Sync` data and no UIKit or FileProvider object handle
    unsafe { NSFileProviderManager::getDomainsWithCompletionHandler(&handler) };
}
