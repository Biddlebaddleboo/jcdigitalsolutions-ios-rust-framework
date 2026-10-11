use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU64, Ordering};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_STAGED_DIRECTORY: AtomicU64 = AtomicU64::new(0);

/// Why a bytes-backed document export file could not be staged.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentExportFileError {
    /// The name is not a safe single file-name component.
    InvalidFileName,
    /// The app temporary-directory path cannot be represented as a Foundation string.
    TemporaryPathNotUtf8,
    /// Creating or writing the staged file failed.
    Io(io::Error),
}

impl From<io::Error> for DocumentExportFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl core::fmt::Display for DocumentExportFileError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFileName => formatter.write_str("invalid document export file name"),
            Self::TemporaryPathNotUtf8 => {
                formatter.write_str("temporary document export path is not valid UTF-8")
            }
            Self::Io(error) => write!(formatter, "could not stage document export file: {error}"),
        }
    }
}

impl std::error::Error for DocumentExportFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// A uniquely staged app-temporary file whose drop removes it unless ownership is abandoned.
pub(crate) struct StagedExportFile {
    directory: PathBuf,
    path: PathBuf,
    cleanup: bool,
}

impl StagedExportFile {
    pub(crate) fn from_bytes(
        file_name: &str,
        bytes: &[u8],
    ) -> Result<Self, DocumentExportFileError> {
        if !is_safe_file_name(file_name) {
            return Err(DocumentExportFileError::InvalidFileName);
        }

        let directory = create_staging_directory(&std::env::temp_dir())?;
        let path = directory.join(file_name);
        if path.to_str().is_none() {
            let _ = fs::remove_dir(&directory);
            return Err(DocumentExportFileError::TemporaryPathNotUtf8);
        }

        let staged = Self {
            directory,
            path,
            cleanup: true,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged.path)?;
        file.write_all(bytes)?;
        drop(file);
        Ok(staged)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn preserve(&mut self) {
        self.cleanup = false;
    }
}

impl Drop for StagedExportFile {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_dir(&self.directory);
        }
    }
}

/// Shared accepted-picker ownership; the delegate cleans at terminal, operation drop preserves.
#[derive(Clone)]
pub(crate) struct StagedExportLease(Rc<RefCell<Option<StagedExportFile>>>);

impl StagedExportLease {
    pub(crate) fn new(source: StagedExportFile) -> Self {
        Self(Rc::new(RefCell::new(Some(source))))
    }

    pub(crate) fn finish(&self) {
        drop(self.0.borrow_mut().take());
    }

    pub(crate) fn abandon(&self) {
        if let Some(mut source) = self.0.borrow_mut().take() {
            source.preserve();
        }
    }
}

fn is_safe_file_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && file_name != "."
        && file_name != ".."
        && !file_name
            .chars()
            .any(|character| matches!(character, '/' | '\\' | '\0') || character.is_control())
}

fn create_staging_directory(root: &Path) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for _ in 0..128 {
        let sequence = NEXT_STAGED_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = root.join(format!(
            ".ios-document-export-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique document export directory",
    ))
}
use std::panic::{AssertUnwindSafe, catch_unwind};

/// A terminal event reported by UIKit's document picker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DocumentExportResult {
    /// UIKit called the document-picker success delegate method.
    Exported,
    /// UIKit called the document-picker cancellation delegate method.
    Cancelled,
}

/// Why a document export request was rejected before UIKit presentation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DocumentExportError {
    /// The source is not a local file URL.
    InvalidSourceUrl,
    /// The presenter is not attached to a window or cannot present another controller.
    InvalidPresenter,
    /// This session already has an operation awaiting a UIKit terminal event.
    AlreadyExists,
    /// The current call is not on UIKit's main thread.
    #[cfg(all(target_os = "ios", feature = "document-export"))]
    MainThreadUnavailable,
}

/// A synchronous document-export start failure that returns the completion callback.
pub struct DocumentExportStartError<F> {
    /// The reason UIKit presentation was not requested.
    pub error: DocumentExportError,
    /// The callback remains owned by the caller after rejection.
    pub completion: F,
}

impl<F> core::fmt::Debug for DocumentExportStartError<F> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DocumentExportStartError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

pub(crate) fn accept_start<F>(
    is_local_file_url: bool,
    already_active: bool,
    preflight: impl FnOnce() -> Result<(), DocumentExportError>,
    completion: F,
) -> Result<F, DocumentExportStartError<F>> {
    let result = if !is_local_file_url {
        Err(DocumentExportError::InvalidSourceUrl)
    } else if already_active {
        Err(DocumentExportError::AlreadyExists)
    } else {
        preflight()
    };
    match result {
        Ok(()) => Ok(completion),
        Err(error) => Err(DocumentExportStartError { error, completion }),
    }
}

pub(crate) const fn is_local_file_url(
    is_file_url: bool,
    has_nonempty_host: bool,
    has_absolute_path: bool,
) -> bool {
    is_file_url && !has_nonempty_host && has_absolute_path
}

type Callback = Box<dyn FnOnce(DocumentExportResult) + 'static>;

#[derive(Clone)]
pub(crate) struct DocumentExportCompletion {
    callback: Rc<RefCell<Option<Callback>>>,
}

