use core::marker::PhantomData;
use std::rc::Rc;

use objc2_foundation::NSURL;

/// A successful, caller-owned security-scope access for one file URL.
///
/// Keep this guard alive while accessing the resource. Dropping it balances the successful
/// `startAccessingSecurityScopedResource` call exactly once. The guard is neither `Send` nor
/// `Sync`, so it cannot move to another thread after the scope starts.
#[must_use = "dropping the guard immediately relinquishes security-scoped access"]
pub struct IosSecurityScopedAccess<'url> {
    url: &'url NSURL,
    _thread_bound: PhantomData<Rc<()>>,
}

impl<'url> IosSecurityScopedAccess<'url> {
    /// Starts access to a caller-supplied security-scoped file URL.
    ///
    /// This does not present a picker, resolve a bookmark, or grant new access. The URL must
    /// already carry a scope provided by an API such as `UIDocumentPickerViewController` or by
    /// security-scoped bookmark resolution. A successful start must remain paired with this
    /// guard until all access through that scope is done.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityScopeStartError::NotFileUrl`] for a non-file URL and
    /// [`SecurityScopeStartError::StartRejected`] when Foundation returns `false`. A rejected
    /// start is not followed by a stop call.
    pub fn start(url: &'url NSURL) -> Result<Self, SecurityScopeStartError> {
        if !url.isFileURL() {
            return Err(SecurityScopeStartError::NotFileUrl);
        }

        // SAFETY: `url` is a live borrowed NSURL, and the caller is responsible for supplying a
        // URL whose security scope is already available. The returned Boolean controls whether
        // this guard owns one access reference and therefore whether Drop must balance it.
        if unsafe { url.startAccessingSecurityScopedResource() } {
            Ok(Self {
                url,
                _thread_bound: PhantomData,
            })
        } else {
            Err(SecurityScopeStartError::StartRejected)
        }
    }
}

impl Drop for IosSecurityScopedAccess<'_> {
    fn drop(&mut self) {
        // SAFETY: this guard exists only after a successful start and retains a borrow of the
        // same NSURL; Drop balances that one start reference exactly once.
        unsafe { self.url.stopAccessingSecurityScopedResource() };
    }
}

/// Reason that a security-scoped access guard could not start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityScopeStartError {
    /// The supplied URL is not a file URL.
    NotFileUrl,
    /// Foundation returned `false`; no access reference was acquired.
    StartRejected,
}
