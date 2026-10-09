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
        fd::{AsRawFd, FromRawFd, IntoRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::PathBuf,
};

mod bookmark;
mod coordination;
mod path_validation;
mod security_scope;

pub use bookmark::{IosPlainBookmarkData, IosResolvedBookmark};
pub use coordination::IosFileCoordinator;
use path_validation::path_parts;
pub use security_scope::{IosSecurityScopedAccess, SecurityScopeStartError};

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

/// A point-in-time POSIX modification timestamp reported for one iOS filesystem entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileModificationTime {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileModificationTime {
    /// Returns whole POSIX seconds relative to the Unix epoch.
    pub const fn seconds_since_unix_epoch(self) -> i64 {
        self.seconds_since_unix_epoch
    }

    /// Returns the subsecond nanosecond field reported by `stat`, in `0..1_000_000_000`.
    ///
    /// The filesystem may store or report a coarser precision than one nanosecond.
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

/// A point-in-time POSIX status-change timestamp reported for one iOS filesystem entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileStatusChangeTime {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileStatusChangeTime {
    /// Returns whole POSIX seconds relative to the Unix epoch.
    pub const fn seconds_since_unix_epoch(self) -> i64 {
        self.seconds_since_unix_epoch
    }

    /// Returns the subsecond nanosecond field reported by `stat`, in `0..1_000_000_000`.
    ///
    /// The filesystem may store or report a coarser precision than one nanosecond.
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

/// A point-in-time `(st_dev, st_ino)` pair for one iOS filesystem entry.
///
/// This value is not a persistent identifier, open handle, or guarantee against inode reuse.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileIdentitySnapshot {
    device_id: u64,
    inode_number: u64,
}

impl IosFileIdentitySnapshot {
    /// Returns the nonnegative device identifier reported by `stat`.
    pub const fn device_id(self) -> u64 {
        self.device_id
    }

    /// Returns the inode number reported by `stat`.
    pub const fn inode_number(self) -> u64 {
        self.inode_number
    }
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

    /// Copies a URLSession temporary download file to an app-sandbox path.
    ///
    /// Call this synchronous operation before the URLSession download delegate callback returns.
    /// It validates the destination with the same descriptor-relative, no-follow rules as other
    /// file operations, copies without a payload-sized `Vec` into a private same-directory
    /// staging file, then commits with one atomic `renameat` after a checked close. It creates or
    /// replaces the final entry; a final symlink is replaced rather than followed. An existing
    /// destination that names the same file as the source is rejected. The operation does not
    /// retain the URL or remove the URLSession-owned source file. Only pass the URLSession callback
    /// URL; this method does not establish containment for arbitrary source file URLs. It reads
    /// from one opened source descriptor without coordinating or snapshotting the source, so
    /// concurrent source mutation through another handle can affect the copied bytes. Atomic
    /// visibility does not imply crash durability. The alias check is repeated before commit, but
    /// concurrent destination mutation through another handle is not serialized.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid destination, unsafe/missing parent, source/destination
    /// alias, invalid source URL, non-regular source, copy or close failure, or failed atomic
    /// rename. POSIX errors preserve their native code. Staging-file cleanup after a copy, close,
    /// alias-check, or rename failure is best-effort.
    pub fn adopt_url_session_download(
        &mut self,
        temporary_file_url: &NSURL,
        destination: AppPath<'_>,
    ) -> Result<WriteOutcome, FileError> {
        let parts = path_parts(destination.relative())?;
        let root = self.root(destination.directory())?;
        let (parent, leaf) = open_parent(root, &parts)?;
        let mut source = open_url_session_temporary_file(temporary_file_url)?;
        reject_source_alias(&source, &parent, &leaf)?;
        let (staging_name, mut staging_file) = self.temporary_file(&parent)?;
        let copy_result = io::copy(&mut source, &mut staging_file);
        let close_result = close_file(staging_file);
        if let Err(error) = copy_result {
            unlink_if_present(&parent, &staging_name);
            return Err(file_error(error));
        }
        if let Err(error) = close_result {
            unlink_if_present(&parent, &staging_name);
            return Err(file_error(error));
        }
        if let Err(error) = reject_source_alias(&source, &parent, &leaf) {
            unlink_if_present(&parent, &staging_name);
            return Err(error);
        }
        if let Err(error) = rename_at(&parent, &staging_name, &leaf) {
            unlink_if_present(&parent, &staging_name);
            return Err(file_error(error));
        }
        Ok(WriteOutcome::new(WriteAtomicity::Atomic))
    }

