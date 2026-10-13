use core::ptr::NonNull;
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_files::FileError;
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{
    NSCocoaErrorDomain, NSError, NSFeatureUnsupportedError, NSFileNoSuchFileError,
    NSFileReadNoPermissionError, NSFileReadNoSuchFileError, NSFileWriteNoPermissionError,
    NSFileWriteVolumeReadOnlyError, NSNumber, NSPOSIXErrorDomain, NSURL,
    NSURLIsExcludedFromBackupKey, NSUnderlyingErrorKey,
};
use std::{
    ffi::{CStr, c_char},
    fs::File,
    io,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::MetadataExt,
    },
};

const DARWIN_MAXPATHLEN: usize = 1024;

pub(crate) fn set_at(
    root: &File,
    parent: &File,
    leaf: &CStr,
    excluded: bool,
) -> Result<(), FileError> {
    let (target, is_directory) = open_target(parent, leaf)?;
    let url = resolved_file_url(root, &target, is_directory)?;
    objc2::rc::autoreleasepool(|_| {
        let key = unsafe { NSURLIsExcludedFromBackupKey };
        let value = NSNumber::new_bool(excluded);
        // SAFETY: Apple's NSURL resource key is a writable Boolean NSNumber value.
        unsafe { url.setResourceValue_forKey_error(Some(&value), key) }
            .map_err(|error| resource_error(&error))?;

        let verified_url = resolved_file_url(root, &target, is_directory)?;
        let actual = excluded_value(&verified_url)?;
        verify_set(excluded, actual)
    })
}

pub(crate) fn get_at(root: &File, parent: &File, leaf: &CStr) -> Result<bool, FileError> {
    let (target, is_directory) = open_target(parent, leaf)?;
    let url = resolved_file_url(root, &target, is_directory)?;
    objc2::rc::autoreleasepool(|_| excluded_value(&url))
}

fn open_target(parent: &File, leaf: &CStr) -> Result<(File, bool), FileError> {
    let leaf_bytes = leaf.to_bytes();
    if leaf_bytes.is_empty()
        || leaf_bytes == b"."
        || leaf_bytes == b".."
        || leaf_bytes.contains(&b'/')
        || leaf_bytes.contains(&b'\\')
    {
        return Err(FileError::InvalidPath);
    }
    // SAFETY: `parent` is a validated open directory and `leaf` is one validated component.
    // O_NOFOLLOW rejects a final symlink and O_NONBLOCK avoids waiting on a replaced special file.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    // SAFETY: `openat` returned a newly owned descriptor.
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file.metadata().map_err(file_error)?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    Ok((file, metadata.is_dir()))
}

