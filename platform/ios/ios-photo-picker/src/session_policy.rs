use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::copy::IosPickedImage;
use crate::{
    IosPhotoPickerEvent, MAX_MULTI_IMAGE_AGGREGATE_BYTES, MAX_MULTI_IMAGE_SELECTION,
    MIN_MULTI_IMAGE_SELECTION,
};

/// An error from PhotosUI selection or bounded provider-file copying.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IosPhotoPickerError {
    /// The caller supplied a zero-byte ceiling.
    InvalidByteLimit,
    /// The caller supplied a multi-image selection limit outside the supported range.
    InvalidSelectionLimit {
        /// The smallest supported multi-image selection limit.
        min: u8,
        /// The largest supported multi-image selection limit.
        max: u8,
    },
    /// The caller supplied a zero or over-maximum aggregate byte ceiling.
    InvalidAggregateByteLimit {
        /// The maximum supported aggregate byte ceiling.
        max: u64,
    },
    /// The selected provider did not supply an image file representation.
    ImageRepresentationUnavailable,
    /// The selected result count violated the configured selection contract.
    UnexpectedSelectionCount {
        /// The reported number of selected results.
        actual: u64,
    },
    /// The provider returned a native error.
    Provider {
        /// The native NSError domain.
        domain: String,
        /// The native NSError code.
        code: i64,
    },
    /// The provider file was empty or was not a regular file.
    InvalidProviderFile,
    /// The copied provider file exceeded its byte ceiling.
    ImageTooLarge {
        /// The applicable byte ceiling.
        limit: u64,
    },
    /// The aggregate copied provider files exceeded the caller's byte ceiling.
    AggregateImagesTooLarge {
        /// The caller-supplied aggregate byte ceiling.
        limit: u64,
    },
    /// An image file could not be copied into app-temporary storage.
    CopyFailed {
        /// A stable diagnostic from the Rust filesystem operation.
        message: String,
    },
}

type Callback = Box<dyn FnOnce(IosPhotoPickerEvent) + Send + 'static>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Pending,
    Loading,
    Terminal,
    Detached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectionDecision {
    Cancelled,
    Selected,
    UnexpectedCount(u64),
}

pub(crate) fn selection_decision(count: u64, limit: u8) -> SelectionDecision {
    match count {
        0 => SelectionDecision::Cancelled,
        actual if actual <= u64::from(limit) => SelectionDecision::Selected,
        actual => SelectionDecision::UnexpectedCount(actual),
    }
}

pub(crate) fn validate_multi_limits(
    selection_limit: u8,
    max_bytes: u64,
) -> Result<(), IosPhotoPickerError> {
    if !(MIN_MULTI_IMAGE_SELECTION..=MAX_MULTI_IMAGE_SELECTION).contains(&selection_limit) {
        return Err(IosPhotoPickerError::InvalidSelectionLimit {
            min: MIN_MULTI_IMAGE_SELECTION,
            max: MAX_MULTI_IMAGE_SELECTION,
        });
    }
    if !(1..=MAX_MULTI_IMAGE_AGGREGATE_BYTES).contains(&max_bytes) {
        return Err(IosPhotoPickerError::InvalidAggregateByteLimit {
            max: MAX_MULTI_IMAGE_AGGREGATE_BYTES,
        });
    }
    Ok(())
}

struct Inner {
    phase: Phase,
    callback: Option<Callback>,
}

/// Shared terminal-state arbitration for single-image picker and provider callbacks.
pub(crate) struct PickerCompletion {
    inner: Arc<Mutex<Inner>>,
}

