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
    NSURLResourceKey, NSURLVolumeSupportsCasePreservedNamesKey,
    NSURLVolumeSupportsCaseSensitiveNamesKey, NSURLVolumeSupportsExclusiveRenamingKey,
    NSURLVolumeSupportsFileCloningKey, NSURLVolumeSupportsSwapRenamingKey,
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

// These public `sys/clonefile.h` macros lack Rust constants in the locked `libc` binding.
const CLONE_NOFOLLOW_ANY: u32 = 0x0008;
const CLONE_RESOLVE_BENEATH: u32 = 0x0010;
// These public `sys/stat.h` EF macros are not bound as Rust constants by locked `libc`.
const EF_MAY_SHARE_BLOCKS: u64 = 0x0000_0001;
const EF_NO_XATTRS: u64 = 0x0000_0002;
const EF_IS_PURGEABLE: u64 = 0x0000_0008;
const EF_IS_SPARSE: u64 = 0x0000_0010;
const EF_SHARES_ALL_BLOCKS: u64 = 0x0000_0040;
// This public `sys/attr.h` macro is not bound as a Rust constant by locked `libc`.
const ATTR_CMNEXT_CLONE_REFCNT: u32 = 0x0000_1000;

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
    rename_exclusive: [Option<bool>; 4],
    rename_swap: [Option<bool>; 4],
    case_sensitive_names: [Option<bool>; 4],
    case_preserved_names: [Option<bool>; 4],
    volume_file_cloning: [Option<bool>; 4],
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

/// A point-in-time POSIX access timestamp reported for one iOS filesystem entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileAccessTime {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileAccessTime {
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

/// A point-in-time filesystem-reported added-to-directory timestamp for one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileAddedTime {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileAddedTime {
    /// Returns whole seconds relative to the Unix epoch.
    pub const fn seconds_since_unix_epoch(self) -> i64 {
        self.seconds_since_unix_epoch
    }

    /// Returns the subsecond nanosecond field reported by the filesystem.
    ///
    /// The filesystem may store or report a coarser precision than one nanosecond.
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

/// A point-in-time filesystem-stored backup-time marker for one iOS sandbox file or directory.
///
/// This value is only the timestamp stored by the filesystem. It does not prove that an OS or
/// iCloud backup completed or includes the object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileBackupTimeMarker {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileBackupTimeMarker {
    /// Returns the stored marker's whole seconds relative to the Unix epoch.
    pub const fn seconds_since_unix_epoch(self) -> i64 {
        self.seconds_since_unix_epoch
    }

    /// Returns the stored marker's subsecond nanosecond field.
    ///
    /// The filesystem may store or report a coarser precision than one nanosecond.
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

/// A point-in-time filesystem-reported creation timestamp for one iOS sandbox regular file.
///
/// The value is read/write metadata. It is not immutable proof of the real-world creation event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileCreationTime {
    seconds_since_unix_epoch: i64,
    nanoseconds: u32,
}

impl IosFileCreationTime {
    /// Returns the filesystem-reported whole seconds relative to the Unix epoch.
    pub const fn seconds_since_unix_epoch(self) -> i64 {
        self.seconds_since_unix_epoch
    }

    /// Returns the filesystem-reported subsecond nanosecond field.
    ///
    /// The filesystem may store or report a coarser precision than one nanosecond.
    pub const fn nanoseconds(self) -> u32 {
        self.nanoseconds
    }
}

/// A point-in-time set of Darwin BSD file flags from one iOS filesystem entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosBsdFileFlags(u32);

impl IosBsdFileFlags {
    /// The `UF_NODUMP` flag.
    pub const UF_NODUMP: Self = Self(libc::UF_NODUMP);
    /// The `UF_IMMUTABLE` flag.
    pub const UF_IMMUTABLE: Self = Self(libc::UF_IMMUTABLE);
    /// The `UF_APPEND` flag.
    pub const UF_APPEND: Self = Self(libc::UF_APPEND);
    /// The `UF_OPAQUE` flag.
    pub const UF_OPAQUE: Self = Self(libc::UF_OPAQUE);
    /// The `UF_COMPRESSED` flag.
    pub const UF_COMPRESSED: Self = Self(libc::UF_COMPRESSED);
    /// The `UF_HIDDEN` flag.
    pub const UF_HIDDEN: Self = Self(libc::UF_HIDDEN);
    /// The `SF_ARCHIVED` flag.
    pub const SF_ARCHIVED: Self = Self(libc::SF_ARCHIVED);
    /// The `SF_IMMUTABLE` flag.
    pub const SF_IMMUTABLE: Self = Self(libc::SF_IMMUTABLE);
    /// The `SF_APPEND` flag.
    pub const SF_APPEND: Self = Self(libc::SF_APPEND);

    /// Returns the raw `st_flags` bits, including bits this API does not name.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether all bits in `flag` are present.
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// Raw public Darwin extended flags for one regular iOS filesystem file.
///
/// The snapshot preserves bits not named by this binding. Named flags are point-in-time
/// filesystem reports, not stable allocation or future-operation guarantees.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileExtendedFlags(u64);

impl IosFileExtendedFlags {
    /// `EF_MAY_SHARE_BLOCKS`: the file may share blocks with another file.
    pub const EF_MAY_SHARE_BLOCKS: Self = Self(EF_MAY_SHARE_BLOCKS);
    /// `EF_NO_XATTRS`: the file has no extended attributes.
    pub const EF_NO_XATTRS: Self = Self(EF_NO_XATTRS);
    /// `EF_IS_PURGEABLE`: the filesystem may delete the file when asked to free space.
    pub const EF_IS_PURGEABLE: Self = Self(EF_IS_PURGEABLE);
    /// `EF_IS_SPARSE`: the file has at least one sparse region.
    pub const EF_IS_SPARSE: Self = Self(EF_IS_SPARSE);
    /// `EF_SHARES_ALL_BLOCKS`: the file shares all of its blocks with another file.
    pub const EF_SHARES_ALL_BLOCKS: Self = Self(EF_SHARES_ALL_BLOCKS);

    /// Returns the raw `ATTR_CMNEXT_EXT_FLAGS` bits, including unnamed bits.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Returns whether all bits in `flag` are present.
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// An opaque point-in-time `ATTR_CMNEXT_CLONEID` value for one regular iOS file.
///
/// Compare snapshots only as a current XNU clone-ID report. This value has no documented
/// persistence or change-token guarantee and is not a content hash.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileCloneIdSnapshot(u64);

impl IosFileCloneIdSnapshot {
    /// Returns the opaque 64-bit clone ID reported by the filesystem.
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// An opaque point-in-time `ATTR_CMNEXT_LINKID` value for one regular iOS file entry.
///
/// XNU scopes link IDs to a mounted volume. Do not treat this value as a content identity or a
/// cross-mount persistent identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileLinkIdSnapshot(u64);

impl IosFileLinkIdSnapshot {
    /// Returns the opaque 64-bit link ID reported by the filesystem.
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A point-in-time count of full clones reported for one regular iOS file.
///
/// The count does not identify clone paths or include partial block-sharing peers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileFullCloneCountSnapshot(u32);

impl IosFileFullCloneCountSnapshot {
    /// Returns the reported number of full clones.
    pub const fn count(self) -> u32 {
        self.0
    }
}

/// A point-in-time byte count reported as private to one regular iOS file.
///
/// XNU defines this count as bytes not trapped in a clone or snapshot that would be freed
/// immediately if the file were deleted. It does not reserve capacity for a later operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFilePrivateSizeSnapshot(u64);

impl IosFilePrivateSizeSnapshot {
    /// Returns the reported private size in bytes.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A point-in-time logical byte count across all forks of one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileTotalForkSizeSnapshot(u64);

impl IosFileTotalForkSizeSnapshot {
    /// Returns the reported total logical size in bytes across all file forks.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A point-in-time allocated byte count for the data fork of one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileDataForkAllocatedSizeSnapshot(u64);

impl IosFileDataForkAllocatedSizeSnapshot {
    /// Returns the reported data-fork allocation in bytes.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A point-in-time allocated byte count for the resource fork of one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileResourceForkAllocatedSizeSnapshot(u64);

impl IosFileResourceForkAllocatedSizeSnapshot {
    /// Returns the reported resource-fork allocation in bytes.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A point-in-time logical byte count for the resource fork of one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileResourceForkSizeSnapshot(u64);

impl IosFileResourceForkSizeSnapshot {
    /// Returns the reported logical resource-fork size in bytes.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A point-in-time document ID for one regular iOS file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileDocumentIdSnapshot(u32);

impl IosFileDocumentIdSnapshot {
    /// Returns the nonzero document ID, or `None` when XNU reports its invalid zero value.
    pub const fn document_id(self) -> Option<u32> {
        if self.0 == 0 { None } else { Some(self.0) }
    }
}

/// An opaque numeric code returned for a file or directory's data-protection class.
///
/// Apple documents the field as a `u32` class value but does not publish a numeric mapping to
/// named protection levels. Preserve or display this raw value only; it does not identify a
/// protection level or establish whether data can be accessed.
#[derive(Clone, Copy, Debug)]
pub struct IosFileDataProtectionClassCode(u32);

impl IosFileDataProtectionClassCode {
    /// Returns the opaque `u32` reported by `ATTR_CMN_DATA_PROTECT_FLAGS`.
    pub const fn raw_value(self) -> u32 {
        self.0
    }
}

/// A point-in-time POSIX numeric owner and group pair for one iOS filesystem entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileOwnerIds {
    user_id: u32,
    group_id: u32,
}

impl IosFileOwnerIds {
    /// Returns the numeric POSIX owner ID.
    pub const fn user_id(self) -> u32 {
        self.user_id
    }

    /// Returns the numeric POSIX group ID.
    pub const fn group_id(self) -> u32 {
        self.group_id
    }
}

/// Point-in-time effective read, write, and execute/search permissions for one iOS entry.
///
/// The native mask describes the calling process's effective UID. It is not a guarantee that a
/// later operation will succeed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosEntryEffectiveAccess(u32);

impl IosEntryEffectiveAccess {
    /// Returns the raw `ATTR_CMN_USERACCESS` permission mask, including unnamed bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether the effective permission mask includes `R_OK`.
    pub const fn allows_read(self) -> bool {
        self.0 & libc::R_OK as u32 != 0
    }

