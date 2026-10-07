#![cfg(target_os = "ios")]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
//! iOS sandbox file backend for the portable `framework-files` contract.

use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use framework_files::{
    AppDirectory, AppPath, DirectoryEntry, FileBackend, FileError, FileKind, FileWriteMode,
    WriteAtomicity, WriteOptions, WriteOutcome,
};
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::{
    NSError, NSFileManager, NSNumber, NSSearchPathDirectory, NSSearchPathDomainMask, NSURL,
    NSURLResourceKey, NSURLVolumeSupportsExclusiveRenamingKey, NSURLVolumeSupportsSwapRenamingKey,
};
use std::{
    ffi::{CStr, CString},
    fs::File,
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
    path::PathBuf,
};

/// A caller-owned set of open iOS application-sandbox directory roots.
///
/// File operations are synchronous and may block. Each operation is relative to an open root
/// descriptor and rejects symbolic links in every traversed component.
pub struct IosFiles {
    documents: File,
    caches: File,
    temporary: File,
    application_support: File,
    rename_exclusive: [bool; 4],
    rename_swap: [bool; 4],
    next_temporary_id: u64,
}

impl IosFiles {
    /// Resolves and opens the app's Documents, Caches, Temporary, and Application Support roots.
    pub fn new() -> Result<Self, FileError> {
        let document_url = directory_url(NSSearchPathDirectory::DocumentDirectory)?;
        let caches_url = directory_url(NSSearchPathDirectory::CachesDirectory)?;
        let temporary_url = temporary_url()?;
        let support_url = directory_url(NSSearchPathDirectory::ApplicationSupportDirectory)?;
        let documents = open_root(&document_url)?;
        let caches = open_root(&caches_url)?;
        let temporary = open_root(&temporary_url)?;
        let application_support = open_root(&support_url)?;
        let urls = [&document_url, &caches_url, &temporary_url, &support_url];
        // SAFETY: these Foundation exports are immutable NSURL resource-key constants.
        let exclusive_key = unsafe { NSURLVolumeSupportsExclusiveRenamingKey };
        // SAFETY: these Foundation exports are immutable NSURL resource-key constants.
        let swap_key = unsafe { NSURLVolumeSupportsSwapRenamingKey };
        let rename_exclusive = urls.map(|url| volume_supports(url, exclusive_key));
        let rename_swap = urls.map(|url| volume_supports(url, swap_key));
        Ok(Self {
            documents,
            caches,
            temporary,
            application_support,
            rename_exclusive,
            rename_swap,
            next_temporary_id: 0,
        })
    }

    fn root(&self, directory: AppDirectory) -> Result<&File, FileError> {
        match directory {
            AppDirectory::Documents => Ok(&self.documents),
            AppDirectory::Caches => Ok(&self.caches),
            AppDirectory::Temporary => Ok(&self.temporary),
            AppDirectory::ApplicationSupport => Ok(&self.application_support),
            _ => Err(backend_error(ErrorKind::Unsupported, None)),
        }
    }

    fn volume_supports_exclusive_rename(&self, directory: AppDirectory) -> bool {
        match directory {
            AppDirectory::Documents => self.rename_exclusive[0],
            AppDirectory::Caches => self.rename_exclusive[1],
            AppDirectory::Temporary => self.rename_exclusive[2],
            AppDirectory::ApplicationSupport => self.rename_exclusive[3],
            _ => false,
        }
    }

    fn volume_supports_swap_rename(&self, directory: AppDirectory) -> bool {
        match directory {
            AppDirectory::Documents => self.rename_swap[0],
            AppDirectory::Caches => self.rename_swap[1],
            AppDirectory::Temporary => self.rename_swap[2],
            AppDirectory::ApplicationSupport => self.rename_swap[3],
            _ => false,
        }
    }

    fn temporary_file(&mut self, parent: &File) -> Result<(CString, File), FileError> {
        for _ in 0..32 {
            let id = self.next_temporary_id;
            self.next_temporary_id = self.next_temporary_id.wrapping_add(1);
            let name = CString::new(format!(".ios-files-{}-{id}", std::process::id()))
                .map_err(|_| FileError::InvalidPath)?;
            // SAFETY: `parent` is an open directory descriptor and `name` is NUL-terminated.
            let fd = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o600 as libc::c_uint,
                )
            };
            if fd >= 0 {
                // SAFETY: `openat` returned a new owned descriptor.
                return Ok((name, unsafe { File::from_raw_fd(fd) }));
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(file_error(error));
            }
        }
        Err(backend_error(
            ErrorKind::ResourceExhausted,
            Some(libc::EEXIST),
        ))
    }
}

