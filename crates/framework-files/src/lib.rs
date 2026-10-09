#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable sandbox paths and a static file backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// The semantic root for an application-owned file path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AppDirectory {
    /// User-visible application documents.
    Documents,
    /// Re-creatable application data that may be cleared by platform policy.
    Caches,
    /// Short-lived application data with no persistence promise.
    Temporary,
    /// Application support data that is not part of the user document set.
    ApplicationSupport,
}

/// A validated slash-separated path relative to one semantic application directory.
///
/// This type is not an absolute host path. It rejects empty, dot, parent, backslash, and NUL
/// segments, along with Windows drive-prefixed paths, but does not resolve symbolic links or prove
/// a backend's sandbox containment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AppPath<'a> {
    directory: AppDirectory,
    relative: &'a str,
}

impl<'a> AppPath<'a> {
    /// Creates a path under an application directory without normalizing its text.
    pub fn new(directory: AppDirectory, relative: &'a str) -> Result<Self, FileError> {
        if !valid_relative_path(relative) {
            return Err(FileError::InvalidPath);
        }
        Ok(Self {
            directory,
            relative,
        })
    }

    /// Returns the semantic application directory.
    pub const fn directory(self) -> AppDirectory {
        self.directory
    }

    /// Returns the caller-borrowed relative path text.
    pub const fn relative(self) -> &'a str {
        self.relative
    }
}

fn valid_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.ends_with('/') {
        return false;
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return false;
    }
    if path.bytes().any(|byte| byte == 0 || byte == b'\\') {
        return false;
    }
    path.split('/')
        .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// A stable file-facade error that preserves portable category and native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FileError {
    /// The path is empty, absolute, or contains a forbidden segment or character.
    InvalidPath,
    /// File bytes were not valid UTF-8 for a string read.
    InvalidUtf8,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl FileError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidPath | Self::InvalidUtf8 => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidPath | Self::InvalidUtf8 => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// The create/replace policy for a single file write.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FileWriteMode {
    /// Create the file or replace its full contents when it exists.
    CreateOrReplace,
    /// Create only when no file with this path exists.
    CreateNew,
    /// Replace only when the file already exists.
    ReplaceExisting,
}

/// The required atomicity for a single file write.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AtomicityRequirement {
    /// Accept either an atomic or a non-atomic backend write.
    AllowNonAtomic,
    /// Fail with `Unsupported` unless the backend can provide atomic replacement.
    RequireAtomic,
}

/// The atomicity that a backend reports for one file write.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum WriteAtomicity {
    /// Readers cannot observe partial new bytes; replacement changes from the whole old file to
    /// the whole new file.
    Atomic,
    /// The backend makes no atomic replacement guarantee.
    NotGuaranteed,
}

/// Options for one file write.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WriteOptions {
    mode: FileWriteMode,
    atomicity: AtomicityRequirement,
}

impl WriteOptions {
    /// Creates options with the requested create/replace mode and atomicity requirement.
    pub const fn new(mode: FileWriteMode, atomicity: AtomicityRequirement) -> Self {
        Self { mode, atomicity }
    }

    /// Creates or replaces a file while accepting the backend's atomicity level.
    pub const fn create_or_replace() -> Self {
        Self::new(
            FileWriteMode::CreateOrReplace,
            AtomicityRequirement::AllowNonAtomic,
        )
    }

    /// Returns the create/replace mode.
    pub const fn mode(self) -> FileWriteMode {
        self.mode
    }

    /// Returns the atomicity requirement.
    pub const fn atomicity(self) -> AtomicityRequirement {
        self.atomicity
    }
}

/// The backend's reported result for one file write.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WriteOutcome {
    atomicity: WriteAtomicity,
}

impl WriteOutcome {
    /// Creates a write result with the backend-reported atomicity.
    pub const fn new(atomicity: WriteAtomicity) -> Self {
        Self { atomicity }
    }

    /// Returns the backend-reported atomicity; this says nothing about crash durability.
    pub const fn atomicity(self) -> WriteAtomicity {
        self.atomicity
    }
}

/// A semantic kind for an entry returned by a directory read.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FileKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// An entry the backend does not classify as a regular file or directory.
    Other,
}

/// An owned entry name returned by a directory read.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DirectoryEntry {
    name: String,
    kind: FileKind,
}

impl DirectoryEntry {
    /// Creates an entry from one owned UTF-8 path segment.
    ///
    /// Empty, `.` or `..` names and names containing `/`, `\\`, or NUL are rejected. A backend
    /// that encounters a filesystem name outside this portable representation must return an
    /// error with kind `ErrorKind::InvalidInput` from `read_directory` rather than silently omit
    /// the entry.
    pub fn new(name: String, kind: FileKind) -> Result<Self, FileError> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
            || name.as_bytes().contains(&0)
        {
            return Err(FileError::InvalidPath);
        }
        Ok(Self { name, kind })
    }

    /// Borrows the owned entry name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the semantic entry kind.
    pub const fn kind(&self) -> FileKind {
        self.kind
    }
}