    /// Returns whether the effective permission mask includes `W_OK`.
    ///
    /// For a directory, this is the reported permission to add a child entry.
    pub const fn allows_write(self) -> bool {
        self.0 & libc::W_OK as u32 != 0
    }

    /// Returns whether the effective permission mask includes `X_OK`.
    ///
    /// For a directory, this is the reported search permission.
    pub const fn allows_execute_or_search(self) -> bool {
        self.0 & libc::X_OK as u32 != 0
    }
}

/// Cached Foundation volume support for exclusive and swap rename options.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosVolumeRenameSupportSnapshot {
    exclusive: Option<bool>,
    swap: Option<bool>,
}

impl IosVolumeRenameSupportSnapshot {
    /// Returns whether the volume reports support for `RENAME_EXCL`, or `None` if unreported.
    pub const fn exclusive_rename_supported(self) -> Option<bool> {
        self.exclusive
    }

    /// Returns whether the volume reports support for `RENAME_SWAP`, or `None` if unreported.
    pub const fn swap_rename_supported(self) -> Option<bool> {
        self.swap
    }
}

/// Cached Foundation volume support values for case-sensitive and case-preserved names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosVolumeNameSupportSnapshot {
    case_sensitive: Option<bool>,
    case_preserved: Option<bool>,
}

impl IosVolumeNameSupportSnapshot {
    /// Returns whether the volume reports case-sensitive names, or `None` if unreported.
    pub const fn case_sensitive_names_supported(self) -> Option<bool> {
        self.case_sensitive
    }

    /// Returns whether the volume reports case-preserved names, or `None` if unreported.
    pub const fn case_preserved_names_supported(self) -> Option<bool> {
        self.case_preserved
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

/// A point-in-time file identity and optional XNU data-generation count from one open descriptor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosFileDataGenerationSnapshot {
    identity: IosFileIdentitySnapshot,
    generation_count: Option<u32>,
}

impl IosFileDataGenerationSnapshot {
    /// Returns the device and inode pair reported for the same open descriptor.
    pub const fn identity(self) -> IosFileIdentitySnapshot {
        self.identity
    }

    /// Returns the nonzero generation count, or `None` when XNU reports its invalid zero value.
    pub const fn generation_count(self) -> Option<u32> {
        self.generation_count
    }
}

/// Point-in-time counts of direct directory entries by `FileKind` classification.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IosDirectoryEntryKindCounts {
    files: u64,
    directories: u64,
    other: u64,
}

impl IosDirectoryEntryKindCounts {
    /// Returns the count of regular-file entries.
    pub const fn files(self) -> u64 {
        self.files
    }

    /// Returns the count of directory entries.
    pub const fn directories(self) -> u64 {
        self.directories
    }

    /// Returns the count of symbolic links and other non-file, non-directory entries.
    pub const fn other(self) -> u64 {
        self.other
    }
}

/// A detailed point-in-time POSIX object kind for one iOS app-sandbox entry.
///
/// Unlike portable `FileKind`, this value distinguishes symbolic links and common special-file
/// types. `Unknown` contains the raw `st_mode & S_IFMT` bits for an unrecognized type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum IosEntryObjectKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link, not its target.
    Symlink,
    /// A FIFO (named pipe).
    Fifo,
    /// A local-domain or other socket entry.
    Socket,
    /// A block device entry.
    BlockDevice,
    /// A character device entry.
    CharacterDevice,
    /// An unrecognized raw `st_mode & S_IFMT` value.
    Unknown(u32),
}

/// A point-in-time physical allocation size reported for one iOS directory object.
///
/// This counts bytes used by the directory itself, not bytes in its children or descendants.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IosDirectoryAllocatedSizeSnapshot(u64);

impl IosDirectoryAllocatedSizeSnapshot {
    /// Returns the reported directory-object allocation in bytes.
    pub const fn bytes(self) -> u64 {
        self.0
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
        // SAFETY: these Foundation exports are immutable NSURL resource-key constants.
        let case_sensitive_key = unsafe { NSURLVolumeSupportsCaseSensitiveNamesKey };
        // SAFETY: these Foundation exports are immutable NSURL resource-key constants.
        let case_preserved_key = unsafe { NSURLVolumeSupportsCasePreservedNamesKey };
        // SAFETY: this Foundation export is an immutable NSURL resource-key constant.
        let file_cloning_key = unsafe { NSURLVolumeSupportsFileCloningKey };
        let rename_exclusive = urls.map(|url| volume_supports(url, exclusive_key));
        let rename_swap = urls.map(|url| volume_supports(url, swap_key));
        let case_sensitive_names = urls.map(|url| volume_supports(url, case_sensitive_key));
        let case_preserved_names = urls.map(|url| volume_supports(url, case_preserved_key));
        let volume_file_cloning = urls.map(|url| volume_supports(url, file_cloning_key));
        Ok(Self {
            documents,
            caches,
            temporary,
            application_support,
            rename_exclusive,
            rename_swap,
            case_sensitive_names,
            case_preserved_names,
            volume_file_cloning,
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

    /// Clones one regular app-sandbox file to a destination that must not exist.
    ///
    /// Both `AppPath` values use this backend's retained semantic roots and the existing
    /// descriptor-relative no-follow parent traversal. The source is opened with `O_NOFOLLOW`,
    /// then checked through its open descriptor to be a regular file. `fclonefileat` receives that
    /// descriptor and the destination parent descriptor, with `CLONE_NOFOLLOW_ANY` and
    /// `CLONE_RESOLVE_BENEATH` for destination resolution. XNU documents the call as expected to
    /// create the complete destination atomically or create no destination; an existing final
    /// entry is never replaced. The operation does not stage or copy file bytes through Rust
    /// memory.
    ///
    /// The result is a native copy-on-write clone: source and destination may share data blocks at
    /// first, while later writes to either file are private to that file. A later overwrite can
    /// still fail with `ENOSPC`. The syscall copies native file attributes and extended attributes;
    /// the destination inherits ACLs from its parent because this method does not set `CLONE_ACL`.
    /// Native owner and setuid/setgid handling also applies. This is not a byte-only copy, crash
    /// durability promise, or performance/storage guarantee. The filesystem may return
    /// `Unsupported` when it cannot clone; source and destination on distinct filesystems fail
    /// with the mapped `EXDEV` error. A `true` volume clone-support key would not ensure this call
    /// succeeds, so this method does not preflight that key.
    ///
    /// The call does not serialize concurrent source writes or destination-parent namespace
    /// changes. It inherits the backend's limit under a concurrent rename of an already-open
    /// parent directory: that descriptor can continue to refer to a directory moved outside the
    /// selected root. The method accepts no arbitrary URL, provider path, or security-scoped URL.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// `NotFound` for a missing source or parent, and `AlreadyExists` when the destination exists.
    /// A final source symlink is not followed. Filesystem, permission, cross-device, and other
    /// native errors use the existing POSIX mapping with the native code preserved.
    pub fn clone_regular_file(
        &self,
        source: AppPath<'_>,
        destination: AppPath<'_>,
    ) -> Result<(), FileError> {
        let source_parts = path_parts(source.relative())?;
        let destination_parts = path_parts(destination.relative())?;
        let (source_parent, source_leaf) =
            open_parent(self.root(source.directory())?, &source_parts)?;
        let (destination_parent, destination_leaf) =
            open_parent(self.root(destination.directory())?, &destination_parts)?;

        // SAFETY: `source_parent` is open and `source_leaf` is one validated component. The flags
        // reject a final source symlink and avoid blocking if a concurrent replacement is special.
        let source_fd = unsafe {
            libc::openat(
                source_parent.as_raw_fd(),
                source_leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let flags = CLONE_NOFOLLOW_ANY | CLONE_RESOLVE_BENEATH;
        // SAFETY: `source_file` is an open descriptor verified as a regular file, the destination
        // parent is an open directory descriptor, and `destination_leaf` is a validated single
        // NUL-terminated path component. The flags enforce no-follow/beneath destination lookup.
        let result = unsafe {
            libc::fclonefileat(
                source_file.as_raw_fd(),
                destination_parent.as_raw_fd(),
                destination_leaf.as_ptr(),
                flags,
            )
        };
        if result < 0 {
            Err(file_error(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    /// Returns the raw Darwin extended flags for one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMNEXT_EXT_FLAGS`. The result preserves unknown bits; named bits are the public
    /// `EF_*` values exposed by [`IosFileExtendedFlags`]. `EF_MAY_SHARE_BLOCKS` and
    /// `EF_SHARES_ALL_BLOCKS` describe whether this file may share blocks or shares all blocks
    /// with another file; they do not identify a particular clone peer or guarantee continued
    /// sharing. `EF_IS_SPARSE` reports a sparse-region flag, not an allocated-space total. Other
    /// flags are point-in-time metadata, not stable allocation or later-operation guarantees.
    /// Filesystems that do not support this attribute return `Unsupported`. This query reads no
    /// file contents, accepts no arbitrary URL, starts no security scope, and changes no portable
    /// `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem does not support the extended flags, or the mapped POSIX error for other
    /// failures.
    pub fn regular_file_extended_flags_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileExtendedFlags, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMNEXT_EXT_FLAGS,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended common attribute; `buffer` holds its u32 length and u64 value. The descriptor
        // binds the query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let flags = u64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileExtendedFlags(flags))
    }

    /// Returns an opaque point-in-time clone ID for one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMNEXT_CLONEID`. XNU defines equal clone IDs as a way to find pure clones that share a
    /// data stream. The value does not identify a particular B196 source path or prove current
    /// block sharing. It has no documented persistence or change-token guarantee and is not a
    /// content hash.
    /// Filesystems that do not support the attribute return `Unsupported`. This query reads no
    /// file contents, accepts no arbitrary URL, starts no security scope, and changes no portable
    /// `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem does not support clone IDs, or the mapped POSIX error for other failures.
    pub fn regular_file_clone_id_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileCloneIdSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMNEXT_CLONEID,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended common attribute; `buffer` holds its u32 length and u64 value. The descriptor
        // binds the query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let clone_id = u64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileCloneIdSnapshot(clone_id))
    }

    /// Returns an opaque link ID for one regular app-sandbox file entry on its mounted volume.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMNEXT_LINKID`. XNU defines a `u64` ID unique within a mounted volume; on HFS+ and
    /// APFS, a hard-link entry has a different link ID from the linked file-system object. This
    /// value is distinct from B99's `(st_dev, st_ino)` snapshot and B211's clone ID. It identifies
    /// neither file contents nor a pure-clone group. XNU says it is persistent only on volumes
    /// that support `VOL_CAP_FMT_PERSISTENTOBJECTIDS`; this method does not query that capability,
    /// so callers must not rely on cross-mount persistence. Filesystems that do not support the
    /// attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem does not support link IDs, or the mapped POSIX error for other failures.
    pub fn regular_file_link_id_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileLinkIdSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMNEXT_LINKID,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended common attribute; `buffer` holds its u32 length and u64 value. The descriptor
        // binds the query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let link_id = u64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileLinkIdSnapshot(link_id))
    }

    /// Returns the current number of full clones reported for one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMNEXT_CLONE_REFCNT`. XNU defines this `u32` value as the number of full clones, each
    /// sharing all of its blocks with this file. It does not count partial block-sharing peers,
    /// identify clone paths or IDs, or provide a stable identity. The value is a point-in-time
    /// filesystem report and may change as files are cloned or removed. Filesystems that do not
    /// support this attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File
    /// Timestamp required-reason API category; the host app must declare an applicable approved
    /// reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem does not support the clone reference count, or the mapped POSIX error for
    /// other failures.
    pub fn regular_file_full_clone_count_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileFullCloneCountSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: ATTR_CMNEXT_CLONE_REFCNT,
        };
        let mut buffer = [0_u8; 8];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended common attribute; `buffer` holds its u32 length and u32 value. The descriptor
        // binds the query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let count = u32::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileFullCloneCountSnapshot(count))
    }

    /// Returns the current private-size byte count for one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMNEXT_PRIVATESIZE`. XNU defines the `off_t` value as bytes not trapped inside a
    /// clone or snapshot that would be freed immediately if the file were deleted. This differs
    /// from allocated size, which may include shared blocks. The result is point-in-time metadata,
    /// not a reservation, quota, later-delete guarantee, or promise about a future write. Filesystems
    /// that do not support this attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// negative private-size value, or malformed attribute buffer, `NotFound` for a missing
    /// source or parent, `Unsupported` when the filesystem does not support this attribute, or the
    /// mapped POSIX error for other failures.
    pub fn regular_file_private_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFilePrivateSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMNEXT_PRIVATESIZE,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended common attribute; `buffer` holds its u32 length and 8-byte off_t. The
        // descriptor binds the query to the opened inode, so no path lookup or symlink traversal
        // occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let private_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if private_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFilePrivateSizeSnapshot(private_size as u64))
    }

    /// Returns the current logical byte count across all forks of one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_FILE_TOTALSIZE`. XNU defines this `off_t` value as the total number of logical bytes
    /// across all file forks. It may differ from a data-fork size and is not the size of a buffer
    /// returned by `FileBackend::read`. The result is point-in-time metadata; concurrent writes
    /// may change it. Filesystems that do not support the attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// negative total size, or malformed attribute buffer, `NotFound` for a missing source or
    /// parent, `Unsupported` when the filesystem does not support this attribute, or the mapped
    /// POSIX error for other failures.
    pub fn regular_file_total_fork_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileTotalForkSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: libc::ATTR_FILE_TOTALSIZE,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // file attribute; `buffer` holds its u32 length and 8-byte off_t. The descriptor binds the
        // query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let total_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if total_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileTotalForkSizeSnapshot(total_size as u64))
    }

    /// Returns the current filesystem-reported allocated byte count for one regular file's data fork.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_FILE_DATAALLOCSIZE`. XNU defines this `off_t` value as bytes on disk used by the data
    /// fork only. It does not include resource-fork allocation and is not a guarantee of exclusive
    /// physical-device storage. The result is point-in-time metadata; concurrent writes may change
    /// it. Filesystems that do not support the attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// negative allocation, or malformed attribute buffer, `NotFound` for a missing source or
    /// parent, `Unsupported` when the filesystem does not support this attribute, or the mapped
    /// POSIX error for other failures.
    pub fn regular_file_data_fork_allocated_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileDataForkAllocatedSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: libc::ATTR_FILE_DATAALLOCSIZE,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // file attribute; `buffer` holds its u32 length and 8-byte off_t. The descriptor binds the
        // query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let allocated_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if allocated_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileDataForkAllocatedSizeSnapshot(allocated_size as u64))
    }

    /// Returns the current filesystem-reported allocated byte count for one regular file's resource fork.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_FILE_RSRCALLOCSIZE`. XNU defines this `off_t` value as bytes on disk used by the
    /// resource fork only. It does not include data-fork allocation and is not a guarantee of
    /// exclusive physical-device storage. A zero result is the reported size only and does not
    /// establish that a resource fork is absent. The result is point-in-time metadata; concurrent
    /// writes may change it. Filesystems that do not support the attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// negative allocation, or malformed attribute buffer, `NotFound` for a missing source or
    /// parent, `Unsupported` when the filesystem does not support this attribute, or the mapped
    /// POSIX error for other failures.
    pub fn regular_file_resource_fork_allocated_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileResourceForkAllocatedSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: libc::ATTR_FILE_RSRCALLOCSIZE,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // file attribute; `buffer` holds its u32 length and 8-byte off_t. The descriptor binds the
        // query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let allocated_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if allocated_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileResourceForkAllocatedSizeSnapshot(
            allocated_size as u64,
        ))
    }

