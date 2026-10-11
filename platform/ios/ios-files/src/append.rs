use std::{
    ffi::CStr,
    fs::File,
    io,
    os::fd::{AsRawFd, FromRawFd},
};

const OPEN_ATTEMPTS: usize = 8;

/// Appends one complete slice through one descriptor-relative write syscall.
///
/// The single syscall keeps successful concurrent append calls contiguous. A short write returns
/// `WriteZero`; the accepted prefix, if any, cannot be rolled back. An empty slice does not open
/// or create the target.
pub(crate) fn append_at(parent: &File, leaf: &CStr, bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }

    let file = open_append_file(parent, leaf)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "append target is not a regular file",
        ));
    }

    loop {
        // SAFETY: the descriptor is open for writing and `bytes` remains readable for its length.
        let written = unsafe {
            libc::write(
                file.as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len() as libc::size_t,
            )
        };
        if written < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(error);
        }
        if written as usize != bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "append write was short",
            ));
        }
        return Ok(());
    }
}

fn open_append_file(parent: &File, leaf: &CStr) -> io::Result<File> {
    let flags =
        libc::O_WRONLY | libc::O_APPEND | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK;
    let mut last_error = None;
    for _ in 0..OPEN_ATTEMPTS {
        // SAFETY: `parent` is open and `leaf` is one validated NUL-terminated component.
        // `O_NOFOLLOW` rejects a final symlink and `O_NONBLOCK` avoids waiting on a special file.
        let fd = unsafe { libc::openat(parent.as_raw_fd(), leaf.as_ptr(), flags) };
        if fd >= 0 {
            // SAFETY: `openat` returned a new owned descriptor.
            return Ok(unsafe { File::from_raw_fd(fd) });
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ENOENT) {
            return Err(error);
        }

        // Exclusive creation makes concurrent first appends converge on one new regular file.
        // Other callers that race the create receive `EEXIST` and retry the existing-file open.
        // SAFETY: `parent` is open and `leaf` is one validated NUL-terminated component.
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                flags | libc::O_CREAT | libc::O_EXCL,
                0o600 as libc::c_uint,
            )
        };
        if fd >= 0 {
            // SAFETY: `openat` returned a new owned descriptor.
            return Ok(unsafe { File::from_raw_fd(fd) });
        }
        let error = io::Error::last_os_error();
        if !matches!(error.raw_os_error(), Some(libc::EEXIST | libc::ENOENT)) {
            return Err(error);
        }
        last_error = Some(error);
    }
    Err(last_error.unwrap_or_else(|| io::Error::from_raw_os_error(libc::ENOENT)))
}