/// A compile-time-selected backend for sandbox-scoped file operations.
///
/// Every path is relative to a semantic application directory. A backend must resolve paths
/// relative to its selected directory while applying a documented symbolic-link policy. It must
/// state any containment limits caused by concurrent native namespace mutation; this trait and
/// [`AppPath`] do not prove race-free sandbox containment. Methods are synchronous and may block;
/// the facade starts no thread or executor and does not retain borrowed arguments.
pub trait FileBackend {
    /// Reports whether sandbox file access is usable in the current context.
    fn availability(&self) -> Availability;

    /// Reads a file into a caller-owned byte vector.
    fn read(&mut self, path: AppPath<'_>) -> Result<Vec<u8>, FileError>;

    /// Writes one byte slice and reports the atomicity the backend can guarantee.
    ///
    /// The backend borrows `bytes` only for this call. If `RequireAtomic` is set and atomicity is
    /// unavailable, it must return `Unsupported` before mutation. On success, a later read through
    /// the same backend observes these bytes unless another writer changes the file. A failed
    /// `AllowNonAtomic` write may have changed the file. Atomic commit does not imply crash
    /// durability, and a backend must not report `Atomic` unless readers cannot observe partial
    /// new bytes.
    fn write(
        &mut self,
        path: AppPath<'_>,
        bytes: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, FileError>;

    /// Creates exactly the requested directory; parent directories are not implied.
    fn create_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError>;

    /// Reads one directory into owned names; entry order is unspecified.
    ///
    /// If an entry name cannot be represented by `DirectoryEntry`, the backend returns an
    /// error with kind `ErrorKind::InvalidInput` rather than silently omitting that entry.
    fn read_directory(&mut self, path: AppPath<'_>) -> Result<Vec<DirectoryEntry>, FileError>;

    /// Removes one file; directory removal is a separate operation.
    fn remove_file(&mut self, path: AppPath<'_>) -> Result<(), FileError>;

    /// Removes one empty directory; recursive deletion is not implied.
    fn remove_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError>;

    /// Reports whether any final directory entry exists without following a final symlink.
    ///
    /// A present regular file, directory, symlink, or other entry returns `true`. A confirmed
    /// absent final component returns `false`; errors resolving parent components or inspecting
    /// the entry remain errors. The backend applies its documented path and symbolic-link policy
    /// while resolving the parent path and documents any limits caused by concurrent namespace
    /// mutation.
    fn exists(&mut self, path: AppPath<'_>) -> Result<bool, FileError>;
}

/// A thin file facade over a caller-owned, statically selected backend.
pub struct Files<B> {
    backend: B,
}

impl<B: FileBackend> Files<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without performing a global lookup.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Reads bytes into a caller-owned vector; no facade-level copy follows the backend result.
    pub fn read(&mut self, path: AppPath<'_>) -> Result<Vec<u8>, FileError> {
        self.backend.read(path)
    }

    /// Reads bytes and transfers the returned allocation into a UTF-8 string without a second
    /// facade-level byte copy.
    pub fn read_string(&mut self, path: AppPath<'_>) -> Result<String, FileError> {
        String::from_utf8(self.read(path)?).map_err(|_| FileError::InvalidUtf8)
    }

    /// Writes a borrowed byte slice; the backend may copy it but may not retain this borrow.
    pub fn write(
        &mut self,
        path: AppPath<'_>,
        bytes: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, FileError> {
        self.backend.write(path, bytes, options)
    }

    /// Encodes and writes UTF-8 bytes borrowed from `value` for the duration of this call.
    pub fn write_string(
        &mut self,
        path: AppPath<'_>,
        value: &str,
        options: WriteOptions,
    ) -> Result<WriteOutcome, FileError> {
        self.write(path, value.as_bytes(), options)
    }

    /// Creates exactly one directory; parent directories are not implied.
    pub fn create_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        self.backend.create_directory(path)
    }

    /// Reads one directory into owned entries; ordering is unspecified.
    ///
    /// Returns an error with kind `ErrorKind::InvalidInput` if an entry name cannot be represented
    /// by `DirectoryEntry`.
    pub fn read_directory(&mut self, path: AppPath<'_>) -> Result<Vec<DirectoryEntry>, FileError> {
        self.backend.read_directory(path)
    }

    /// Removes one file.
    pub fn remove_file(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        self.backend.remove_file(path)
    }

    /// Removes one empty directory without recursive deletion.
    pub fn remove_directory(&mut self, path: AppPath<'_>) -> Result<(), FileError> {
        self.backend.remove_directory(path)
    }

    /// Reports whether any final directory entry exists without following a final symlink.
    pub fn exists(&mut self, path: AppPath<'_>) -> Result<bool, FileError> {
        self.backend.exists(path)
    }

    /// Borrows the backend for file operations not yet modeled by this facade.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for file operations not yet modeled by this facade.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    struct Backend {
        bytes: Vec<u8>,
        atomicity: WriteAtomicity,
    }

    impl FileBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn read(&mut self, _path: AppPath<'_>) -> Result<Vec<u8>, FileError> {
            Ok(self.bytes.clone())
        }

        fn write(
            &mut self,
            _path: AppPath<'_>,
            bytes: &[u8],
            options: WriteOptions,
        ) -> Result<WriteOutcome, FileError> {
            if options.atomicity() == AtomicityRequirement::RequireAtomic
                && self.atomicity != WriteAtomicity::Atomic
            {
                return Err(FileError::Backend(Error::new(ErrorKind::Unsupported)));
            }
            self.bytes.clear();
            self.bytes.extend_from_slice(bytes);
            Ok(WriteOutcome::new(self.atomicity))
        }

        fn create_directory(&mut self, _path: AppPath<'_>) -> Result<(), FileError> {
            Ok(())
        }

        fn read_directory(&mut self, _path: AppPath<'_>) -> Result<Vec<DirectoryEntry>, FileError> {
            Ok(vec![DirectoryEntry::new(
                String::from("note.txt"),
                FileKind::File,
            )?])
        }

        fn remove_file(&mut self, _path: AppPath<'_>) -> Result<(), FileError> {
            Ok(())
        }

        fn remove_directory(&mut self, _path: AppPath<'_>) -> Result<(), FileError> {
            Ok(())
        }

        fn exists(&mut self, _path: AppPath<'_>) -> Result<bool, FileError> {
            Ok(true)
        }
    }

    #[test]
    fn paths_are_relative_and_reject_traversal_and_host_separators() {
        assert!(AppPath::new(AppDirectory::Documents, "notes/today.txt").is_ok());
        for path in [
            "",
            "/tmp/file",
            "../outside",
            "notes/..",
            "notes//file",
            "a\\b",
            "C:/outside",
            "c:relative",
        ] {
            assert_eq!(
                AppPath::new(AppDirectory::Documents, path).err(),
                Some(FileError::InvalidPath)
            );
        }
    }

    #[test]
    fn byte_and_utf8_views_have_explicit_owned_read_semantics() {
        let mut files = Files::new(Backend {
            bytes: vec![b'h', b'i'],
            atomicity: WriteAtomicity::NotGuaranteed,
        });
        let path = AppPath::new(AppDirectory::Documents, "greeting.txt").unwrap();
        assert_eq!(files.read_string(path), Ok(String::from("hi")));
        let path = AppPath::new(AppDirectory::Documents, "greeting.txt").unwrap();
        let outcome = files
            .write(path, b"updated", WriteOptions::create_or_replace())
            .unwrap();
        assert_eq!(outcome.atomicity(), WriteAtomicity::NotGuaranteed);
    }

    #[test]
    fn malformed_utf8_is_a_stable_semantic_error() {
        let mut files = Files::new(Backend {
            bytes: vec![0xff],
            atomicity: WriteAtomicity::NotGuaranteed,
        });
        let path = AppPath::new(AppDirectory::Documents, "invalid.txt").unwrap();
        assert_eq!(files.read_string(path), Err(FileError::InvalidUtf8));
        assert_eq!(FileError::InvalidUtf8.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn required_atomic_write_fails_before_mutation_when_unavailable() {
        let mut files = Files::new(Backend {
            bytes: vec![b'o', b'l', b'd'],
            atomicity: WriteAtomicity::NotGuaranteed,
        });
        let path = AppPath::new(AppDirectory::Documents, "note.txt").unwrap();
        let options = WriteOptions::new(
            FileWriteMode::CreateOrReplace,
            AtomicityRequirement::RequireAtomic,
        );

        assert_eq!(
            files.write(path, b"new", options),
            Err(FileError::Backend(Error::new(ErrorKind::Unsupported)))
        );
        assert_eq!(files.read(path), Ok(vec![b'o', b'l', b'd']));
    }

    #[test]
    fn directory_entry_names_are_single_relative_segments() {
        for name in ["", ".", "..", "a/b", "a\\b", "a\0b"] {
            assert_eq!(
                DirectoryEntry::new(String::from(name), FileKind::File).err(),
                Some(FileError::InvalidPath)
            );
        }
    }
}