fn resolved_file_url(
    root: &File,
    target: &File,
    is_directory: bool,
) -> Result<Retained<NSURL>, FileError> {
    let root_path = descriptor_path(root)?;
    let mut target_path = descriptor_path(target)?;
    // SAFETY: `descriptor_path` fills a NUL-terminated pathname or returns an error.
    let target_path_cstr = unsafe { CStr::from_ptr(target_path.as_ptr()) };
    if !is_beneath(&root_path, target_path_cstr) {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }

    objc2::rc::autoreleasepool(|_| {
        let path_url = unsafe {
            NSURL::fileURLWithFileSystemRepresentation_isDirectory_relativeToURL(
                NonNull::from(&mut target_path[0]),
                is_directory,
                None,
            )
        };
        let reference_url = path_url
            .fileReferenceURL()
            .ok_or_else(|| backend_error(ErrorKind::Unsupported, None))?;
        if !reference_url.isFileReferenceURL() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        let reference_path_url = reference_url
            .filePathURL()
            .ok_or_else(|| backend_error(ErrorKind::Unavailable, None))?;
        let reference_path = reference_path_url.fileSystemRepresentation();
        // SAFETY: NSURL owns the file-system representation while `reference_path_url` is live.
        let reference_path = unsafe { CStr::from_ptr(reference_path.as_ptr()) };
        if !is_beneath(&root_path, reference_path) {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        // SAFETY: `reference_path` is an absolute NSURL file-system path. O_NOFOLLOW rejects a
        // final symlink; the device/inode comparison below proves this URL still names `target`.
        let fd = unsafe {
            libc::open(
                reference_path.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `open` returned a newly owned descriptor.
        let resolved = unsafe { File::from_raw_fd(fd) };
        let target_metadata = target.metadata().map_err(file_error)?;
        let resolved_metadata = resolved.metadata().map_err(file_error)?;
        if target_metadata.dev() != resolved_metadata.dev()
            || target_metadata.ino() != resolved_metadata.ino()
            || target_metadata.is_dir() != resolved_metadata.is_dir()
        {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(reference_path_url)
    })
}

fn descriptor_path(file: &File) -> Result<[c_char; DARWIN_MAXPATHLEN], FileError> {
    let mut path = [0; DARWIN_MAXPATHLEN];
    // SAFETY: F_GETPATH is public Darwin fcntl; `path` meets its MAXPATHLEN output contract.
    let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETPATH, path.as_mut_ptr()) };
    if result < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    Ok(path)
}

fn is_beneath(root: &[c_char], path: &CStr) -> bool {
    // SAFETY: `descriptor_path` fills a NUL-terminated pathname or returns an error.
    let root = unsafe { CStr::from_ptr(root.as_ptr()) }.to_bytes();
    let path = path.to_bytes();
    if root == b"/" {
        return path.starts_with(b"/");
    }
    path.starts_with(root) && path.get(root.len()) == Some(&b'/')
}

fn excluded_value(url: &NSURL) -> Result<bool, FileError> {
    let key = unsafe { NSURLIsExcludedFromBackupKey };
    url.removeCachedResourceValueForKey(key);
    let mut value = None;
    // SAFETY: Apple's NSURL resource key returns a Boolean NSNumber when supported.
    unsafe { url.getResourceValue_forKey_error(&mut value, key) }
        .map_err(|error| resource_error(&error))?;
    decode_value(value)
}

pub(crate) fn decode_value(value: Option<Retained<AnyObject>>) -> Result<bool, FileError> {
    let Some(value) = value else {
        return Ok(false);
    };
    value
        .downcast::<NSNumber>()
        .ok()
        .map(|value| value.as_bool())
        .ok_or_else(|| backend_error(ErrorKind::Unsupported, None))
}

pub(crate) fn verify_set(requested: bool, actual: bool) -> Result<(), FileError> {
    if requested == actual {
        Ok(())
    } else {
        Err(backend_error(ErrorKind::Unsupported, None))
    }
}

fn resource_error(error: &NSError) -> FileError {
    if let Some(underlying) = error
        .userInfo()
        .objectForKey(unsafe { NSUnderlyingErrorKey })
        .and_then(|value| value.downcast::<NSError>().ok())
    {
        let mapped = resource_error_code(&underlying);
        if mapped.kind() != ErrorKind::Platform {
            return mapped;
        }
    }
    resource_error_code(error)
}

fn resource_error_code(error: &NSError) -> FileError {
    let code = i32::try_from(error.code()).ok();
    let domain = error.domain();
    let kind = if domain.isEqualToString(unsafe { NSPOSIXErrorDomain }) {
        match code {
            Some(value) if value == libc::EACCES || value == libc::EPERM => {
                ErrorKind::PermissionDenied
            }
            Some(value) if value == libc::ENOENT => ErrorKind::NotFound,
            Some(value) if value == libc::ENOTSUP || value == libc::EOPNOTSUPP => {
                ErrorKind::Unsupported
            }
            Some(value)
                if value == libc::EINVAL
                    || value == libc::ENOTDIR
                    || value == libc::ELOOP
                    || value == libc::ENAMETOOLONG =>
            {
                ErrorKind::InvalidInput
            }
            _ => ErrorKind::Platform,
        }
    } else if domain.isEqualToString(unsafe { NSCocoaErrorDomain }) {
        match code {
            Some(value)
                if value == NSFileReadNoPermissionError as i32
                    || value == NSFileWriteNoPermissionError as i32 =>
            {
                ErrorKind::PermissionDenied
            }
            Some(value)
                if value == NSFileNoSuchFileError as i32
                    || value == NSFileReadNoSuchFileError as i32 =>
            {
                ErrorKind::NotFound
            }
            Some(value) if value == NSFileWriteVolumeReadOnlyError as i32 => {
                ErrorKind::PermissionDenied
            }
            Some(value) if value == NSFeatureUnsupportedError as i32 => ErrorKind::Unsupported,
            _ => ErrorKind::Platform,
        }
    } else {
        ErrorKind::Platform
    };
    backend_error(kind, code)
}

fn file_error(error: io::Error) -> FileError {
    let code = error.raw_os_error();
    let kind = match code {
        Some(libc::ENOENT) => ErrorKind::NotFound,
        Some(value) if value == libc::EACCES || value == libc::EPERM => ErrorKind::PermissionDenied,
        Some(value) if value == libc::ENOTSUP || value == libc::EOPNOTSUPP => {
            ErrorKind::Unsupported
        }
        Some(value)
            if value == libc::EINVAL
                || value == libc::ENOTDIR
                || value == libc::EISDIR
                || value == libc::ELOOP
                || value == libc::ENAMETOOLONG =>
        {
            ErrorKind::InvalidInput
        }
        _ => ErrorKind::Platform,
    };
    backend_error(kind, code)
}

fn backend_error(kind: ErrorKind, code: Option<i32>) -> FileError {
    let error = match code.and_then(PlatformErrorCode::new) {
        Some(code) => Error::new(kind).with_platform_code(code),
        None => Error::new(kind),
    };
    FileError::Backend(error)
}