    /// Returns the current filesystem-reported logical byte count for one regular file's resource fork.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_FILE_RSRCLENGTH`. XNU defines this `off_t` value as the logical length of the resource
    /// fork in bytes. A zero result is only the reported length and does not establish that a
    /// resource fork is absent. The result is point-in-time metadata; concurrent writes may change
    /// it. Filesystems that do not support the attribute return `Unsupported`.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source,
    /// negative length, or malformed attribute buffer, `NotFound` for a missing source or parent,
    /// `Unsupported` when the filesystem does not support this attribute, or the mapped POSIX
    /// error for other failures.
    pub fn regular_file_resource_fork_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileResourceForkSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: libc::ATTR_FILE_RSRCLENGTH,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // file attribute; `buffer` holds its u32 length and 8-byte off_t. The descriptor binds the
        // query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let logical_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if logical_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileResourceForkSizeSnapshot(logical_size as u64))
    }

    /// Returns the current document ID reported for one regular app-sandbox file.
    ///
    /// The file is opened through the validated `AppPath` parent with `O_NOFOLLOW`, checked as a
    /// regular file, and queried by its open descriptor with `fgetattrlist` requesting
    /// `ATTR_CMN_DOCUMENT_ID` and `FSOPT_ATTR_CMN_EXTENDED`. XNU defines a nonzero `u32` document
    /// ID that is sticky to the path it was assigned to across safe saves; zero is invalid and
    /// becomes `None`. Filesystem support may vary. The value is a point-in-time path/document
    /// token, not an inode, content hash, clone ID, link ID, cross-volume ID, or durable identity.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem omits or does not support the attribute, or the mapped POSIX error for other
    /// failures.
    pub fn regular_file_document_id_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileDocumentIdSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMN_DOCUMENT_ID,
        };
        let mut buffer = [0_u8; 8];
        // SAFETY: `source_file` is an open regular file. `attributes` requests one documented
        // extended-common attribute; `buffer` holds its u32 length and u32 value. The descriptor
        // binds the query to the opened inode, so no path lookup or symlink traversal occurs here.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let document_id = u32::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileDocumentIdSnapshot(document_id))
    }