impl FileBackend for IosFiles {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn read(&mut self, path: AppPath<'_>) -> Result<Vec<u8>, FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is an open directory descriptor and `leaf` is one validated component.
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
        // SAFETY: `openat` returned a new owned descriptor.
        let mut file = unsafe { File::from_raw_fd(fd) };
        if !file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(file_error)?;
        Ok(bytes)
    }

    fn write(
        &mut self,
        path: AppPath<'_>,
        bytes: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mode = options.mode();
        let atomic_supported = match mode {
            FileWriteMode::CreateOrReplace => true,
            FileWriteMode::CreateNew => self.volume_supports_exclusive_rename(path.directory()),
            FileWriteMode::ReplaceExisting => self.volume_supports_swap_rename(path.directory()),
            _ => false,
        };
        if !atomic_supported
            && options.atomicity() == framework_files::AtomicityRequirement::RequireAtomic
        {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if !atomic_supported && mode == FileWriteMode::CreateNew {
            return write_create_new(&parent, &leaf, bytes);
        }
        if !atomic_supported && mode == FileWriteMode::ReplaceExisting {
            return write_replace_existing(&parent, &leaf, bytes);
        }
        let (temporary_name, mut temporary_file) = self.temporary_file(&parent)?;
        if let Err(error) = temporary_file.write_all(bytes) {
            drop(temporary_file);
            unlink_if_present(&parent, &temporary_name);
            return Err(file_error(error));
        }
        drop(temporary_file);

        let rename_result = match mode {
            FileWriteMode::CreateOrReplace => rename_at(&parent, &temporary_name, &leaf),
            FileWriteMode::CreateNew => {
                rename_atx(&parent, &temporary_name, &leaf, libc::RENAME_EXCL)
            }
            FileWriteMode::ReplaceExisting => {
                rename_atx(&parent, &temporary_name, &leaf, libc::RENAME_SWAP)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported file write mode",
            )),
        };
        if let Err(error) = rename_result {
            unlink_if_present(&parent, &temporary_name);
            return Err(file_error(error));
        }
        if mode != FileWriteMode::CreateOrReplace {
            // A successful swap/exclusive rename leaves no owned staging object or an old file at
            // the temporary name. Cleanup is best-effort after the visible atomic commit.
            unlink_if_present(&parent, &temporary_name);
        }
        Ok(WriteOutcome::new(WriteAtomicity::Atomic))
    }