impl PickerCompletion {
    pub(crate) fn new(callback: impl FnOnce(IosPhotoPickerEvent) + Send + 'static) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                phase: Phase::Pending,
                callback: Some(Box::new(callback)),
            })),
        }
    }

    pub(crate) fn clone_for_provider(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn begin_loading(&self) -> bool {
        let mut inner = lock(&self.inner);
        if inner.phase != Phase::Pending {
            return false;
        }
        inner.phase = Phase::Loading;
        true
    }

    pub(crate) fn is_loading(&self) -> bool {
        lock(&self.inner).phase == Phase::Loading
    }

    pub(crate) fn finish(&self, event: IosPhotoPickerEvent) -> bool {
        let callback = {
            let mut inner = lock(&self.inner);
            if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
                return false;
            }
            inner.phase = Phase::Terminal;
            inner.callback.take()
        };
        invoke(callback, event)
    }

    pub(crate) fn detach(&self) -> bool {
        let mut inner = lock(&self.inner);
        if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
            return false;
        }
        inner.phase = Phase::Detached;
        inner.callback.take().is_some()
    }
}

struct MultiInner {
    phase: Phase,
    callback: Option<Callback>,
    max_bytes: u64,
    reserved_bytes: u64,
    pending: usize,
    reservations: Vec<Option<u64>>,
    images: Vec<Option<IosPickedImage>>,
}

/// Ordered, aggregate-byte-bounded arbitration for multi-image provider callbacks.
pub(crate) struct MultiPickerCompletion {
    inner: Arc<Mutex<MultiInner>>,
}

impl MultiPickerCompletion {
    pub(crate) fn new(
        max_bytes: u64,
        callback: impl FnOnce(IosPhotoPickerEvent) + Send + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MultiInner {
                phase: Phase::Pending,
                callback: Some(Box::new(callback)),
                max_bytes,
                reserved_bytes: 0,
                pending: 0,
                reservations: Vec::new(),
                images: Vec::new(),
            })),
        }
    }

    pub(crate) fn clone_for_provider(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn begin_loading(&self, count: usize) -> bool {
        let mut inner = lock(&self.inner);
        if inner.phase != Phase::Pending
            || count == 0
            || count > usize::from(MAX_MULTI_IMAGE_SELECTION)
        {
            return false;
        }
        inner.phase = Phase::Loading;
        inner.pending = count;
        inner.reservations = (0..count).map(|_| None).collect();
        inner.images = (0..count).map(|_| None).collect();
        true
    }

    pub(crate) fn is_loading(&self) -> bool {
        lock(&self.inner).phase == Phase::Loading
    }

    /// Reserves provider-reported bytes before a file copy can consume temporary storage.
    pub(crate) fn reserve(&self, index: usize, byte_len: u64) -> bool {
        let result = {
            let mut inner = lock(&self.inner);
            if inner.phase != Phase::Loading
                || index >= inner.reservations.len()
                || inner.reservations[index].is_some()
            {
                return false;
            }
            let total = (byte_len != 0)
                .then(|| inner.reserved_bytes.checked_add(byte_len))
                .flatten()
                .filter(|total| *total <= inner.max_bytes);
            if let Some(total) = total {
                inner.reservations[index] = Some(byte_len);
                inner.reserved_bytes = total;
                return true;
            }
            inner.phase = Phase::Terminal;
            let callback = inner.callback.take();
            let images = core::mem::take(&mut inner.images);
            (callback, images)
        };
        drop(result.1);
        invoke(
            result.0,
            IosPhotoPickerEvent::Failed(if byte_len == 0 {
                IosPhotoPickerError::InvalidProviderFile
            } else {
                IosPhotoPickerError::AggregateImagesTooLarge {
                    limit: lock(&self.inner).max_bytes,
                }
            }),
        );
        false
    }

    /// Records a copied image at its PhotosUI selection index and emits only after all finish.
    pub(crate) fn finish_item(&self, index: usize, image: IosPickedImage) -> bool {
        let finished = {
            let mut inner = lock(&self.inner);
            if inner.phase != Phase::Loading
                || index >= inner.images.len()
                || inner.reservations[index].is_none()
                || inner.images[index].is_some()
            {
                return false;
            }
            inner.images[index] = Some(image);
            inner.pending -= 1;
            if inner.pending != 0 {
                return true;
            }
            inner.phase = Phase::Terminal;
            let callback = inner.callback.take();
            let images = core::mem::take(&mut inner.images)
                .into_iter()
                .map(|image| image.expect("all selected provider slots are complete"))
                .collect();
            Some((callback, images))
        };
        if let Some((callback, images)) = finished {
            invoke(callback, IosPhotoPickerEvent::SelectedMultiple(images));
        }
        true
    }

    pub(crate) fn fail(&self, error: IosPhotoPickerError) -> bool {
        let (callback, images) = {
            let mut inner = lock(&self.inner);
            if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
                return false;
            }
            inner.phase = Phase::Terminal;
            (inner.callback.take(), core::mem::take(&mut inner.images))
        };
        drop(images);
        invoke(callback, IosPhotoPickerEvent::Failed(error))
    }

    pub(crate) fn cancel(&self) -> bool {
        let (callback, images) = {
            let mut inner = lock(&self.inner);
            if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
                return false;
            }
            inner.phase = Phase::Terminal;
            (inner.callback.take(), core::mem::take(&mut inner.images))
        };
        drop(images);
        invoke(callback, IosPhotoPickerEvent::Cancelled)
    }

    pub(crate) fn detach(&self) -> bool {
        let (detached, images) = {
            let mut inner = lock(&self.inner);
            if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
                return false;
            }
            inner.phase = Phase::Detached;
            (
                inner.callback.take().is_some(),
                core::mem::take(&mut inner.images),
            )
        };
        drop(images);
        detached
    }
}