impl DocumentExportCompletion {
    pub(crate) fn new(completion: impl FnOnce(DocumentExportResult) + 'static) -> Self {
        Self {
            callback: Rc::new(RefCell::new(Some(Box::new(completion)))),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.callback.borrow().is_some()
    }

    pub(crate) fn detach(&self) -> bool {
        self.callback.borrow_mut().take().is_some()
    }

    pub(crate) fn complete(&self, result: DocumentExportResult) -> bool {
        let Some(callback) = self.callback.borrow_mut().take() else {
            return false;
        };
        let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentExportCompletion, DocumentExportError, DocumentExportFileError,
        DocumentExportResult, StagedExportFile, StagedExportLease, accept_start, is_local_file_url,
    };
    use std::cell::Cell;
    use std::fs;
    use std::rc::Rc;

    #[test]
    fn non_file_url_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let preflight_calls = Cell::new(false);
        let rejected = match accept_start(
            false,
            false,
            || {
                preflight_calls.set(true);
                Ok(())
            },
            move |_| {
                callback_calls.set(callback_calls.get() + 1);
            },
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("non-file URL was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::InvalidSourceUrl);
        assert_eq!(calls.get(), 0);
        assert!(!preflight_calls.get());
        (rejected.completion)(DocumentExportResult::Exported);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn remote_host_file_url_is_not_a_local_export_source() {
        assert!(!is_local_file_url(true, true, true));
        assert!(!is_local_file_url(true, false, false));
        assert!(is_local_file_url(true, false, true));
    }

    #[test]
    fn second_active_request_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let rejected = match accept_start(
            true,
            true,
            || panic!("active request must not preflight"),
            move |_| {
                callback_calls.set(callback_calls.get() + 1);
            },
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("second active export was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::AlreadyExists);
        assert_eq!(calls.get(), 0);
        (rejected.completion)(DocumentExportResult::Cancelled);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn invalid_presenter_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let rejected = match accept_start(
            true,
            false,
            || Err(DocumentExportError::InvalidPresenter),
            move |_| callback_calls.set(callback_calls.get() + 1),
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("invalid presenter was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::InvalidPresenter);
        assert_eq!(calls.get(), 0);
        (rejected.completion)(DocumentExportResult::Exported);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn terminal_result_is_one_shot_and_detach_suppresses_late_events() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let completion = DocumentExportCompletion::new(move |_| {
            callback_calls.set(callback_calls.get() + 1);
        });
        assert!(completion.is_active());
        assert!(completion.complete(DocumentExportResult::Exported));
        assert!(!completion.is_active());
        assert!(!completion.complete(DocumentExportResult::Cancelled));
        assert_eq!(calls.get(), 1);

        let late_calls = Rc::new(Cell::new(0));
        let callback_late_calls = Rc::clone(&late_calls);
        let detached = DocumentExportCompletion::new(move |_| {
            callback_late_calls.set(callback_late_calls.get() + 1);
        });
        assert!(detached.detach());
        assert!(!detached.complete(DocumentExportResult::Cancelled));
        assert_eq!(late_calls.get(), 0);
    }

    #[test]
    fn callback_panic_is_contained_and_callback_cannot_run_twice() {
        let completion = DocumentExportCompletion::new(|_| panic!("callback panic"));
        assert!(completion.complete(DocumentExportResult::Cancelled));
        assert!(!completion.complete(DocumentExportResult::Exported));
    }

    #[test]
    fn staged_bytes_preserve_nul_and_arbitrary_binary_content() {
        let bytes = [0x00, 0xff, 0x41, 0x00, 0xc3, 0x28];
        let source = StagedExportFile::from_bytes("logs.txt", &bytes).unwrap();
        assert_eq!(source.path().file_name().unwrap(), "logs.txt");
        assert_eq!(fs::read(source.path()).unwrap(), bytes);
    }

    #[test]
    fn unsafe_staged_file_names_are_rejected() {
        for file_name in [
            "",
            ".",
            "..",
            "../escape.txt",
            "nested/file.txt",
            "nested\\file.txt",
            "bad\0name",
            "bad\nname",
        ] {
            assert!(
                matches!(
                    StagedExportFile::from_bytes(file_name, b"data"),
                    Err(DocumentExportFileError::InvalidFileName)
                ),
                "accepted unsafe file name {file_name:?}"
            );
        }
    }

    #[test]
    fn ordinary_staged_file_drop_removes_file_and_directory() {
        let source = StagedExportFile::from_bytes("drop.txt", b"data").unwrap();
        let path = source.path().to_owned();
        let directory = path.parent().unwrap().to_owned();
        assert!(path.exists());
        drop(source);
        assert!(!path.exists());
        assert!(!directory.exists());
    }

    #[test]
    fn accepted_picker_terminal_event_cleans_staged_source_before_callback() {
        let source = StagedExportFile::from_bytes("terminal.txt", b"data").unwrap();
        let path = source.path().to_owned();
        let directory = path.parent().unwrap().to_owned();
        let lease = StagedExportLease::new(source);
        let operation_owner = lease.clone();
        let callback_path = path.clone();
        let callback_saw_file = Rc::new(Cell::new(true));
        let callback_saw_file_value = Rc::clone(&callback_saw_file);
        let completion = DocumentExportCompletion::new(move |_| {
            callback_saw_file_value.set(callback_path.exists());
        });

        lease.finish();
        assert!(completion.complete(DocumentExportResult::Exported));
        assert!(!callback_saw_file.get());
        operation_owner.abandon();
        assert!(!path.exists());
        assert!(!directory.exists());
    }

    #[test]
    fn operation_abandon_preserves_staged_source_for_temp_cleanup() {
        let source = StagedExportFile::from_bytes("abandoned.txt", b"data").unwrap();
        let path = source.path().to_owned();
        let directory = path.parent().unwrap().to_owned();
        let lease = StagedExportLease::new(source);
        let delegate_owner = lease.clone();

        lease.abandon();
        assert_eq!(fs::read(&path).unwrap(), b"data");
        delegate_owner.finish();
        assert!(path.exists());
        assert!(directory.exists());
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