    fn create_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is an open directory descriptor and `leaf` is one validated component.
        let result = unsafe { libc::mkdirat(parent.as_raw_fd(), leaf.as_ptr(), 0o700) };
        if result < 0 {
            Err(file_error(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    fn read_directory(&mut self, path: AppPath<'_>) -> Result<Vec<DirectoryEntry>, FileError> {
        let parts = path_parts(path)?;
        let directory = open_directory(self.root(path.directory())?, &parts).map_err(file_error)?;
        // `fdopendir` takes ownership of its descriptor, so duplicate the borrowed directory fd.
        // SAFETY: `directory` is a valid open descriptor.
        let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
        if duplicate < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `duplicate` is a newly owned descriptor that `fdopendir` may consume.
        let stream = unsafe { libc::fdopendir(duplicate) };
        if stream.is_null() {
            let error = io::Error::last_os_error();
            // SAFETY: `fdopendir` failed and did not take ownership of `duplicate`.
            unsafe { libc::close(duplicate) };
            return Err(file_error(error));
        }
        let mut entries = Vec::new();
        let read_result = loop {
            // SAFETY: `stream` remains live until `closedir` below.
            unsafe { *libc::__error() = 0 };
            // SAFETY: `stream` is a valid directory stream.
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                // SAFETY: `__error` returns this thread's errno slot.
                let code = unsafe { *libc::__error() };
                break if code == 0 {
                    Ok(())
                } else {
                    Err(file_error(io::Error::from_raw_os_error(code)))
                };
            }
            // SAFETY: `readdir` returns a live dirent whose name is NUL-terminated.
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            if name.to_bytes() == b"." || name.to_bytes() == b".." {
                continue;
            }
            let name_text = match name.to_str() {
                Ok(name) => name,
                Err(_) => break Err(backend_error(ErrorKind::InvalidInput, None)),
            };
            let kind = match entry_kind(directory.as_raw_fd(), name) {
                Ok(kind) => kind,
                Err(error) => break Err(file_error(error)),
            };
            match DirectoryEntry::new(name_text.to_owned(), kind) {
                Ok(entry) => entries.push(entry),
                Err(error) => break Err(error),
            }
        };
        // SAFETY: `stream` was returned by `fdopendir` and is closed exactly once.
        let close_result = unsafe { libc::closedir(stream) };
        read_result?;
        if close_result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        Ok(entries)
    }

    fn remove_file(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is a single validated component. `unlinkat` does not
        // follow a final symlink.
        let result = unsafe { libc::unlinkat(parent.as_raw_fd(), leaf.as_ptr(), 0) };
        if result < 0 {
            Err(file_error(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    fn remove_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is a single validated component. `AT_REMOVEDIR`
        // removes one empty directory and does not follow a final symlink.
        let result =
            unsafe { libc::unlinkat(parent.as_raw_fd(), leaf.as_ptr(), libc::AT_REMOVEDIR) };
        if result < 0 {
            Err(file_error(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    fn exists(&mut self, path: AppPath<'_>) -> Result<bool, FileError> {
        let parts = path_parts(path)?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            Ok(true)
        } else {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                Ok(false)
            } else {
                Err(file_error(error))
            }
        }
    }
}

fn directory_url(directory: NSSearchPathDirectory) -> Result<Retained<NSURL>, FileError> {
    autoreleasepool(|_| {
        let manager = NSFileManager::defaultManager();
        manager
            .URLForDirectory_inDomain_appropriateForURL_create_error(
                directory,
                NSSearchPathDomainMask::UserDomainMask,
                None,
                true,
            )
            .map_err(|error| foundation_error(&error))
    })
}

fn temporary_url() -> Result<Retained<NSURL>, FileError> {
    autoreleasepool(|_| {
        let manager = NSFileManager::defaultManager();
        Ok(manager.temporaryDirectory())
    })
}

fn volume_supports(url: &NSURL, key: &NSURLResourceKey) -> bool {
    autoreleasepool(|_| {
        let mut value = None;
        // SAFETY: Apple's NSURL resource keys used here return NSNumber boolean values.
        unsafe { url.getResourceValue_forKey_error(&mut value, key) }.ok()?;
        value?
            .downcast::<NSNumber>()
            .ok()
            .map(|number| number.boolValue())
    })
    .unwrap_or(false)
}

fn path_from_url(url: &NSURL) -> Result<PathBuf, FileError> {
    let path = url
        .path()
        .ok_or_else(|| backend_error(ErrorKind::Unavailable, None))?;
    Ok(PathBuf::from(path.to_string()))
}

fn open_root(url: &NSURL) -> Result<File, FileError> {
    let path = path_from_url(url)?;
    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| FileError::InvalidPath)?;
    // SAFETY: `path` is NUL-terminated and Foundation supplied this app-sandbox URL. The flags
    // reject a final symlink and require the resolved root to be a directory.
    let fd = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    // SAFETY: `open` returned a newly owned descriptor.
    let file = unsafe { File::from_raw_fd(fd) };
    if file.metadata().map_err(file_error)?.is_dir() {
        Ok(file)
    } else {
        Err(backend_error(ErrorKind::Unavailable, None))
    }
}

fn path_parts(path: AppPath<'_>) -> Result<Vec<CString>, FileError> {
    let text = path.relative();
    if text.is_empty()
        || text.starts_with('/')
        || text.ends_with('/')
        || text.bytes().any(|byte| byte == 0 || byte == b'\\')
    {
        return Err(FileError::InvalidPath);
    }
    text.split('/')
        .map(|part| {
            if part.is_empty() || part == "." || part == ".." {
                return Err(FileError::InvalidPath);
            }
            CString::new(part).map_err(|_| FileError::InvalidPath)
        })
        .collect()
}

fn open_parent(root: &File, parts: &[CString]) -> Result<(File, CString), FileError> {
    let (leaf, parents) = parts.split_last().ok_or(FileError::InvalidPath)?;
    Ok((
        open_directory(root, parents).map_err(file_error)?,
        leaf.clone(),
    ))
}

fn open_directory(root: &File, parts: &[CString]) -> io::Result<File> {
    let mut directory = root.try_clone()?;
    for part in parts {
        // SAFETY: the current fd is a directory and `part` is one validated component. The flags
        // reject both final symlinks and non-directory entries.
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                part.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `openat` returned a newly owned descriptor.
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Ok(directory)
}

fn entry_kind(parent_fd: libc::c_int, name: &CStr) -> io::Result<FileKind> {
    let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: the parent descriptor is open, name is NUL-terminated, and metadata is writable.
    let result = unsafe {
        libc::fstatat(
            parent_fd,
            name.as_ptr(),
            metadata.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful `fstatat` initialized the structure.
    let mode = unsafe { metadata.assume_init() }.st_mode & libc::S_IFMT;
    Ok(match mode {
        libc::S_IFREG => FileKind::File,
        libc::S_IFDIR => FileKind::Directory,
        _ => FileKind::Other,
    })
}

fn write_create_new(
    parent: &File,
    leaf: &CString,
    bytes: &[u8],
) -> Result<WriteOutcome, FileError> {
    // SAFETY: `parent` is open and `leaf` is one validated component. O_EXCL prevents replacement
    // and O_NOFOLLOW rejects a final symlink.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_WRONLY
                | libc::O_CREAT
                | libc::O_EXCL
                | libc::O_NOFOLLOW
                | libc::O_CLOEXEC
                | libc::O_NONBLOCK,
            0o600 as libc::c_uint,
        )
    };
    if fd < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    // SAFETY: `openat` returned a new owned descriptor.
    let mut file = unsafe { File::from_raw_fd(fd) };
    file.write_all(bytes).map_err(file_error)?;
    Ok(WriteOutcome::new(WriteAtomicity::NotGuaranteed))
}

fn write_replace_existing(
    parent: &File,
    leaf: &CString,
    bytes: &[u8],
) -> Result<WriteOutcome, FileError> {
    // Open without truncation first so non-regular entries are rejected before mutation.
    // SAFETY: `parent` is open and `leaf` is one validated component. O_NOFOLLOW rejects a final
    // symlink and O_NONBLOCK avoids waiting on a FIFO before its type is checked.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_WRONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    // SAFETY: `openat` returned a new owned descriptor.
    let mut file = unsafe { File::from_raw_fd(fd) };
    if !file.metadata().map_err(file_error)?.is_file() {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    // SAFETY: the descriptor is open for writing and refers to the checked regular file.
    if unsafe { libc::ftruncate(file.as_raw_fd(), 0) } < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    file.write_all(bytes).map_err(file_error)?;
    Ok(WriteOutcome::new(WriteAtomicity::NotGuaranteed))
}

fn rename_at(parent: &File, source: &CString, target: &CString) -> io::Result<()> {
    // SAFETY: both names are NUL-terminated single components within the same open directory.
    let result = unsafe {
        libc::renameat(
            parent.as_raw_fd(),
            source.as_ptr(),
            parent.as_raw_fd(),
            target.as_ptr(),
        )
    };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn rename_atx(
    parent: &File,
    source: &CString,
    target: &CString,
    flags: libc::c_uint,
) -> io::Result<()> {
    // SAFETY: both names are NUL-terminated single components within the same open directory.
    let result = unsafe {
        libc::renameatx_np(
            parent.as_raw_fd(),
            source.as_ptr(),
            parent.as_raw_fd(),
            target.as_ptr(),
            flags,
        )
    };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn unlink_if_present(parent: &File, name: &CString) {
    // SAFETY: the parent descriptor is open and name is one temporary component.
    unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) };
}

fn foundation_error(error: &NSError) -> FileError {
    let code = i32::try_from(error.code()).ok();
    backend_error(ErrorKind::Platform, code)
}

fn file_error(error: io::Error) -> FileError {
    let code = error.raw_os_error();
    let kind = match code {
        Some(value) if value == libc::ENOENT => ErrorKind::NotFound,
        Some(value) if value == libc::EEXIST => ErrorKind::AlreadyExists,
        Some(value) if value == libc::EACCES || value == libc::EPERM => ErrorKind::PermissionDenied,
        Some(value) if value == libc::ENOTSUP || value == libc::EOPNOTSUPP => {
            ErrorKind::Unsupported
        }
        Some(value)
            if value == libc::ENOMEM
                || value == libc::ENOSPC
                || value == libc::EMFILE
                || value == libc::ENFILE =>
        {
            ErrorKind::ResourceExhausted
        }
        Some(value)
            if value == libc::EINVAL
                || value == libc::ENOTDIR
                || value == libc::ELOOP
                || value == libc::ENAMETOOLONG =>
        {
            ErrorKind::InvalidInput
        }
        _ => match error.kind() {
            io::ErrorKind::NotFound => ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            io::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
            io::ErrorKind::Unsupported => ErrorKind::Unsupported,
            io::ErrorKind::OutOfMemory => ErrorKind::ResourceExhausted,
            _ => ErrorKind::Platform,
        },
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