fn invoke(callback: Option<Callback>, event: IosPhotoPickerEvent) -> bool {
    if let Some(callback) = callback {
        let _ = catch_unwind(AssertUnwindSafe(|| callback(event)));
        true
    } else {
        false
    }
}

fn lock<T>(inner: &Mutex<T>) -> MutexGuard<'_, T> {
    inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::{
        MultiPickerCompletion, PickerCompletion, SelectionDecision, selection_decision,
        validate_multi_limits,
    };
    use crate::copy::copy_provider_file;
    use crate::{
        IosPhotoPickerError, IosPhotoPickerEvent, MAX_MULTI_IMAGE_AGGREGATE_BYTES,
        MAX_MULTI_IMAGE_SELECTION,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    fn fixture_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ios-photo-picker-policy-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn cancellation_and_late_provider_result_settle_once() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion = PickerCompletion::new(move |event| captured.lock().unwrap().push(event));
        assert!(completion.begin_loading());
        assert!(completion.finish(IosPhotoPickerEvent::Cancelled));
        assert!(!completion.finish(IosPhotoPickerEvent::Cancelled));
        assert_eq!(events.lock().unwrap().len(), 1);
    }

    #[test]
    fn session_drop_detaches_and_ignores_late_provider_completion() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion = PickerCompletion::new(move |event| captured.lock().unwrap().push(event));
        let provider = completion.clone_for_provider();
        assert!(completion.begin_loading());
        assert!(completion.detach());
        assert!(!provider.is_loading());
        assert!(!provider.finish(IosPhotoPickerEvent::Cancelled));
        assert!(events.lock().unwrap().is_empty());
    }

    #[test]
    fn callback_panic_is_contained_after_terminal_state() {
        let completion = PickerCompletion::new(|_| panic!("callback failure"));
        assert!(completion.finish(IosPhotoPickerEvent::Cancelled));
        assert!(!completion.finish(IosPhotoPickerEvent::Cancelled));
    }

    #[test]
    fn selection_count_and_multi_limits_are_bounded() {
        assert_eq!(selection_decision(1, 1), SelectionDecision::Selected);
        assert_eq!(
            selection_decision(2, 1),
            SelectionDecision::UnexpectedCount(2)
        );
        assert_eq!(selection_decision(0, 10), SelectionDecision::Cancelled);
        assert_eq!(selection_decision(1, 10), SelectionDecision::Selected);
        assert_eq!(selection_decision(10, 10), SelectionDecision::Selected);
        assert_eq!(
            selection_decision(11, 10),
            SelectionDecision::UnexpectedCount(11)
        );
        assert_eq!(
            validate_multi_limits(0, 1),
            Err(IosPhotoPickerError::InvalidSelectionLimit { min: 2, max: 10 })
        );
        assert_eq!(
            validate_multi_limits(1, 1),
            Err(IosPhotoPickerError::InvalidSelectionLimit { min: 2, max: 10 })
        );
        assert_eq!(
            validate_multi_limits(MAX_MULTI_IMAGE_SELECTION + 1, 1),
            Err(IosPhotoPickerError::InvalidSelectionLimit { min: 2, max: 10 })
        );
        assert_eq!(
            validate_multi_limits(2, 0),
            Err(IosPhotoPickerError::InvalidAggregateByteLimit {
                max: MAX_MULTI_IMAGE_AGGREGATE_BYTES
            })
        );
        assert_eq!(
            validate_multi_limits(2, MAX_MULTI_IMAGE_AGGREGATE_BYTES + 1),
            Err(IosPhotoPickerError::InvalidAggregateByteLimit {
                max: MAX_MULTI_IMAGE_AGGREGATE_BYTES
            })
        );
        assert_eq!(validate_multi_limits(2, 1), Ok(()));
        assert_eq!(
            validate_multi_limits(MAX_MULTI_IMAGE_SELECTION, MAX_MULTI_IMAGE_AGGREGATE_BYTES),
            Ok(())
        );
    }

    #[test]
    fn multi_selection_preserves_order_and_drop_cleans_all_files() {
        let fixture = fixture_dir();
        let source_a = fixture.join("a.heic");
        let source_b = fixture.join("b.png");
        let storage = fixture.join("storage");
        fs::write(&source_a, b"first").unwrap();
        fs::write(&source_b, b"second").unwrap();
        fs::create_dir(&storage).unwrap();
        let events = Arc::new(Mutex::new(None));
        let captured = events.clone();
        let completion =
            MultiPickerCompletion::new(64, move |event| *captured.lock().unwrap() = Some(event));
        assert!(completion.begin_loading(2));
        let image_a = copy_provider_file(&source_a, &storage, 5, || true).unwrap();
        let image_b = copy_provider_file(&source_b, &storage, 6, || true).unwrap();
        let path_a = image_a.path().to_owned();
        let path_b = image_b.path().to_owned();
        assert!(completion.reserve(1, 6));
        assert!(completion.finish_item(1, image_b));
        assert!(completion.reserve(0, 5));
        assert!(completion.finish_item(0, image_a));
        let event = events.lock().unwrap().take().unwrap();
        let IosPhotoPickerEvent::SelectedMultiple(images) = event else {
            panic!("expected ordered multi-image selection")
        };
        assert_eq!(fs::read(images[0].path()).unwrap(), b"first");
        assert_eq!(fs::read(images[1].path()).unwrap(), b"second");
        assert_eq!(images[0].byte_len(), 5);
        assert_eq!(images[1].byte_len(), 6);
        drop(images);
        assert!(!path_a.exists());
        assert!(!path_b.exists());
        assert_eq!(fs::read_dir(&storage).unwrap().count(), 0);
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn multi_selection_accepts_fewer_results_than_its_limit() {
        let fixture = fixture_dir();
        let source = fixture.join("one.jpg");
        let storage = fixture.join("storage");
        fs::write(&source, b"one").unwrap();
        fs::create_dir(&storage).unwrap();
        let events = Arc::new(Mutex::new(None));
        let captured = events.clone();
        let completion =
            MultiPickerCompletion::new(64, move |event| *captured.lock().unwrap() = Some(event));
        assert!(completion.begin_loading(1));
        assert!(completion.reserve(0, 3));
        let image = copy_provider_file(&source, &storage, 3, || completion.is_loading()).unwrap();
        let path = image.path().to_owned();
        assert!(completion.finish_item(0, image));
        let event = events.lock().unwrap().take().unwrap();
        let IosPhotoPickerEvent::SelectedMultiple(images) = event else {
            panic!("expected a one-item multi-image result")
        };
        assert_eq!(images.len(), 1);
        drop(images);
        assert!(!path.exists());
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn aggregate_limit_failure_cleans_partial_output_and_settles_once() {
        let fixture = fixture_dir();
        let source = fixture.join("partial.jpg");
        let storage = fixture.join("storage");
        fs::write(&source, b"12345").unwrap();
        fs::create_dir(&storage).unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion =
            MultiPickerCompletion::new(10, move |event| captured.lock().unwrap().push(event));
        assert!(completion.begin_loading(2));
        assert!(completion.reserve(0, 5));
        let image = copy_provider_file(&source, &storage, 5, || completion.is_loading()).unwrap();
        let path = image.path().to_owned();
        assert!(completion.finish_item(0, image));
        assert!(!completion.reserve(1, 6));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(&storage).unwrap().count(), 0);
        assert_eq!(events.lock().unwrap().len(), 1);
        assert!(matches!(
            events.lock().unwrap()[0],
            IosPhotoPickerEvent::Failed(IosPhotoPickerError::AggregateImagesTooLarge { limit: 10 })
        ));
        assert!(!completion.fail(IosPhotoPickerError::InvalidProviderFile));
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn cancelling_multi_session_cleans_partial_output_and_notifies_once() {
        let fixture = fixture_dir();
        let source = fixture.join("partial.jpg");
        let storage = fixture.join("storage");
        fs::write(&source, b"12345").unwrap();
        fs::create_dir(&storage).unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion =
            MultiPickerCompletion::new(64, move |event| captured.lock().unwrap().push(event));
        let provider = completion.clone_for_provider();
        assert!(completion.begin_loading(2));
        assert!(completion.reserve(0, 5));
        let image = copy_provider_file(&source, &storage, 5, || completion.is_loading()).unwrap();
        let path = image.path().to_owned();
        assert!(completion.finish_item(0, image));
        assert!(completion.cancel());
        assert!(!completion.cancel());
        assert!(!provider.is_loading());
        assert!(!provider.reserve(1, 5));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(&storage).unwrap().count(), 0);
        assert_eq!(events.lock().unwrap().len(), 1);
        assert!(matches!(
            events.lock().unwrap()[0],
            IosPhotoPickerEvent::Cancelled
        ));
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn dropped_multi_session_cleans_outputs_and_ignores_late_items() {
        let fixture = fixture_dir();
        let source = fixture.join("late.jpg");
        let storage = fixture.join("storage");
        fs::write(&source, b"late").unwrap();
        fs::create_dir(&storage).unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion =
            MultiPickerCompletion::new(64, move |event| captured.lock().unwrap().push(event));
        let provider = completion.clone_for_provider();
        assert!(completion.begin_loading(2));
        assert!(completion.reserve(0, 4));
        let image = copy_provider_file(&source, &storage, 4, || completion.is_loading()).unwrap();
        let path = image.path().to_owned();
        assert!(completion.finish_item(0, image));
        assert!(completion.detach());
        assert!(!provider.is_loading());
        assert!(!provider.reserve(1, 4));
        let late = copy_provider_file(&source, &storage, 4, || provider.is_loading());
        assert!(late.is_err());
        assert!(!path.exists());
        assert!(events.lock().unwrap().is_empty());
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn multi_callback_panic_is_contained() {
        let completion = MultiPickerCompletion::new(64, |_| panic!("callback failure"));
        assert!(completion.cancel());
        assert!(!completion.cancel());
    }
}