    /// Returns one regular file's current byte length without reading its contents.
    ///
    /// The path uses the same app-sandbox root and descriptor-relative, no-follow traversal as
    /// the `FileBackend` methods. The final entry is inspected with one
    /// `fstatat(..., AT_SYMLINK_NOFOLLOW)` call; a final symlink, directory, or special entry is
    /// rejected. The returned size is a point-in-time metadata snapshot and may differ from a
    /// later read if another handle changes or replaces the file. This method does not follow
    /// arbitrary URLs, start a security scope, or establish stronger containment than the
    /// documented concurrent directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a non-regular entry or
    /// invalid size, or the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn regular_file_size(&self, path: AppPath<'_>) -> Result<u64, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` inspects rather than follows a final symbolic link.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        u64::try_from(metadata.st_size).map_err(|_| backend_error(ErrorKind::InvalidInput, None))
    }

    /// Returns one regular file's filesystem-reported allocated block count.
    ///
    /// The `u64` is the `st_blocks` value in 512-byte units from one
    /// `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup. It is distinct from logical byte length returned
    /// by [`Self::regular_file_size`]; sparse files may report fewer allocated blocks than their
    /// logical size implies. This is the filesystem's metadata value, not a promise of exact
    /// physical-device usage, exclusive allocation, or a stable measure across file systems. The
    /// final entry must be a regular file; a symbolic link, directory, or special entry is
    /// rejected. Another handle may modify or replace the entry before a later operation. This
    /// method reads no file contents, accepts no arbitrary URL, and does not start a security
    /// scope or exceed the documented concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a non-regular entry or
    /// negative block count, or the mapped POSIX error for path traversal and metadata lookup
    /// failures.
    pub fn regular_file_allocated_blocks_512(&self, path: AppPath<'_>) -> Result<u64, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` inspects rather than follows a final symbolic link.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        u64::try_from(metadata.st_blocks).map_err(|_| backend_error(ErrorKind::InvalidInput, None))
    }

    /// Returns the no-follow kind of one app-sandbox directory entry.
    ///
    /// Classification matches `read_directory`: regular files return `File`, directories return
    /// `Directory`, and symbolic links or other entry types return `Other`. The path uses the same
    /// validated `AppPath` and no-follow parent traversal as other sandbox methods. This reads no
    /// file contents and does not follow the final symbolic link. The result is a point-in-time
    /// observation; another handle may replace the entry before a later operation. It does not
    /// accept arbitrary URLs, start a security scope, or establish stronger containment than the
    /// documented concurrent directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, or the mapped POSIX error for path traversal
    /// and metadata lookup failures.
    pub fn entry_kind(&self, path: AppPath<'_>) -> Result<FileKind, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        self::entry_kind(parent.as_raw_fd(), leaf.as_c_str()).map_err(file_error)
    }

    /// Returns one entry's no-follow data-modification timestamp.
    ///
    /// The timestamp is the POSIX seconds/nanoseconds pair from `fstatat(...,
    /// AT_SYMLINK_NOFOLLOW)`. It describes the final entry itself, including a symbolic link rather
    /// than its target. The query uses the same validated `AppPath` and no-follow parent traversal
    /// as other sandbox methods, reads no file contents, and accepts files, directories, symlinks,
    /// and special entries. Filesystems may store coarser precision; callers may also set file
    /// timestamps, so this value is not a content version, reliable change token, or durability
    /// proof. It is a point-in-time result and another handle may replace the entry before a later
    /// operation. This method does not accept arbitrary URLs, start a security scope, or establish
    /// stronger containment than the documented concurrent directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a malformed timestamp
    /// field, or the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn entry_modification_time(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileModificationTime, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` reads the final link's own metadata instead of following it.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        let seconds_since_unix_epoch = metadata.st_mtime;
        let nanoseconds = u32::try_from(metadata.st_mtime_nsec)
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
        if nanoseconds >= 1_000_000_000 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileModificationTime {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns one non-symlink entry's no-follow POSIX status-change timestamp.
    ///
    /// The timestamp is the seconds/nanoseconds pair from `st_ctime` and `st_ctime_nsec` in one
    /// `fstatat(..., AT_SYMLINK_NOFOLLOW)` result. Apple documents this field as the time of the
    /// last file-status change, including changes from operations such as `chmod`, `chown`,
    /// `link`, `rename`, `unlink`, `utimes`, and `write`. A final symbolic link is rejected
    /// because Apple's `lstat` contract does not define its own timestamp fields. The query uses
    /// the same validated `AppPath` and no-follow parent traversal as other sandbox methods,
    /// reads no file contents, and accepts regular files, directories, and special entries.
    /// Filesystems may report coarser precision; this is a point-in-time metadata observation, not
    /// a content version, reliable change token, or durability proof. Another handle may change or
    /// replace the entry before a later operation. This method does not accept arbitrary URLs,
    /// start a security scope, or establish stronger containment than the documented concurrent
    /// directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a final symbolic link or
    /// malformed timestamp field, or the mapped POSIX error for path traversal and metadata lookup
    /// failures.
    pub fn entry_status_change_time(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileStatusChangeTime, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` prevents following a final symbolic link.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT == libc::S_IFLNK {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let seconds_since_unix_epoch = metadata.st_ctime;
        let nanoseconds = u32::try_from(metadata.st_ctime_nsec)
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
        if nanoseconds >= 1_000_000_000 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileStatusChangeTime {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns one entry's no-follow device and inode numbers.
    ///
    /// The pair is read from one `fstatat(..., AT_SYMLINK_NOFOLLOW)` result for the final entry
    /// itself, including a symbolic link rather than its target. The query uses the same
    /// validated `AppPath` and no-follow parent traversal as other sandbox methods. It accepts
    /// files, directories, symlinks, and special entries, reads no file contents, and does not
    /// open or retain the entry. Equal pairs can identify the same live inode at observation time,
    /// such as two hard-link names; they are not globally unique or persistent, and a filesystem
    /// may reuse an inode number after removal. Separate path queries are not atomic with later
    /// operations and may race another handle's changes. This method does not accept arbitrary
    /// URLs, start a security scope, or establish stronger containment than the documented
    /// concurrent directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a negative device ID, or
    /// the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn entry_identity_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileIdentitySnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` reads the final link's own metadata instead of following it.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        let device_id = u64::try_from(metadata.st_dev)
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
        Ok(IosFileIdentitySnapshot {
            device_id,
            inode_number: metadata.st_ino,
        })
    }

    /// Returns one entry's raw POSIX permission and special-mode bits.
    ///
    /// The result is `st_mode & 0o7777` from one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup of the
    /// final entry itself. It includes owner/group/other read, write, and execute bits plus the
    /// set-user-ID, set-group-ID, and sticky bits. For a symlink, the value comes from the link
    /// itself. These stored bits do not determine whether a later operation will succeed; this
    /// method does not check effective access, App Sandbox policy, file-protection state, or
    /// concurrent path changes. The query uses the same validated `AppPath` and no-follow parent
    /// traversal as other sandbox methods, reads no file contents, and retains no descriptor.
    /// It does not accept arbitrary URLs, start a security scope, or establish stronger
    /// containment than the documented concurrent directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, or the mapped POSIX error for path traversal
    /// and metadata lookup failures.
    pub fn entry_posix_permission_bits(&self, path: AppPath<'_>) -> Result<u16, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` reads the final link's own metadata instead of following it.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        Ok(metadata.st_mode & 0o7777)
    }

    /// Returns the point-in-time hard-link count for one regular file.
    ///
    /// The count comes from one `fstatat(..., AT_SYMLINK_NOFOLLOW)` result. The final entry must
    /// be a regular file; a symbolic link, directory, or special entry returns `InvalidInput`
    /// rather than following or reinterpreting it. A count greater than one reports multiple
    /// hard links to the same inode at observation time, but does not list their paths or prove
    /// exclusive ownership. Another handle may add or remove links, or replace the path, before a
    /// later operation. This query reads no file contents, retains no descriptor, does not accept
    /// arbitrary URLs, and does not start a security scope. It has the documented concurrent
    /// directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a non-regular entry, or
    /// the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn regular_file_hard_link_count(&self, path: AppPath<'_>) -> Result<u64, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `parent` is open, `leaf` is one validated component, and `metadata` is writable.
        // `AT_SYMLINK_NOFOLLOW` inspects rather than follows a final symbolic link.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatat` initialized the structure.
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(u64::from(metadata.st_nlink))
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
        let parts = path_parts(path.relative())?;
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
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mode = options.mode();
        if mode == FileWriteMode::ReplaceExisting {
            require_regular_file(&parent, &leaf)?;
        }
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

        if mode == FileWriteMode::ReplaceExisting {
            if let Err(error) = require_regular_file(&parent, &leaf) {
                unlink_if_present(&parent, &temporary_name);
                return Err(error);
            }
        }

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
        let parts = path_parts(path.relative())?;
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
        let parts = path_parts(path.relative())?;
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
        let parts = path_parts(path.relative())?;
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
        let parts = path_parts(path.relative())?;
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
        let parts = path_parts(path.relative())?;
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

fn open_url_session_temporary_file(url: &NSURL) -> Result<File, FileError> {
    if !url.isFileURL() {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    autoreleasepool(|_| {
        let representation = url.fileSystemRepresentation();
        // SAFETY: Foundation returns a NUL-terminated inner pointer valid for this autorelease
        // pool. `path` is read and opened before the pool drains.
        let path = unsafe { CStr::from_ptr(representation.as_ptr()) };
        if !path.to_bytes().starts_with(b"/") {
            return Err(FileError::InvalidPath);
        }
        // SAFETY: `path` is a live NUL-terminated absolute path. These flags reject a final
        // symlink, keep the descriptor local to this process, and avoid blocking on a special file.
        let fd = unsafe {
            libc::open(
                path.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `open` returned a new owned descriptor.
        let file = unsafe { File::from_raw_fd(fd) };
        if !file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(file)
    })
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

fn require_regular_file(parent: &File, name: &CStr) -> Result<(), FileError> {
    match entry_kind(parent.as_raw_fd(), name).map_err(file_error)? {
        FileKind::File => Ok(()),
        _ => Err(backend_error(ErrorKind::InvalidInput, None)),
    }
}

fn reject_source_alias(source: &File, parent: &File, target: &CStr) -> Result<(), FileError> {
    let source_metadata = source.metadata().map_err(file_error)?;
    let mut target_metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: `parent` is open, `target` is one validated component, and the output is writable.
    let result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            target.as_ptr(),
            target_metadata.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(());
        }
        return Err(file_error(error));
    }
    // SAFETY: successful `fstatat` initialized the structure.
    let target_metadata = unsafe { target_metadata.assume_init() };
    if u64::try_from(target_metadata.st_dev).ok() == Some(source_metadata.dev())
        && source_metadata.ino() == target_metadata.st_ino
    {
        Err(backend_error(ErrorKind::InvalidInput, None))
    } else {
        Ok(())
    }
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

fn close_file(file: File) -> io::Result<()> {
    let fd = file.into_raw_fd();
    // SAFETY: ownership of this descriptor was transferred from `File` and it is closed once.
    if unsafe { libc::close(fd) } < 0 {
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
                || value == libc::EISDIR
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