    /// Returns the opaque data-protection-class code for one opened app-sandbox file or directory.
    ///
    /// The validated `AppPath` is resolved with the existing no-follow descriptor traversal. The
    /// final entry is opened with `O_NOFOLLOW | O_NONBLOCK`; only regular files and directories
    /// are accepted. The open descriptor is queried with `fgetattrlist(ATTR_CMN_DATA_PROTECT_FLAGS)`.
    /// Apple documents the returned `u32` as the data-protection class, but does not publish a
    /// numeric mapping to named protection levels. `raw_value()` is for preservation or display
    /// only: do not compare it across objects or OS versions, map it to a named level, or infer
    /// current/future data access from it. The value is not an identity and is not a cross-call
    /// snapshot of a stable object.
    ///
    /// This query reads no file contents, accepts no arbitrary URL, starts no security scope, and
    /// changes no portable `FileBackend` behavior. Opening the entry can still fail, including due
    /// to access policy or current file state; such failure is not a class result. Apple lists
    /// `fgetattrlist` in the File Timestamp required-reason API category; the host app must declare
    /// an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a final symlink, an
    /// entry other than a regular file or directory, or a malformed attribute buffer, `NotFound`
    /// for a missing source or parent, `Unsupported` when the filesystem omits or does not support
    /// the attribute, or the mapped POSIX error for other failures.
    pub fn entry_data_protection_class_code(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileDataProtectionClassCode, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        let metadata = source_file.metadata().map_err(file_error)?;
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: libc::ATTR_CMN_DATA_PROTECT_FLAGS,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 8];
        // SAFETY: `source_file` is an open regular file or directory. `attributes` requests one
        // documented common u32 attribute, and `buffer` holds its u32 length and value. The
        // descriptor binds the query to the opened entry without another path lookup.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let code = u32::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileDataProtectionClassCode(code))
    }

    /// Returns one regular file's identity and XNU data-generation count from the same open
    /// descriptor.
    ///
    /// The method opens the validated `AppPath` with no-follow descriptor traversal. It reads the
    /// device/inode identity from `fstat` on that descriptor, then requests only
    /// `ATTR_CMN_GEN_COUNT` through `fgetattrlist` on the same descriptor. The generation count is
    /// an extended common attribute and requires `FSOPT_ATTR_CMN_EXTENDED`. XNU documents equality
    /// comparison only for the same filesystem object; its count is invalid while a file is
    /// memory-mapped and zero maps to `None`. The reported identity lets callers compare the
    /// generation only when the device/inode pair matches. It is not persistent and does not
    /// prevent inode reuse.
    ///
    /// This is a point-in-time metadata snapshot, not a general content-change token. The query
    /// reads no contents, accepts no arbitrary URL, starts no security scope, and does not change
    /// portable `FileBackend` behavior. Apple lists `fgetattrlist` in the File Timestamp
    /// required-reason API category; the host app must declare an applicable approved reason in
    /// `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a non-regular source or
    /// malformed attribute buffer, `NotFound` for a missing source or parent, `Unsupported` when
    /// the filesystem omits or does not support any requested attribute, or the mapped POSIX
    /// error for other failures.
    pub fn regular_file_data_generation_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileDataGenerationSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        let metadata = source_file.metadata().map_err(file_error)?;
        if !metadata.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let device_id = metadata.dev();
        let inode_number = metadata.ino();

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: libc::ATTR_CMN_GEN_COUNT,
        };
        let mut buffer = [0_u8; 8];
        // SAFETY: `source_file` is an open regular file. The request contains one documented
        // extended-common u32 attribute, and `buffer` holds its u32 length and value. The
        // descriptor binds this query to the same object as the `fstat` identity above.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::FSOPT_ATTR_CMN_EXTENDED,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let generation_count = u32::from_ne_bytes(
            buffer[4..8]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        Ok(IosFileDataGenerationSnapshot {
            identity: IosFileIdentitySnapshot {
                device_id,
                inode_number,
            },
            generation_count: (generation_count != 0).then_some(generation_count),
        })
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

    /// Returns the filesystem-reported volume space available to non-superusers, in bytes.
    ///
    /// This multiplies `fstatfs`'s `f_bavail` block count by `f_bsize` for the retained root that
    /// backs the selected semantic directory. The value describes the volume, not an app-specific
    /// quota or a reservation; another process can consume space, and a later write can still
    /// fail. Apple lists `fstatfs` as a required-reason Disk Space API. The app or SDK that uses
    /// this method must declare an approved privacy-manifest reason that matches its actual use
    /// and follow that reason's data limits. This iOS-only query reads no file contents, accepts no
    /// arbitrary URL, starts no security scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for an unsupported semantic directory, the mapped POSIX error for
    /// `fstatfs` failure, `InvalidInput` for a zero block size, or `ResourceExhausted` if the byte
    /// product overflows `u64`.
    pub fn volume_available_capacity_bytes(
        &self,
        directory: AppDirectory,
    ) -> Result<u64, FileError> {
        let root = self.root(directory)?;
        let mut filesystem = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: `root` is an open descriptor and `filesystem` is writable storage.
        let result = unsafe { libc::fstatfs(root.as_raw_fd(), filesystem.as_mut_ptr()) };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatfs` initialized the structure.
        let filesystem = unsafe { filesystem.assume_init() };
        let block_size = u64::from(filesystem.f_bsize);
        if block_size == 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        filesystem
            .f_bavail
            .checked_mul(block_size)
            .ok_or_else(|| backend_error(ErrorKind::ResourceExhausted, None))
    }

    /// Returns the filesystem-reported total data capacity of a volume, in bytes.
    ///
    /// This multiplies `fstatfs`'s `f_blocks` by `f_bsize` for the retained root that backs the
    /// selected semantic directory. It describes the mounted volume, not physical device
    /// capacity, an app-container quota, or space reserved for this app. The value is a
    /// point-in-time filesystem report and may not match a later available-space snapshot. Apple
    /// lists `fstatfs` as a required-reason Disk Space API; the app or SDK that uses this method
    /// must declare an approved privacy-manifest reason that matches its actual use and follow
    /// that reason's data limits. This iOS-only query reads no file contents, accepts no arbitrary
    /// URL, starts no security scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for an unsupported semantic directory, the mapped POSIX error for
    /// `fstatfs` failure, `InvalidInput` for a zero block size, or `ResourceExhausted` if the byte
    /// product overflows `u64`.
    pub fn volume_total_capacity_bytes(&self, directory: AppDirectory) -> Result<u64, FileError> {
        let root = self.root(directory)?;
        let mut filesystem = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: `root` is an open descriptor and `filesystem` is writable storage.
        let result = unsafe { libc::fstatfs(root.as_raw_fd(), filesystem.as_mut_ptr()) };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatfs` initialized the structure.
        let filesystem = unsafe { filesystem.assume_init() };
        let block_size = u64::from(filesystem.f_bsize);
        if block_size == 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        filesystem
            .f_blocks
            .checked_mul(block_size)
            .ok_or_else(|| backend_error(ErrorKind::ResourceExhausted, None))
    }

    /// Returns the filesystem-reported total space used on a volume, in bytes.
    ///
    /// On the retained semantic-root descriptor, this first requests
    /// `ATTR_VOL_INFO | ATTR_VOL_ATTRIBUTES` and requires `validattr.volattr` to include
    /// `ATTR_VOL_SPACEUSED`. It then requests `ATTR_VOL_INFO | ATTR_VOL_SPACEUSED` on that same
    /// descriptor and parses the returned `off_t`. An unsupported volume or omitted value returns
    /// `Unsupported`; a negative or malformed value returns `InvalidInput`.
    ///
    /// This is a point-in-time, volume-wide value. XNU warns that on space-sharing volumes it may
    /// differ from volume size minus free space, so do not derive it from B137 or B146. It is not
    /// app/container use, a quota, physical-device use, a reservation, or a guarantee that a later
    /// write will succeed. The support and value queries are separate calls and do not form one
    /// atomic snapshot. Apple lists `fgetattrlist` in the File Timestamp required-reason API
    /// category; the app or SDK that uses this method must declare an applicable approved reason
    /// in `PrivacyInfo.xcprivacy`.
    ///
    /// This iOS-only query reads no file contents, accepts no arbitrary URL, starts no security
    /// scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for an unsupported semantic directory, an unadvertised volume
    /// attribute, an omitted value, or a native unsupported-attribute error; `InvalidInput` for a
    /// negative value or malformed attribute buffer; or the mapped POSIX error for other failures.
    pub fn volume_used_capacity_bytes(&self, directory: AppDirectory) -> Result<u64, FileError> {
        let root = self.root(directory)?;
        let mut support_attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: libc::ATTR_VOL_INFO | libc::ATTR_VOL_ATTRIBUTES,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut support_buffer =
            [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::vol_attributes_attr_t>()];
        // SAFETY: `root` is an open semantic-root descriptor. The request contains only the
        // fixed-size volume-attributes structure and `support_buffer` holds its length and payload.
        let result = unsafe {
            libc::fgetattrlist(
                root.as_raw_fd(),
                (&mut support_attributes as *mut libc::attrlist).cast(),
                support_buffer.as_mut_ptr().cast(),
                support_buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        if parse_volume_valid_attribute_masks(&support_buffer)?.1 & libc::ATTR_VOL_SPACEUSED == 0 {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: libc::ATTR_VOL_INFO | libc::ATTR_VOL_SPACEUSED,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::off_t>()];
        // SAFETY: `root` is the same open descriptor used for the support query. XNU defines the
        // fixed-size `ATTR_VOL_SPACEUSED` payload as `off_t`; the buffer fits its length and value.
        let result = unsafe {
            libc::fgetattrlist(
                root.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        parse_volume_off_t_attribute(&buffer)
    }

    /// Returns whether the mounted volume for a semantic app directory reports a read-only mount.
    ///
    /// This checks `fstatfs`'s `f_flags` for `MNT_RDONLY` on the retained directory-root
    /// descriptor. It describes the volume mount only; `false` does not establish that this app
    /// can write a particular path, because sandbox policy, directory permissions, file
    /// protection, file flags, quotas, and available space can still prevent a write. The result
    /// is point-in-time and does not reserve a later operation. Apple lists `fstatfs` as a
    /// required-reason Disk Space API, so the app or SDK using this method must declare an
    /// approved privacy-manifest reason matching its actual use and follow that reason's data
    /// limits. This iOS-only query reads no file contents, accepts no arbitrary URL, starts no
    /// security scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for an unsupported semantic directory or the mapped POSIX error if
    /// `fstatfs` fails.
    pub fn volume_is_read_only(&self, directory: AppDirectory) -> Result<bool, FileError> {
        let root = self.root(directory)?;
        let mut filesystem = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: `root` is an open descriptor and `filesystem` is writable storage.
        let result = unsafe { libc::fstatfs(root.as_raw_fd(), filesystem.as_mut_ptr()) };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatfs` initialized the structure.
        let filesystem = unsafe { filesystem.assume_init() };
        Ok(filesystem.f_flags & libc::MNT_RDONLY as u32 != 0)
    }

    /// Returns the filesystem-reported optimal transfer block size, in bytes, for a volume.
    ///
    /// This returns `fstatfs`'s `f_iosize` for the retained root that backs the selected semantic
    /// directory. It is a filesystem-provided sizing hint for caller-managed I/O, not a required
    /// alignment, buffer-size mandate, throughput guarantee, or claim that using this size
    /// improves performance for a particular workload. It is a point-in-time volume value. Apple
    /// lists `fstatfs` as a required-reason Disk Space API, so the app or SDK using this method must
    /// declare an approved privacy-manifest reason matching its actual use and follow that
    /// reason's data limits. This iOS-only query reads no file contents, accepts no arbitrary URL,
    /// starts no security scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for an unsupported semantic directory, the mapped POSIX error if
    /// `fstatfs` fails, or `InvalidInput` if the filesystem reports a nonpositive transfer size.
    pub fn volume_optimal_io_block_size_bytes(
        &self,
        directory: AppDirectory,
    ) -> Result<u32, FileError> {
        let root = self.root(directory)?;
        let mut filesystem = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: `root` is an open descriptor and `filesystem` is writable storage.
        let result = unsafe { libc::fstatfs(root.as_raw_fd(), filesystem.as_mut_ptr()) };
        if result < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: successful `fstatfs` initialized the structure.
        let filesystem = unsafe { filesystem.assume_init() };
        u32::try_from(filesystem.f_iosize)
            .ok()
            .filter(|size| *size != 0)
            .ok_or_else(|| backend_error(ErrorKind::InvalidInput, None))
    }

    /// Returns the maximum filename-component length in bytes reported for an app-directory root.
    ///
    /// This queries `fpathconf` with `_PC_NAME_MAX` on the retained root descriptor. The value
    /// applies to direct child names in that root only; it does not report a total path limit or
    /// establish a limit for a nested directory. `None` means the filesystem returned the
    /// POSIX no-limit sentinel (`-1` without setting `errno`). A reported value is
    /// a filesystem limit snapshot, not proof that a later create or rename will succeed. This
    /// iOS-only query reads no file contents, accepts no arbitrary URL, starts no security scope,
    /// and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns the mapped POSIX error if `fpathconf` fails, `Unsupported` for an unsupported
    /// semantic directory, or `InvalidInput` for an unexpected negative result other than the
    /// documented indeterminate sentinel.
    pub fn app_directory_name_max_bytes(
        &self,
        directory: AppDirectory,
    ) -> Result<Option<u64>, FileError> {
        let root = self.root(directory)?;
        // SAFETY: `__error` returns this thread's errno slot.
        unsafe { *libc::__error() = 0 };
        // SAFETY: `root` is an open directory descriptor and `_PC_NAME_MAX` is a valid query.
        let value = unsafe { libc::fpathconf(root.as_raw_fd(), libc::_PC_NAME_MAX) };
        if value == -1 {
            // SAFETY: `__error` returns this thread's errno slot.
            let code = unsafe { *libc::__error() };
            return if code == 0 {
                Ok(None)
            } else {
                Err(file_error(io::Error::from_raw_os_error(code)))
            };
        }
        u64::try_from(value)
            .map(Some)
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))
    }

    /// Returns the filesystem's case-sensitivity property for one retained app-directory root.
    ///
    /// This queries `fpathconf` with `_PC_CASE_SENSITIVE` on the retained Documents, Caches,
    /// Temporary, or Application Support descriptor. `Some(false)` represents a zero result;
    /// `Some(true)` represents the Darwin filesystem Boolean encodings `1` or `-1` (legacy
    /// FSKit uses `-1` for Boolean true). `None` means the filesystem does not associate this
    /// property with the descriptor and returned `EINVAL`. The result is a point-in-time
    /// filesystem property. It does not define Unicode normalization or collation, guarantee
    /// whether any particular pair of names collides, reserve a name, or guarantee a later
    /// create or rename. It does not change portable `AppPath` comparison or validation. This
    /// iOS-only query reads no file contents, accepts no arbitrary URL, starts no security scope,
    /// and adds no required-reason privacy API.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for a semantic directory that this backend does not retain, the
    /// mapped POSIX error for other `fpathconf` failures, or `InvalidInput` for an unexpected
    /// result encoding.
    pub fn app_directory_case_sensitivity(
        &self,
        directory: AppDirectory,
    ) -> Result<Option<bool>, FileError> {
        let root = self.root(directory)?;
        // SAFETY: `__error` returns this thread's errno slot.
        unsafe { *libc::__error() = 0 };
        // SAFETY: `root` is an open directory descriptor and `_PC_CASE_SENSITIVE` is a valid
        // Darwin query.
        let value = unsafe { libc::fpathconf(root.as_raw_fd(), libc::_PC_CASE_SENSITIVE) };
        match value {
            0 => Ok(Some(false)),
            1 => Ok(Some(true)),
            -1 => {
                // SAFETY: `__error` returns this thread's errno slot.
                let code = unsafe { *libc::__error() };
                if code == 0 {
                    // Legacy FSKit pathconf encodes a Boolean true as -1; this selector is a
                    // Boolean property, not a numeric limit with a no-limit sentinel.
                    Ok(Some(true))
                } else if code == libc::EINVAL {
                    Ok(None)
                } else {
                    Err(file_error(io::Error::from_raw_os_error(code)))
                }
            }
            _ => Err(backend_error(ErrorKind::InvalidInput, None)),
        }
    }

    /// Reports whether a filesystem may truncate overlong names for one retained app root.
    ///
    /// This queries `fpathconf` with `_PC_NO_TRUNC` on the retained Documents, Caches,
    /// Temporary, or Application Support descriptor. `Some(true)` means the filesystem may
    /// truncate a component longer than its `_PC_NAME_MAX` value; `Some(false)` means the
    /// filesystem preserves an overlong name so the path operation returns its native error,
    /// normally `ENAMETOOLONG`. `None` means this filesystem does not associate the property with
    /// the descriptor and returned `EINVAL`. A legacy FSKit Boolean `true` encoding of `-1` with
    /// unchanged `errno` is also reported as `Some(true)`.
    ///
    /// AppPath traversal checks this property on each opened parent. If the property reports
    /// possible truncation, traversal also queries that parent's `_PC_NAME_MAX` and rejects a
    /// longer component before a syscall could target a different name. If `_PC_NO_TRUNC` is
    /// unsupported, traversal fails closed with `Unsupported`. This snapshot is point-in-time,
    /// does not define Unicode normalization or collation, and does not guarantee a later file
    /// operation. It reads no file contents, accepts no arbitrary URL, starts no security scope,
    /// and adds no required-reason privacy API.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for a semantic directory that this backend does not retain,
    /// `InvalidInput` for an unexpected result encoding, or the mapped POSIX error if `fpathconf`
    /// fails.
    pub fn app_directory_truncates_long_names(
        &self,
        directory: AppDirectory,
    ) -> Result<Option<bool>, FileError> {
        let root = self.root(directory)?;
        query_name_truncation(root)
    }

    /// Returns the cached rename-option support values for one semantic app-directory volume.
    ///
    /// `IosFiles::new` reads Foundation's `NSURLVolumeSupportsExclusiveRenamingKey` and
    /// `NSURLVolumeSupportsSwapRenamingKey` values for the four retained app-directory roots.
    /// `Some(false)` means Foundation reported that the option is not supported; `None` means
    /// that the resource value could not be read or did not contain an `NSNumber`. The result is
    /// the value cached at construction, not a fresh volume query. Support for `RENAME_EXCL` or
    /// `RENAME_SWAP` does not guarantee a later path operation or replace the actual operation's
    /// result. This iOS-only snapshot reads no file contents, accepts no arbitrary URL, starts no
    /// security scope, and changes no portable `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for a semantic directory that this backend does not retain.
    pub fn volume_rename_support_snapshot(
        &self,
        directory: AppDirectory,
    ) -> Result<IosVolumeRenameSupportSnapshot, FileError> {
        let index = match directory {
            AppDirectory::Documents => 0,
            AppDirectory::Caches => 1,
            AppDirectory::Temporary => 2,
            AppDirectory::ApplicationSupport => 3,
            _ => return Err(backend_error(ErrorKind::Unsupported, None)),
        };
        Ok(IosVolumeRenameSupportSnapshot {
            exclusive: self.rename_exclusive[index],
            swap: self.rename_swap[index],
        })
    }

    /// Returns Foundation's cached volume-cloning support value for one app-directory volume.
    ///
    /// `IosFiles::new` reads `NSURLVolumeSupportsFileCloningKey` for each of its four retained
    /// roots. `Some(true)` or `Some(false)` is Foundation's Boolean report; `None` means the
    /// resource query failed or did not contain an `NSNumber`. This is a cached volume-level hint,
    /// not a guarantee that a particular source/destination pair or a future
    /// [`Self::clone_regular_file`] call will succeed. Cross-volume errors, current filesystem
    /// state, and the clone syscall's flags remain authoritative. The snapshot reads no file
    /// contents, accepts no arbitrary URL, starts no security scope, and changes no portable
    /// `FileBackend` behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for a semantic directory that this backend does not retain.
    pub fn volume_clone_support_snapshot(
        &self,
        directory: AppDirectory,
    ) -> Result<Option<bool>, FileError> {
        let index = match directory {
            AppDirectory::Documents => 0,
            AppDirectory::Caches => 1,
            AppDirectory::Temporary => 2,
            AppDirectory::ApplicationSupport => 3,
            _ => return Err(backend_error(ErrorKind::Unsupported, None)),
        };
        Ok(self.volume_file_cloning[index])
    }

    /// Returns cached case-name support values for one semantic app-directory volume.
    ///
    /// `IosFiles::new` reads Foundation's `NSURLVolumeSupportsCaseSensitiveNamesKey` and
    /// `NSURLVolumeSupportsCasePreservedNamesKey` values for the four retained app-directory
    /// roots. `Some(false)` means Foundation reported that the volume does not support that
    /// property; `None` means the value could not be read or did not contain an `NSNumber`. The
    /// result is cached at construction, not a fresh volume query. These values do not define
    /// Unicode normalization or collation, reserve a name, or guarantee that a later operation
    /// will succeed. This iOS-only snapshot reads no file contents, accepts no arbitrary URL,
    /// starts no security scope, and changes no portable `FileBackend` path behavior.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` for a semantic directory that this backend does not retain.
    pub fn volume_name_support_snapshot(
        &self,
        directory: AppDirectory,
    ) -> Result<IosVolumeNameSupportSnapshot, FileError> {
        let index = match directory {
            AppDirectory::Documents => 0,
            AppDirectory::Caches => 1,
            AppDirectory::Temporary => 2,
            AppDirectory::ApplicationSupport => 3,
            _ => return Err(backend_error(ErrorKind::Unsupported, None)),
        };
        Ok(IosVolumeNameSupportSnapshot {
            case_sensitive: self.case_sensitive_names[index],
            case_preserved: self.case_preserved_names[index],
        })
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

    /// Counts direct entries in one app-sandbox directory without building a listing.
    ///
    /// The count includes every name except `.` and `..`, regardless of entry kind or name
    /// encoding. It uses the same validated `AppPath`, retained root descriptor, and no-follow
    /// directory traversal as `read_directory`, then scans with `readdir` without building a
    /// Rust `Vec<DirectoryEntry>`, copying names to per-entry `String`s, or looking up each
    /// entry's kind. The call is synchronous and takes time proportional to the number of entries;
    /// libc may allocate memory for its directory stream. Concurrent namespace mutation can
    /// affect which names are observed; the result is not an atomic snapshot, a
    /// reservation, or authorization to remove the directory. This iOS-only query reads no file
    /// contents, follows no final symlink, starts no security scope, and preserves the documented
    /// concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns the mapped POSIX error for path traversal, directory opening, enumeration, or
    /// close failure, and `ResourceExhausted` if the count cannot fit in `u64`.
    pub fn directory_entry_count(&self, path: AppPath<'_>) -> Result<u64, FileError> {
        let parts = path_parts(path.relative())?;
        let directory = open_directory(self.root(path.directory())?, &parts)?;
        scan_directory_entries(&directory, false)
    }

    /// Reports whether a directory scan finds any direct entry besides `.` and `..`.
    ///
    /// This uses the same validated `AppPath`, retained root descriptor, and no-follow directory
    /// traversal as `read_directory`, then stops at the first child name. It can avoid a full scan
    /// when the directory is nonempty, but libc may allocate memory for its directory stream.
    /// Concurrent namespace mutation can affect the result. `true` is not a reservation or proof
    /// that a later removal will succeed; `remove_directory` remains authoritative. This iOS-only
    /// query reads no file contents, follows no final symlink, starts no security scope, and keeps
    /// the documented concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns the mapped POSIX error for path traversal, directory opening, enumeration, or
    /// close failure.
    pub fn directory_is_empty(&self, path: AppPath<'_>) -> Result<bool, FileError> {
        let parts = path_parts(path.relative())?;
        let directory = open_directory(self.root(path.directory())?, &parts)?;
        scan_directory_entries(&directory, true).map(|count| count == 0)
    }

    /// Counts direct entries by the no-follow kind used by `read_directory`.
    ///
    /// The returned fixed-width fields count regular files, directories, and all other entry
    /// kinds. The query uses the same validated path and no-follow directory traversal as
    /// `read_directory`, then calls `fstatat(..., AT_SYMLINK_NOFOLLOW)` for each observed name. It
    /// does not decode or copy names to `String`s or build a `Vec<DirectoryEntry>`. Concurrent
    /// namespace mutation can affect the result or make a name lookup fail; the counts are not an
    /// atomic snapshot, reservation, or basis to remove the directory. The call is synchronous;
    /// libc may allocate memory for the directory stream. This iOS-only query reads no file
    /// contents, follows no final symlink, starts no security scope, and keeps the documented
    /// concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns the mapped POSIX error for path traversal, directory opening, entry metadata, or
    /// close failure, and `ResourceExhausted` if a category count cannot fit in `u64`.
    pub fn directory_entry_kind_counts(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosDirectoryEntryKindCounts, FileError> {
        let parts = path_parts(path.relative())?;
        let directory = open_directory(self.root(path.directory())?, &parts)?;
        scan_directory_entry_kind_counts(&directory)
    }

    /// Returns the current on-disk allocation reported for one app-sandbox directory object.
    ///
    /// This opens the validated `AppPath` with the backend's descriptor-relative no-follow
    /// directory traversal, then requests `ATTR_DIR_ALLOCSIZE` by `fgetattrlist` on the open
    /// directory descriptor. XNU defines this `off_t` value as bytes on disk used by the directory
    /// itself; it is not the aggregate size of child files or descendants. The result is
    /// filesystem-specific point-in-time metadata, not an app quota, storage reservation, or
    /// performance signal. Filesystems that do not support the attribute return `Unsupported`.
    ///
    /// This iOS-only query reads no file contents, accepts no arbitrary URL, and starts no security
    /// scope. Apple lists `fgetattrlist` in the File Timestamp required-reason API category; the
    /// host app must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual
    /// use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `NotFound` for a missing path,
    /// `InvalidInput` for a negative allocation or malformed attribute buffer, `Unsupported` when
    /// the filesystem does not support directory allocation size, or the mapped POSIX error for
    /// other failures.
    pub fn directory_allocated_size_snapshot(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosDirectoryAllocatedSizeSnapshot, FileError> {
        let parts = path_parts(path.relative())?;
        let directory = open_directory(self.root(path.directory())?, &parts)?;
        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: 0,
            dirattr: libc::ATTR_DIR_ALLOCSIZE,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; 12];
        // SAFETY: `directory` is an open directory descriptor and `attributes` requests one
        // documented directory attribute. `buffer` holds its u32 length and 8-byte off_t; the
        // descriptor binds the query to the opened directory inode.
        let result = unsafe {
            libc::fgetattrlist(
                directory.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length = u32::from_ne_bytes(
            buffer[..4]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ) as usize;
        if returned_length == std::mem::size_of::<u32>() {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }
        if returned_length != buffer.len() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        let allocated_size = i64::from_ne_bytes(
            buffer[4..]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        );
        if allocated_size < 0 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosDirectoryAllocatedSizeSnapshot(allocated_size as u64))
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

    /// Returns the detailed no-follow POSIX object kind of one app-sandbox entry.
    ///
    /// This uses the same validated `AppPath` and parent traversal as `entry_kind`, then reads
    /// `st_mode & S_IFMT` with one `fstatat(..., AT_SYMLINK_NOFOLLOW)` call. It distinguishes the
    /// common special types that portable `FileKind::Other` groups together. `Unknown` preserves
    /// the raw mode-type bits. The method does not open or follow the entry and reads no contents;
    /// in particular, a FIFO is classified without opening it. The result is point-in-time and
    /// does not reserve the path for a later operation. It keeps the documented concurrent
    /// opened-parent directory-rename limit and adds no portable `FileBackend` behavior.
    /// Apple lists `fstatat` under the File Timestamp required-reason API category; the host app
    /// must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `NotFound` for a missing final entry, or
    /// the mapped POSIX error for parent traversal and metadata lookup failures.
    pub fn entry_object_kind(&self, path: AppPath<'_>) -> Result<IosEntryObjectKind, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        self::entry_object_kind(parent.as_raw_fd(), leaf.as_c_str()).map_err(file_error)
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

    /// Returns one non-symlink entry's no-follow POSIX access timestamp.
    ///
    /// The timestamp is the seconds/nanoseconds pair from `st_atime` and `st_atime_nsec` in one
    /// `fstatat(..., AT_SYMLINK_NOFOLLOW)` result. Apple documents this field as the time file data
    /// was last accessed and lists `mknod`, `utimes`, and `read` as operations that change it. A
    /// final symbolic link is rejected because Apple's `lstat` contract does not define its own
    /// timestamp fields. The query uses the same validated `AppPath` and no-follow parent
    /// traversal as other sandbox methods; it reads no target contents and accepts regular files,
    /// directories, and special entries. Filesystems may report coarser precision; this lookup
    /// cannot establish whether every data access refreshes the field. The field can also be
    /// explicitly set, so this is a point-in-time filesystem value, not an access log, content
    /// version, reliable change token, or durability proof. Another handle may change or replace
    /// the entry before a later operation. This method does not accept arbitrary URLs, start a
    /// security scope, or establish stronger containment than the documented concurrent
    /// directory-rename limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a final symbolic link or
    /// malformed timestamp field, or the mapped POSIX error for path traversal and metadata lookup
    /// failures.
    pub fn entry_access_time(&self, path: AppPath<'_>) -> Result<IosFileAccessTime, FileError> {
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
        let seconds_since_unix_epoch = metadata.st_atime;
        let nanoseconds = u32::try_from(metadata.st_atime_nsec)
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
        if nanoseconds >= 1_000_000_000 {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosFileAccessTime {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns the filesystem-reported time one regular file was created or renamed into its
    /// containing directory.
    ///
    /// The query requests `ATTR_CMN_ADDEDTIME` with `fgetattrlist` on an opened regular-file
    /// descriptor, after the usual validated `AppPath` and no-follow parent traversal. Apple's
    /// XNU documentation warns that this attribute may be inconsistent for hard-linked items, so
    /// the result is not a reliable path-history or creation-time record. It is a point-in-time
    /// filesystem value; another handle may change or replace the entry before a later operation.
    /// This method reads no contents, accepts no arbitrary URL, starts no security scope, and has
    /// the documented concurrent directory-rename containment limit.
    ///
    /// Apple lists `fgetattrlist` in the File Timestamp required-reason API category. The host app
    /// must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing source or parent, `InvalidInput` for a non-regular source
    /// or malformed timestamp buffer, `Unsupported` when the filesystem does not report this
    /// attribute, or the mapped POSIX error for other failures.
    pub fn entry_added_time(&self, path: AppPath<'_>) -> Result<IosFileAddedTime, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: libc::ATTR_CMN_ADDEDTIME,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::timespec>()];
        // SAFETY: `source_file` is an open regular file. The request contains one documented
        // common attribute, and `buffer` fits its u32 length plus a `timespec`. The descriptor
        // binds the query to the open file, with no later path lookup or symlink traversal.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let (seconds_since_unix_epoch, nanoseconds) = parse_timespec_attribute(&buffer)?;
        Ok(IosFileAddedTime {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns the filesystem-stored backup-time marker for one app-sandbox file or directory.
    ///
    /// The method requests `ATTR_CMN_BKUPTIME` with `fgetattrlist` on an opened descriptor after
    /// validated `AppPath` and no-follow parent traversal. It reports only the filesystem's
    /// stored `timespec`. XNU documents this field for backup utilities and says the filesystem
    /// stores but does not interpret it; this value does not prove OS or iCloud backup completion,
    /// inclusion, or freshness. The query does not set the marker or perform a backup. Filesystem
    /// support and precision may vary, and the result is point-in-time. This method reads no file
    /// contents, accepts no arbitrary URL, starts no security scope, and retains the documented
    /// concurrent opened-parent directory-rename limit.
    ///
    /// Apple lists `fgetattrlist` in the File Timestamp required-reason API category. The host app
    /// must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a final symlink, an
    /// entry other than a regular file or directory, or a malformed attribute buffer, `NotFound`
    /// for a missing entry or parent, `Unsupported` when the filesystem omits or does not support
    /// the attribute, or the mapped POSIX error for other failures.
    pub fn entry_stored_backup_time(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileBackupTimeMarker, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        let metadata = source_file.metadata().map_err(file_error)?;
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: libc::ATTR_CMN_BKUPTIME,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::timespec>()];
        // SAFETY: `source_file` is an open regular file or directory. The request contains one
        // documented common `timespec` attribute, and `buffer` fits its length and payload. The
        // descriptor binds the query to the opened entry without another path lookup.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let (seconds_since_unix_epoch, nanoseconds) = parse_timespec_attribute(&buffer)?;
        Ok(IosFileBackupTimeMarker {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns the filesystem-reported creation timestamp for one app-sandbox regular file.
    ///
    /// The method opens the validated `AppPath` without following a final symlink. On that open
    /// descriptor, it first requests `ATTR_VOL_INFO | ATTR_VOL_ATTRIBUTES` and requires the
    /// volume's `validattr.commonattr` to include `ATTR_CMN_CRTIME`. It then requests
    /// `ATTR_CMN_CRTIME` on the same descriptor and returns the filesystem's `timespec`. If the
    /// volume does not advertise support, the method returns `Unsupported`; it does not infer a
    /// creation time from `st_birthtime`.
    ///
    /// XNU defines `ATTR_CMN_CRTIME` as the time the filesystem object was created and marks it
    /// read/write through `setattrlist`. The reported value is therefore mutable metadata, a
    /// point-in-time observation, and not immutable proof of the real-world creation event. The
    /// support-mask query and timestamp query are separate calls, not an atomic metadata snapshot.
    /// Filesystem precision may be coarser than one nanosecond. This method reads no file contents,
    /// accepts no arbitrary URL, starts no security scope, and retains B1's concurrent opened-parent
    /// directory-rename limit.
    ///
    /// Apple lists `fgetattrlist` in the File Timestamp required-reason API category. The host app
    /// must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPath` for a malformed `AppPath`, `InvalidInput` for a final symlink, an
    /// entry other than a regular file, or a malformed attribute buffer, `NotFound` for a missing
    /// entry or parent, `Unsupported` when the volume does not advertise `ATTR_CMN_CRTIME` or
    /// omits the requested value, or the mapped POSIX error for other failures.
    pub fn regular_file_creation_time(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosFileCreationTime, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        // SAFETY: `parent` is open and `leaf` is one validated component. `O_NOFOLLOW` rejects a
        // final symlink, and `O_NONBLOCK` avoids blocking if the entry is concurrently replaced.
        let source_fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        if source_fd < 0 {
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let source_file = unsafe { File::from_raw_fd(source_fd) };
        if !source_file.metadata().map_err(file_error)?.is_file() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        let mut volume_attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: 0,
            volattr: libc::ATTR_VOL_INFO | libc::ATTR_VOL_ATTRIBUTES,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut volume_buffer =
            [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::vol_attributes_attr_t>()];
        // SAFETY: `source_file` is an open regular file. The volume request has no variable-size
        // values; `volume_buffer` fits the leading length and `vol_attributes_attr_t`. The
        // descriptor binds the query to the opened entry's mounted volume.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut volume_attributes as *mut libc::attrlist).cast(),
                volume_buffer.as_mut_ptr().cast(),
                volume_buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        if parse_volume_common_valid_attributes(&volume_buffer)? & libc::ATTR_CMN_CRTIME == 0 {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }

        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: libc::ATTR_CMN_CRTIME,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u8; std::mem::size_of::<u32>() + std::mem::size_of::<libc::timespec>()];
        // SAFETY: `source_file` is the same open regular-file descriptor used for the support
        // query. The request contains one documented common `timespec` attribute, and `buffer`
        // fits its length and payload.
        let result = unsafe {
            libc::fgetattrlist(
                source_file.as_raw_fd(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                0,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP)) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let (seconds_since_unix_epoch, nanoseconds) = parse_timespec_attribute(&buffer)?;
        Ok(IosFileCreationTime {
            seconds_since_unix_epoch,
            nanoseconds,
        })
    }

    /// Returns one non-symlink entry's raw Darwin BSD file flags.
    ///
    /// The `u32` bit set comes from one `fstatat(..., AT_SYMLINK_NOFOLLOW)` result. It is exposed
    /// as [`IosBsdFileFlags`] so known SDK masks can be checked while unknown bits remain
    /// available through `bits()`. A final symbolic link is rejected because Apple's `lstat`
    /// contract does not define link-owned file flags. The query uses the same validated `AppPath`
    /// and no-follow parent traversal as other sandbox methods; it accepts regular files,
    /// directories, and special entries. Some recognized flags can restrict changes or act as
    /// display hints, but their presence is not a complete access check: ACLs, POSIX permissions,
    /// sandbox policy, and filesystem behavior also matter. This is a point-in-time metadata
    /// snapshot and another handle may change or replace the entry before a later operation. The
    /// method reads no contents, accepts no arbitrary URL, starts no security scope, and has the
    /// documented concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a final symbolic link, or
    /// the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn entry_bsd_file_flags(&self, path: AppPath<'_>) -> Result<IosBsdFileFlags, FileError> {
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
        Ok(IosBsdFileFlags(metadata.st_flags))
    }

    /// Returns one non-symlink entry's raw POSIX owner and group IDs.
    ///
    /// Both values come from one `fstatat(..., AT_SYMLINK_NOFOLLOW)` result: `st_uid` is the
    /// numeric owner ID and `st_gid` is the numeric group ID. Apple's `lstat` contract does not
    /// define link-owned IDs, so a final symbolic link returns `InvalidInput`. The query accepts
    /// regular files, directories, and special entries. These numeric values are not account
    /// names, stable user identities, group membership, or an effective-access result. The
    /// result is a point-in-time snapshot and may change if another handle changes or replaces the
    /// entry. This method reads no contents, accepts no arbitrary URL, starts no security scope,
    /// and has the documented concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `InvalidInput` for a final symbolic link, or
    /// the mapped POSIX error for path traversal and metadata lookup failures.
    pub fn entry_owner_ids(&self, path: AppPath<'_>) -> Result<IosFileOwnerIds, FileError> {
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
        let user_id = metadata.st_uid;
        let group_id = metadata.st_gid;
        Ok(IosFileOwnerIds { user_id, group_id })
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

    /// Returns the current process user's effective read/write/execute permissions for one entry.
    ///
    /// This requests `ATTR_CMN_USERACCESS` from `getattrlistat` using the validated final name
    /// relative to an already-open no-follow parent directory descriptor. `FSOPT_NOFOLLOW` keeps a
    /// final symbolic link from redirecting the query; the result describes the final entry, not a
    /// symlink target. The mask is for the calling process's effective UID. Use
    /// [`IosEntryEffectiveAccess::allows_read`], [`IosEntryEffectiveAccess::allows_write`], and
    /// [`IosEntryEffectiveAccess::allows_execute_or_search`] to interpret `R_OK`, `W_OK`, and
    /// `X_OK`; on directories these mean read/list, add a child entry, and search. The query is a
    /// point-in-time permission snapshot, not a guarantee that a later operation will succeed.
    /// Some volume formats may not support `ATTR_CMN_USERACCESS`. A final symbolic link is not
    /// followed; any returned mask is for the no-follow entry query, not the link target. This
    /// iOS-only query reads no file contents, accepts no arbitrary URL, starts no security scope,
    /// and keeps the documented concurrent directory-rename containment limit.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for a missing final entry, `Unsupported` if the volume does not support
    /// `ATTR_CMN_USERACCESS`, `InvalidInput` for an unexpected or truncated attribute buffer, or
    /// the mapped POSIX error for path traversal and metadata query failures.
    pub fn entry_effective_access(
        &self,
        path: AppPath<'_>,
    ) -> Result<IosEntryEffectiveAccess, FileError> {
        let parts = path_parts(path.relative())?;
        let (parent, leaf) = open_parent(self.root(path.directory())?, &parts)?;
        let mut attributes = libc::attrlist {
            bitmapcount: libc::ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: libc::ATTR_CMN_USERACCESS,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };
        let mut buffer = [0_u32; 2];
        // SAFETY: the parent descriptor is open; `leaf` is one validated component; the
        // attribute list is initialized and the output buffer is writable and correctly sized.
        // `FSOPT_NOFOLLOW` prevents following the final symbolic link.
        let result = unsafe {
            libc::getattrlistat(
                parent.as_raw_fd(),
                leaf.as_ptr(),
                (&mut attributes as *mut libc::attrlist).cast(),
                buffer.as_mut_ptr().cast(),
                std::mem::size_of_val(&buffer),
                libc::FSOPT_NOFOLLOW as libc::c_ulong,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINVAL) {
                return Err(backend_error(ErrorKind::Unsupported, error.raw_os_error()));
            }
            return Err(file_error(error));
        }
        let returned_length =
            usize::try_from(buffer[0]).map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
        if returned_length < std::mem::size_of::<u32>() * 2
            || returned_length > std::mem::size_of_val(&buffer)
        {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }
        Ok(IosEntryEffectiveAccess(buffer[1]))
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
            AppDirectory::Documents => self.rename_exclusive[0].unwrap_or(false),
            AppDirectory::Caches => self.rename_exclusive[1].unwrap_or(false),
            AppDirectory::Temporary => self.rename_exclusive[2].unwrap_or(false),
            AppDirectory::ApplicationSupport => self.rename_exclusive[3].unwrap_or(false),
            _ => false,
        }
    }

    fn volume_supports_swap_rename(&self, directory: AppDirectory) -> bool {
        match directory {
            AppDirectory::Documents => self.rename_swap[0].unwrap_or(false),
            AppDirectory::Caches => self.rename_swap[1].unwrap_or(false),
            AppDirectory::Temporary => self.rename_swap[2].unwrap_or(false),
            AppDirectory::ApplicationSupport => self.rename_swap[3].unwrap_or(false),
            _ => false,
        }
    }

    fn temporary_file(&mut self, parent: &File) -> Result<(CString, File), FileError> {
        for _ in 0..32 {
            let id = self.next_temporary_id;
            self.next_temporary_id = self.next_temporary_id.wrapping_add(1);
            let name = CString::new(format!(".ios-files-{}-{id}", std::process::id()))
                .map_err(|_| FileError::InvalidPath)?;
            ensure_component_name_is_preserved(parent, &name)?;
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
        let directory = open_directory(self.root(path.directory())?, &parts)?;
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

fn volume_supports(url: &NSURL, key: &NSURLResourceKey) -> Option<bool> {
    autoreleasepool(|_| {
        let mut value = None;
        // SAFETY: Apple's NSURL resource keys used here return NSNumber boolean values.
        unsafe { url.getResourceValue_forKey_error(&mut value, key) }.ok()?;
        value?
            .downcast::<NSNumber>()
            .ok()
            .map(|number| number.boolValue())
    })
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
    let parent = open_directory(root, parents)?;
    ensure_component_name_is_preserved(&parent, leaf)?;
    Ok((parent, leaf.clone()))
}

fn open_directory(root: &File, parts: &[CString]) -> Result<File, FileError> {
    let mut directory = root.try_clone().map_err(file_error)?;
    for part in parts {
        ensure_component_name_is_preserved(&directory, part)?;
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
            return Err(file_error(io::Error::last_os_error()));
        }
        // SAFETY: `openat` returned a newly owned descriptor.
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Ok(directory)
}

fn raw_fpathconf(directory: &File, selector: libc::c_int) -> (libc::c_long, libc::c_int) {
    // SAFETY: `__error` returns this thread's errno slot.
    unsafe { *libc::__error() = 0 };
    // SAFETY: `directory` is an open descriptor and `selector` is a public pathconf selector.
    let value = unsafe { libc::fpathconf(directory.as_raw_fd(), selector) };
    let error = if value == -1 {
        // SAFETY: `__error` returns this thread's errno slot.
        unsafe { *libc::__error() }
    } else {
        0
    };
    (value, error)
}

fn query_name_truncation(directory: &File) -> Result<Option<bool>, FileError> {
    let (value, error) = raw_fpathconf(directory, libc::_PC_NO_TRUNC);
    match value {
        0 => Ok(Some(false)),
        1 => Ok(Some(true)),
        -1 if error == 0 => {
            // Legacy FSKit can encode a Boolean true as -1 with unchanged errno.
            Ok(Some(true))
        }
        -1 if error == libc::EINVAL => Ok(None),
        -1 => Err(file_error(io::Error::from_raw_os_error(error))),
        _ => Err(backend_error(ErrorKind::InvalidInput, None)),
    }
}

fn ensure_component_name_is_preserved(directory: &File, component: &CStr) -> Result<(), FileError> {
    match query_name_truncation(directory)? {
        Some(false) => Ok(()),
        Some(true) => {
            let (name_max, error) = raw_fpathconf(directory, libc::_PC_NAME_MAX);
            let name_max = match name_max {
                -1 if error == 0 => return Ok(()),
                -1 if error == libc::EINVAL => {
                    return Err(backend_error(ErrorKind::Unsupported, Some(error)));
                }
                -1 => return Err(file_error(io::Error::from_raw_os_error(error))),
                value => usize::try_from(value)
                    .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
            };
            if component.to_bytes().len() > name_max {
                Err(file_error(io::Error::from_raw_os_error(libc::ENAMETOOLONG)))
            } else {
                Ok(())
            }
        }
        None => Err(backend_error(ErrorKind::Unsupported, Some(libc::EINVAL))),
    }
}

fn scan_directory_entries(directory: &File, stop_after_first: bool) -> Result<u64, FileError> {
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
    let mut count = 0_u64;
    let read_result = loop {
        // SAFETY: `stream` remains live until `closedir` below.
        unsafe { *libc::__error() = 0 };
        // SAFETY: `stream` is a valid directory stream.
        let entry = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            // SAFETY: `__error` returns this thread's errno slot.
            let code = unsafe { *libc::__error() };
            break if code == 0 {
                Ok(count)
            } else {
                Err(file_error(io::Error::from_raw_os_error(code)))
            };
        }
        // SAFETY: `readdir` returns a live dirent whose name is NUL-terminated.
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        count = match count.checked_add(1) {
            Some(count) => count,
            None => break Err(backend_error(ErrorKind::ResourceExhausted, None)),
        };
        if stop_after_first {
            break Ok(count);
        }
    };
    // SAFETY: `stream` was returned by `fdopendir` and is closed exactly once.
    let close_result = unsafe { libc::closedir(stream) };
    let count = read_result?;
    if close_result < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    Ok(count)
}

fn parse_timespec_attribute(buffer: &[u8]) -> Result<(i64, u32), FileError> {
    let header_length = std::mem::size_of::<u32>();
    let expected_length = header_length + std::mem::size_of::<libc::timespec>();
    if buffer.len() != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    let returned_length = u32::from_ne_bytes(
        buffer[..header_length]
            .try_into()
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
    ) as usize;
    if returned_length == header_length {
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    if returned_length != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    // SAFETY: the exact returned length proves one complete `timespec` follows the u32 header.
    // XNU aligns attribute payloads to four bytes, so use an unaligned read for this C type.
    let timestamp = unsafe {
        std::ptr::read_unaligned(buffer[header_length..].as_ptr().cast::<libc::timespec>())
    };
    let seconds = timestamp.tv_sec;
    let nanoseconds = u32::try_from(timestamp.tv_nsec)
        .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?;
    if nanoseconds >= 1_000_000_000 {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    Ok((seconds, nanoseconds))
}

fn parse_volume_common_valid_attributes(buffer: &[u8]) -> Result<u32, FileError> {
    parse_volume_valid_attribute_masks(buffer).map(|(common_attributes, _)| common_attributes)
}

fn parse_volume_valid_attribute_masks(buffer: &[u8]) -> Result<(u32, u32), FileError> {
    let header_length = std::mem::size_of::<u32>();
    let expected_length = header_length + std::mem::size_of::<libc::vol_attributes_attr_t>();
    if buffer.len() != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    let returned_length = u32::from_ne_bytes(
        buffer[..header_length]
            .try_into()
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
    ) as usize;
    if returned_length == header_length {
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    if returned_length != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    // `vol_attributes_attr_t.validattr` starts the payload. The SDK and locked libc binding define
    // `attribute_set_t` with `commonattr` then `volattr`, both four-byte `attrgroup_t` fields.
    let read_mask = |offset: usize| -> Result<u32, FileError> {
        Ok(u32::from_ne_bytes(
            buffer[offset..offset + std::mem::size_of::<u32>()]
                .try_into()
                .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
        ))
    };
    let common_attributes = read_mask(header_length)?;
    let volume_attributes = read_mask(header_length + std::mem::size_of::<u32>())?;
    Ok((common_attributes, volume_attributes))
}

fn parse_volume_off_t_attribute(buffer: &[u8]) -> Result<u64, FileError> {
    let header_length = std::mem::size_of::<u32>();
    let expected_length = header_length + std::mem::size_of::<libc::off_t>();
    if buffer.len() != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    let returned_length = u32::from_ne_bytes(
        buffer[..header_length]
            .try_into()
            .map_err(|_| backend_error(ErrorKind::InvalidInput, None))?,
    ) as usize;
    if returned_length == header_length {
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    if returned_length != expected_length {
        return Err(backend_error(ErrorKind::InvalidInput, None));
    }
    // SAFETY: the exact returned length proves one complete `off_t` follows the u32 header. XNU
    // aligns every returned attribute to four bytes, so read the possibly eight-aligned type as
    // unaligned storage.
    let used_bytes =
        unsafe { std::ptr::read_unaligned(buffer[header_length..].as_ptr().cast::<libc::off_t>()) };
    u64::try_from(used_bytes).map_err(|_| backend_error(ErrorKind::InvalidInput, None))
}

fn scan_directory_entry_kind_counts(
    directory: &File,
) -> Result<IosDirectoryEntryKindCounts, FileError> {
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
    let mut counts = IosDirectoryEntryKindCounts::default();
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
        let kind = match entry_kind(directory.as_raw_fd(), name) {
            Ok(kind) => kind,
            Err(error) => break Err(file_error(error)),
        };
        let count = match kind {
            FileKind::File => &mut counts.files,
            FileKind::Directory => &mut counts.directories,
            _ => &mut counts.other,
        };
        *count = match count.checked_add(1) {
            Some(count) => count,
            None => break Err(backend_error(ErrorKind::ResourceExhausted, None)),
        };
    };
    // SAFETY: `stream` was returned by `fdopendir` and is closed exactly once.
    let close_result = unsafe { libc::closedir(stream) };
    read_result?;
    if close_result < 0 {
        return Err(file_error(io::Error::last_os_error()));
    }
    Ok(counts)
}

fn entry_mode_type(parent_fd: libc::c_int, name: &CStr) -> io::Result<libc::mode_t> {
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
    Ok(unsafe { metadata.assume_init() }.st_mode & libc::S_IFMT)
}

fn entry_kind(parent_fd: libc::c_int, name: &CStr) -> io::Result<FileKind> {
    Ok(match entry_mode_type(parent_fd, name)? {
        libc::S_IFREG => FileKind::File,
        libc::S_IFDIR => FileKind::Directory,
        _ => FileKind::Other,
    })
}

fn entry_object_kind(parent_fd: libc::c_int, name: &CStr) -> io::Result<IosEntryObjectKind> {
    let mode_type = entry_mode_type(parent_fd, name)?;
    Ok(match mode_type {
        libc::S_IFREG => IosEntryObjectKind::File,
        libc::S_IFDIR => IosEntryObjectKind::Directory,
        libc::S_IFLNK => IosEntryObjectKind::Symlink,
        libc::S_IFIFO => IosEntryObjectKind::Fifo,
        libc::S_IFSOCK => IosEntryObjectKind::Socket,
        libc::S_IFBLK => IosEntryObjectKind::BlockDevice,
        libc::S_IFCHR => IosEntryObjectKind::CharacterDevice,
        _ => IosEntryObjectKind::Unknown(u32::from(mode_type)),
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
