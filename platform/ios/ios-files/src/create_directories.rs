use std::{
    ffi::{CStr, CString},
    fs::File,
    io,
    os::fd::{AsRawFd, FromRawFd},
};

pub(crate) fn create_directories_at<E>(
    root: &File,
    parts: &[CString],
    mut ensure_name_is_preserved: impl FnMut(&File, &CStr) -> Result<(), E>,
    mut map_error: impl FnMut(io::Error) -> E,
) -> Result<(), E> {
    if parts.is_empty() {
        return Err(map_error(io::Error::from_raw_os_error(libc::EINVAL)));
    }

    let mut directory = root.try_clone().map_err(&mut map_error)?;
    for part in parts {
        ensure_name_is_preserved(&directory, part)?;
        loop {
            // SAFETY: `directory` is open and `part` is one validated NUL-terminated component.
            let result = unsafe { libc::mkdirat(directory.as_raw_fd(), part.as_ptr(), 0o700) };
            if result == 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(map_error(error));
            }
            break;
        }

        let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
        let fd = loop {
            // SAFETY: `directory` is open and `part` is one validated component. These flags
            // reject symlinks and non-directory entries, including an entry raced after mkdirat.
            let fd = unsafe { libc::openat(directory.as_raw_fd(), part.as_ptr(), flags) };
            if fd >= 0 {
                break fd;
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(map_error(error));
        };
        // SAFETY: `openat` returned a new owned descriptor.
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Ok(())
}
