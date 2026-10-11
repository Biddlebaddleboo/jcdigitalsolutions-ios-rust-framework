#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Default-camera AVFoundation capture and preview for iOS"]

#[cfg(any(target_os = "ios", test))]
use framework_media_authorization::MediaAuthorizationStatus;
#[cfg(target_os = "ios")]
use std::marker::PhantomData;
#[cfg(target_os = "ios")]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(any(target_os = "ios", test))]
use std::path::{Path, PathBuf};
#[cfg(target_os = "ios")]
use std::rc::Rc;
#[cfg(target_os = "ios")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "ios")]
use dispatch2::{DispatchQueue, DispatchQueueAttr};
#[cfg(target_os = "ios")]
use framework_media_authorization::{CaptureMedia, MediaAuthorization};
#[cfg(target_os = "ios")]
use ios_media_authorization::IosMediaAuthorization;
#[cfg(target_os = "ios")]
use objc2::rc::{Retained, autoreleasepool};
#[cfg(target_os = "ios")]
use objc2::runtime::ProtocolObject;
#[cfg(target_os = "ios")]
use objc2::{AnyThread, DefinedClass, MainThreadMarker, define_class, msg_send};
#[cfg(target_os = "ios")]
use objc2_av_foundation::{
    AVCaptureConnection, AVCaptureDevice, AVCaptureDeviceInput, AVCaptureFileOutput,
    AVCaptureFileOutputRecordingDelegate, AVCaptureMovieFileOutput, AVCapturePhoto,
    AVCapturePhotoCaptureDelegate, AVCapturePhotoOutput, AVCapturePhotoSettings, AVCaptureSession,
    AVCaptureVideoPreviewLayer, AVMediaTypeVideo,
};
#[cfg(target_os = "ios")]
use objc2_foundation::{NSArray, NSError, NSObject, NSObjectProtocol, NSString, NSURL};

/// Failure to request or complete a processed default-camera photo
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhotoCaptureError {
    /// The operating system predates the processed-photo API floor of iOS 11
    UnsupportedOperatingSystem,
    /// The capture session has not completed startup or is no longer running
    SessionNotRunning,
    /// The session is stopping after an active movie-file recording
    SessionStopping,
    /// The configured capture session does not have its still-photo output
    PhotoOutputUnavailable,
    /// AVFoundation reported an error while capturing or processing the photo
    CaptureFailed,
    /// AVFoundation did not provide processed JPEG file bytes
    PhotoDataUnavailable,
}

#[cfg(any(target_os = "ios", test))]
type PhotoCaptureResult = Result<Vec<u8>, PhotoCaptureError>;

#[cfg(any(target_os = "ios", test))]
type PhotoCaptureCompletion = Box<dyn FnOnce(PhotoCaptureResult) + Send + 'static>;

/// Failure to start or finish a default-camera movie-file recording
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoRecordingError {
    /// The destination is empty, relative, or does not have a `.mov` extension
    InvalidDestination,
    /// The destination path cannot be represented as a Rust UTF-8 string for NSURL conversion
    InvalidPathEncoding,
    /// A file, directory, or symlink already occupies the destination
    DestinationAlreadyExists,
    /// The destination cannot be inspected before recording
    DestinationInspectionFailed,
    /// The session has not completed startup or is no longer running
    SessionNotRunning,
    /// The configured capture session does not have its movie-file output
    MovieOutputUnavailable,
    /// Another movie-file recording is active
    RecordingAlreadyActive,
    /// The session is stopping after a movie-file recording
    SessionStopping,
    /// AVFoundation reported a terminal error for the recording
    RecordingFailed,
    /// AVFoundation finalized no nonempty movie file at the requested destination
    FinalizedFileUnavailable,
}

#[cfg(any(target_os = "ios", test))]
type VideoRecordingResult = Result<PathBuf, VideoRecordingError>;

#[cfg(any(target_os = "ios", test))]
type VideoRecordingCompletion = Box<dyn FnOnce(VideoRecordingResult) + Send + 'static>;

#[cfg(any(target_os = "ios", test))]
#[derive(Default)]
struct MovieRecordingState {
    active: bool,
    stop_requested: bool,
    pending_session_stop: bool,
}

#[cfg(any(target_os = "ios", test))]
impl MovieRecordingState {
    fn begin(&mut self) -> Result<(), VideoRecordingError> {
        if self.pending_session_stop {
            return Err(VideoRecordingError::SessionStopping);
        }
        if self.active {
            return Err(VideoRecordingError::RecordingAlreadyActive);
        }
        self.active = true;
        self.stop_requested = false;
        Ok(())
    }

    fn request_recording_stop(&mut self) -> bool {
        if self.active && !self.stop_requested {
            self.stop_requested = true;
            true
        } else {
            false
        }
    }

    fn request_session_stop(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.pending_session_stop = true;
        true
    }

    fn settle_terminal(&mut self) -> bool {
        self.active = false;
        self.stop_requested = false;
        let stop_session = self.pending_session_stop;
        self.pending_session_stop = false;
        stop_session
    }
}

#[cfg(any(target_os = "ios", test))]
fn validate_movie_destination(path: &Path) -> Result<PathBuf, VideoRecordingError> {
    if path.as_os_str().is_empty() {
        return Err(VideoRecordingError::InvalidDestination);
    }
    let Some(path_text) = path.to_str() else {
        return Err(VideoRecordingError::InvalidPathEncoding);
    };
    if !path.is_absolute()
        || !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mov"))
        || path_text.trim().is_empty()
    {
        return Err(VideoRecordingError::InvalidDestination);
    }
    match std::fs::symlink_metadata(path) {
        Ok(_) => Err(VideoRecordingError::DestinationAlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(_) => Err(VideoRecordingError::DestinationInspectionFailed),
    }
}

#[cfg(any(target_os = "ios", test))]
fn settle_movie_result(path: &Path, terminal_error: bool) -> VideoRecordingResult {
    if terminal_error {
        return Err(VideoRecordingError::RecordingFailed);
    }
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.len() > 0 => Ok(path.to_path_buf()),
        _ => Err(VideoRecordingError::FinalizedFileUnavailable),
    }
}

#[cfg(any(target_os = "ios", test))]
fn take_video_completion(
    terminal_claimed: &mut bool,
    completion: &mut Option<VideoRecordingCompletion>,
) -> Option<VideoRecordingCompletion> {
    if *terminal_claimed {
        return None;
    }
    *terminal_claimed = true;
    completion.take()
}

#[cfg(any(target_os = "ios", test))]
fn photo_request_error(
    is_running: bool,
    output_available: bool,
    pending_session_stop: bool,
) -> Option<PhotoCaptureError> {
    if pending_session_stop {
        Some(PhotoCaptureError::SessionStopping)
    } else if !output_available {
        Some(PhotoCaptureError::PhotoOutputUnavailable)
    } else if !is_running {
        Some(PhotoCaptureError::SessionNotRunning)
    } else {
        None
    }
}

#[cfg(target_os = "ios")]
struct PhotoCaptureState {
    processed: Option<PhotoCaptureResult>,
    completion: Option<PhotoCaptureCompletion>,
    self_retention: Option<Retained<PhotoCaptureDelegateObject>>,
}

#[cfg(target_os = "ios")]
struct PhotoCaptureDelegateIvars {
    state: Mutex<PhotoCaptureState>,
}

#[cfg(target_os = "ios")]
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = AnyThread]
    #[ivars = PhotoCaptureDelegateIvars]
    struct PhotoCaptureDelegateObject;

    // SAFETY: NSObjectProtocol adds no methods; all mutable state uses one mutex.
    unsafe impl NSObjectProtocol for PhotoCaptureDelegateObject {}

    // SAFETY: AVFoundation may invoke this delegate on an arbitrary callback queue. Every
    // callback retains the receiver, contains Rust panics, and copies native bytes before return.
    #[allow(non_snake_case)]
    unsafe impl AVCapturePhotoCaptureDelegate for PhotoCaptureDelegateObject {
        #[unsafe(method(captureOutput:didFinishProcessingPhoto:error:))]
        unsafe fn captureOutput_didFinishProcessingPhoto_error(
            &self,
            _output: &AVCapturePhotoOutput,
            photo: &AVCapturePhoto,
            error: Option<&NSError>,
        ) {
            let _receiver = unsafe { retain_photo_callback_receiver(self) };
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    let processed = if error.is_some() {
                        Err(PhotoCaptureError::CaptureFailed)
                    } else {
                        copy_processed_photo(photo)
                    };
                    self.record_processed(processed);
                });
            }));
            if result.is_err() {
                self.record_processed(Err(PhotoCaptureError::CaptureFailed));
            }
        }

        #[unsafe(method(captureOutput:didFinishCaptureForResolvedSettings:error:))]
        unsafe fn captureOutput_didFinishCaptureForResolvedSettings_error(
            &self,
            _output: &AVCapturePhotoOutput,
            _resolved_settings: &objc2_av_foundation::AVCaptureResolvedPhotoSettings,
            error: Option<&NSError>,
        ) {
            let _receiver = unsafe { retain_photo_callback_receiver(self) };
            let result = catch_unwind(AssertUnwindSafe(|| self.finish(error.is_some())));
            if result.is_err() {
                self.finish_after_panic();
            }
            self.release_self_retention();
        }
    }
);

#[cfg(target_os = "ios")]
impl PhotoCaptureDelegateObject {
    fn new(completion: PhotoCaptureCompletion) -> Retained<Self> {
        let ivars = PhotoCaptureDelegateIvars {
            state: Mutex::new(PhotoCaptureState {
                processed: None,
                completion: Some(completion),
                self_retention: None,
            }),
        };
        let allocated = Self::alloc().set_ivars(ivars);
        // SAFETY: The delegate ivars are initialized before NSObject's initializer runs.
        let delegate: Retained<Self> = unsafe { msg_send![super(allocated), init] };
        // AVFoundation does not own this Rust-created delegate. Its callback contract ends only
        // at the terminal method, so a per-request self-retain keeps the delegate alive until then.
        let retained = unsafe {
            Retained::retain(&*delegate as *const Self as *mut Self)
                .expect("live photo delegate must retain itself")
        };
        delegate
            .ivars()
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .self_retention = Some(retained);
        delegate
    }

    fn record_processed(&self, processed: PhotoCaptureResult) {
        let mut state = self
            .ivars()
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.completion.is_some() && state.processed.is_none() {
            state.processed = Some(processed);
        }
    }

    fn finish(&self, terminal_error: bool) {
        let completion = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let processed = state.processed.take();
            take_photo_completion(&mut state.completion, processed, terminal_error)
        };
        if let Some((completion, result)) = completion {
            let _ = catch_unwind(AssertUnwindSafe(|| completion(result)));
        }
    }

    fn finish_after_panic(&self) {
        let completion = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.processed = Some(Err(PhotoCaptureError::CaptureFailed));
            state.completion.take()
        };
        if let Some(completion) = completion {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                completion(Err(PhotoCaptureError::CaptureFailed))
            }));
        }
    }

    fn release_self_retention(&self) {
        let retained = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.self_retention.take()
        };
        drop(retained);
    }
}

#[cfg(target_os = "ios")]
unsafe fn retain_photo_callback_receiver(
    receiver: &PhotoCaptureDelegateObject,
) -> Option<Retained<PhotoCaptureDelegateObject>> {
    // SAFETY: objc2 supplies a live callback receiver; this retain keeps its ivars valid through
    // completion settlement and release of the per-request self-retain.
    unsafe {
        Retained::retain(
            receiver as *const PhotoCaptureDelegateObject as *mut PhotoCaptureDelegateObject,
        )
    }
}

#[cfg(target_os = "ios")]
fn copy_processed_photo(photo: &AVCapturePhoto) -> PhotoCaptureResult {
    // SAFETY: The caller's capture request is gated to iOS 11+, the documented availability floor
    // of fileDataRepresentation.
    let data =
        unsafe { photo.fileDataRepresentation() }.ok_or(PhotoCaptureError::PhotoDataUnavailable)?;
    let bytes = data.to_vec();
    if bytes.is_empty() {
        return Err(PhotoCaptureError::PhotoDataUnavailable);
    }
    Ok(bytes)
}

#[cfg(target_os = "ios")]
struct MovieRecordingDelegateState {
    graph: Option<Arc<Mutex<CaptureGraph>>>,
    queue: Option<dispatch2::DispatchRetained<DispatchQueue>>,
    destination: PathBuf,
    completion: Option<VideoRecordingCompletion>,
    self_retention: Option<Retained<MovieRecordingDelegateObject>>,
    terminal_claimed: bool,
}

#[cfg(target_os = "ios")]
struct MovieRecordingTerminalPayload {
    graph: Option<Arc<Mutex<CaptureGraph>>>,
    queue: Option<dispatch2::DispatchRetained<DispatchQueue>>,
    destination: PathBuf,
    completion: Option<VideoRecordingCompletion>,
}

#[cfg(target_os = "ios")]
struct MovieRecordingDelegateIvars {
    state: Mutex<MovieRecordingDelegateState>,
}

#[cfg(target_os = "ios")]
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = AnyThread]
    #[ivars = MovieRecordingDelegateIvars]
    struct MovieRecordingDelegateObject;

    // SAFETY: NSObjectProtocol adds no methods; all mutable delegate state is mutex-protected.
    unsafe impl NSObjectProtocol for MovieRecordingDelegateObject {}

    // SAFETY: AVFoundation may invoke the required terminal callback on any thread. It retains
    // the receiver through callback settlement, contains Rust panics, and never exposes samples.
    unsafe impl AVCaptureFileOutputRecordingDelegate for MovieRecordingDelegateObject {
        #[unsafe(method(captureOutput:didFinishRecordingToOutputFileAtURL:fromConnections:error:))]
        #[allow(non_snake_case)]
        unsafe fn captureOutput_didFinishRecordingToOutputFileAtURL_fromConnections_error(
            &self,
            _output: &AVCaptureFileOutput,
            _output_file_url: &NSURL,
            _connections: &NSArray<AVCaptureConnection>,
            error: Option<&NSError>,
        ) {
            let _receiver = unsafe { retain_movie_callback_receiver(self) };
            let terminal_error = error.is_some();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| self.finish(terminal_error));
            }));
            if outcome.is_err() {
                self.finish_after_panic();
            }
            let _ = catch_unwind(AssertUnwindSafe(|| self.release_self_retention()));
        }
    }
);

#[cfg(target_os = "ios")]
impl MovieRecordingDelegateObject {
    fn new(
        graph: Arc<Mutex<CaptureGraph>>,
        queue: dispatch2::DispatchRetained<DispatchQueue>,
        destination: PathBuf,
        completion: VideoRecordingCompletion,
    ) -> Retained<Self> {
        let ivars = MovieRecordingDelegateIvars {
            state: Mutex::new(MovieRecordingDelegateState {
                graph: Some(graph),
                queue: Some(queue),
                destination,
                completion: Some(completion),
                self_retention: None,
                terminal_claimed: false,
            }),
        };
        let allocated = Self::alloc().set_ivars(ivars);
        // SAFETY: Delegate ivars are initialized before NSObject's initializer runs.
        let delegate: Retained<Self> = unsafe { msg_send![super(allocated), init] };
        // AVFoundation's delegate ownership is not relied on; retain through didFinish explicitly.
        let retained = unsafe {
            Retained::retain(&*delegate as *const Self as *mut Self)
                .expect("live movie delegate must retain itself")
        };
        delegate
            .ivars()
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .self_retention = Some(retained);
        delegate
    }

    fn finish(&self, terminal_error: bool) {
        let payload = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let completion = {
                let MovieRecordingDelegateState {
                    terminal_claimed,
                    completion,
                    ..
                } = &mut *state;
                take_video_completion(terminal_claimed, completion)
            };
            let Some(completion) = completion else {
                return;
            };
            MovieRecordingTerminalPayload {
                graph: state.graph.take(),
                queue: state.queue.take(),
                destination: state.destination.clone(),
                completion: Some(completion),
            }
        };
        Self::enqueue_settlement(payload, terminal_error);
    }

    fn finish_after_panic(&self) {
        let payload = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let completion = {
                let MovieRecordingDelegateState {
                    terminal_claimed,
                    completion,
                    ..
                } = &mut *state;
                take_video_completion(terminal_claimed, completion)
            };
            let Some(completion) = completion else {
                return;
            };
            MovieRecordingTerminalPayload {
                graph: state.graph.take(),
                queue: state.queue.take(),
                destination: state.destination.clone(),
                completion: Some(completion),
            }
        };
        Self::enqueue_settlement(payload, true);
    }

    fn enqueue_settlement(payload: MovieRecordingTerminalPayload, terminal_error: bool) {
        let MovieRecordingTerminalPayload {
            graph: Some(graph),
            queue: Some(queue),
            destination,
            completion: Some(completion),
        } = payload
        else {
            return;
        };
        queue.exec_async(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                settle_movie_result(&destination, terminal_error)
            }))
            .unwrap_or(Err(VideoRecordingError::RecordingFailed));
            let stop_session = {
                let mut graph_state = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                graph_state.movie_recording.settle_terminal()
            };
            invoke_video_completion(completion, result);
            if stop_session {
                let graph_state = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // SAFETY: File finalization and its user completion have reached terminal state;
                // this settlement runs on the graph's private serial queue.
                unsafe { graph_state.session.stopRunning() };
            }
            drop(graph);
        });
    }

    fn release_self_retention(&self) {
        let retained = {
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.self_retention.take()
        };
        drop(retained);
    }
}

#[cfg(target_os = "ios")]
unsafe fn retain_movie_callback_receiver(
    receiver: &MovieRecordingDelegateObject,
) -> Option<Retained<MovieRecordingDelegateObject>> {
    // SAFETY: Keep the callback receiver alive while its self-retain is released at terminal.
    unsafe {
        Retained::retain(
            receiver as *const MovieRecordingDelegateObject as *mut MovieRecordingDelegateObject,
        )
    }
}

#[cfg(any(target_os = "ios", test))]
fn settle_photo_result(
    processed: Option<PhotoCaptureResult>,
    terminal_error: bool,
) -> PhotoCaptureResult {
    if terminal_error {
        Err(PhotoCaptureError::CaptureFailed)
    } else {
        processed.unwrap_or(Err(PhotoCaptureError::PhotoDataUnavailable))
    }
}

#[cfg(any(target_os = "ios", test))]
fn take_photo_completion(
    completion: &mut Option<PhotoCaptureCompletion>,
    processed: Option<PhotoCaptureResult>,
    terminal_error: bool,
) -> Option<(PhotoCaptureCompletion, PhotoCaptureResult)> {
    completion
        .take()
        .map(|completion| (completion, settle_photo_result(processed, terminal_error)))
}

#[cfg(any(target_os = "ios", test))]
const fn may_create_device_input(status: MediaAuthorizationStatus) -> bool {
    matches!(status, MediaAuthorizationStatus::Authorized)
}

/// Failure to create or configure a default-camera capture session
#[cfg(target_os = "ios")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraCaptureError {
    /// The app has not been granted camera access
    AuthorizationNotGranted,
    /// The public AVFoundation video media-type constant is unavailable
    VideoMediaTypeUnavailable,
    /// The system has no default video capture device
    NoDefaultVideoDevice,
    /// AVFoundation could not create an input for the default video device
    DeviceInputCreationFailed,
    /// AVFoundation rejected the default video input for the new session
    SessionRejectedInput,
    /// AVFoundation rejected the still-photo output for the new session
    SessionRejectedPhotoOutput,
    /// AVFoundation rejected the movie-file output for the new session
    SessionRejectedMovieOutput,
    /// The operating system predates the required AVFoundation authorization API
    UnsupportedOperatingSystem,
}

#[cfg(target_os = "ios")]
struct CaptureGraph {
    session: Retained<AVCaptureSession>,
    photo_output: Option<Retained<AVCapturePhotoOutput>>,
    movie_output: Option<Retained<AVCaptureMovieFileOutput>>,
    movie_recording: MovieRecordingState,
}

// SAFETY: The graph is private and every session, photo-output, movie-output, and recording-state
// operation is protected by its mutex. Setup and all later capture/lifecycle calls use one private
// serial DispatchQueue. The only main-thread session access is the synchronous preview-layer
// initializer after queue setup and before publication. Published graph destruction occurs on the
// serial queue after stop or the movie terminal callback. The preview layer is created/exposed only
// on main; the recording delegate retains the graph without the graph retaining the delegate.
#[cfg(target_os = "ios")]
unsafe impl Send for CaptureGraph {}

/// An iOS default-camera session with a live preview-layer escape hatch
///
/// Creation requires camera authorization to be granted already and the host app to provide
/// `NSCameraUsageDescription`. This type does not request permission, record audio, or expose media
/// samples. Create and use it on the main thread. Creation synchronously opens and
/// configures the default video input before returning, so it may block while AVFoundation prepares
/// the device. Its preview layer can be added to a UIKit view's layer hierarchy. Start and stop
/// requests run asynchronously on a private serial queue; returning from either method does not
/// prove that AVFoundation started or stopped the session. This wrapper does not observe
/// AVFoundation's native runtime-error notifications. Video recording writes one QuickTime movie
/// to a caller-owned `.mov` path and invokes its completion after AVFoundation finalizes the file.
/// Stopping the session during recording defers `stopRunning` until that terminal callback. The
/// host owns audio-session configuration and may import completed files into Photos separately.
///
/// Compilation and linking validate the native API boundary only; they do not prove camera access
/// or preview behavior on a simulator or physical device.
#[cfg(target_os = "ios")]
pub struct IosCameraCaptureSession {
    graph: Arc<Mutex<CaptureGraph>>,
    queue: dispatch2::DispatchRetained<DispatchQueue>,
    preview_layer: Retained<AVCaptureVideoPreviewLayer>,
    _main_thread_only: PhantomData<Rc<()>>,
}

#[cfg(target_os = "ios")]
impl IosCameraCaptureSession {
    /// Create a session for the already-authorized default video capture device
    ///
    /// This call does not show a permission prompt. Request camera authorization through the
    /// framework's media-authorization capability before calling this method.
    pub fn new(_main_thread: MainThreadMarker) -> Result<Self, CameraCaptureError> {
        if !objc2::available!(ios = 7.0, ..) {
            return Err(CameraCaptureError::UnsupportedOperatingSystem);
        }

        let authorization = IosMediaAuthorization::authorization_status(CaptureMedia::Camera);
        if !may_create_device_input(authorization) {
            return Err(CameraCaptureError::AuthorizationNotGranted);
        }

        // SAFETY: The deployment availability check above covers the iOS 7.0 authorization API
        // floor, which is newer than AVCaptureSession and its initialization API.
        let session = unsafe { AVCaptureSession::new() };
        let queue = DispatchQueue::new(
            "com.jcdigitalsolutions.ios-camera-capture-session",
            DispatchQueueAttr::SERIAL,
        );
        let graph = Arc::new(Mutex::new(CaptureGraph {
            session,
            photo_output: None,
            movie_output: None,
            movie_recording: MovieRecordingState::default(),
        }));

        let mut configuration = Ok(());
        let graph_for_configuration = Arc::clone(&graph);
        queue.exec_sync(|| {
            let mut graph = graph_for_configuration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            configuration = configure_session(&mut graph);
        });
        if let Err(error) = configuration {
            let graph_to_drop = Arc::clone(&graph);
            drop(graph);
            queue.exec_sync(move || drop(graph_to_drop));
            return Err(error);
        }

        let preview_layer = {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // SAFETY: The session has a configured video input, and the initializer is called on
            // the main thread as required for the UIKit layer escape hatch.
            unsafe { AVCaptureVideoPreviewLayer::layerWithSession(&graph.session) }
        };

        Ok(Self {
            graph,
            queue,
            preview_layer,
            _main_thread_only: PhantomData,
        })
    }

    /// Retain the preview layer for insertion into a UIKit layer hierarchy
    pub fn preview_layer(
        &self,
        _main_thread: MainThreadMarker,
    ) -> Retained<AVCaptureVideoPreviewLayer> {
        self.preview_layer.clone()
    }

    /// Enqueue a nonblocking start request on the private serial queue
    ///
    /// The request is serialized with configuration, stop requests, and drop-time shutdown. The
    /// method returns before AVFoundation completes startup.
    pub fn start(&self) {
        let graph = Arc::clone(&self.graph);
        self.queue.exec_async(move || {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if graph.movie_recording.pending_session_stop {
                return;
            }
            // SAFETY: This call runs on the private serial queue after configuration has completed;
            // no configuration call runs concurrently with start or stop.
            unsafe { graph.session.startRunning() };
        });
    }

    /// Enqueue a nonblocking stop request on the private serial queue
    ///
    /// The method returns before AVFoundation completes shutdown.
    pub fn stop(&self) {
        enqueue_stop(&self.queue, &self.graph);
    }

    /// Record video from the default camera to a new QuickTime movie file
    ///
    /// The destination must be an absolute, UTF-8 `.mov` path in caller-owned writable storage
    /// and must not already exist. Preflight existence checks are racy; AVFoundation remains
    /// authoritative if another actor creates the file before recording starts. A synchronous
    /// error means the request was not queued and `completion` is not called. After `Ok(())`, the
    /// completion runs once on the private serial queue only after AVFoundation's required
    /// terminal recording callback; it receives the finalized destination or a normalized error.
    /// A terminal native error is failure even if AVFoundation marks the file playable. At most
    /// one recording may be active. This operation requires an already authorized and running
    /// camera session, does not request permission or configure audio, and does not write to Photos.
    /// `stop_video_recording` requests finalization without deleting a partial file; session
    /// `stop`/drop defers `stopRunning` until finalization completes. UIKit work from completion
    /// must hop to main. Compilation and linking do not prove camera capture, encoding, or playback.
    pub fn start_video_recording<F>(
        &self,
        destination: impl AsRef<Path>,
        completion: F,
    ) -> Result<(), VideoRecordingError>
    where
        F: FnOnce(Result<PathBuf, VideoRecordingError>) + Send + 'static,
    {
        let destination = validate_movie_destination(destination.as_ref())?;
        let graph = Arc::clone(&self.graph);
        let callback_queue = self.queue.clone();
        self.queue.exec_async(move || {
            let mut completion = Some(Box::new(completion) as VideoRecordingCompletion);
            let mut delegate_created = false;
            let result = catch_unwind(AssertUnwindSafe(|| {
                let setup = {
                    let mut graph_state = graph
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if graph_state.movie_recording.pending_session_stop {
                        Err(VideoRecordingError::SessionStopping)
                    } else if graph_state.movie_recording.active {
                        Err(VideoRecordingError::RecordingAlreadyActive)
                    } else if !unsafe { graph_state.session.isRunning() } {
                        Err(VideoRecordingError::SessionNotRunning)
                    } else {
                        match graph_state.movie_output.clone() {
                            Some(output) => graph_state.movie_recording.begin().map(|()| output),
                            None => Err(VideoRecordingError::MovieOutputUnavailable),
                        }
                    }
                };
                let output = match setup {
                    Ok(output) => output,
                    Err(error) => {
                        invoke_video_completion(
                            completion.take().expect("completion is available"),
                            Err(error),
                        );
                        return;
                    }
                };
                let path_text = destination
                    .to_str()
                    .expect("destination validation guarantees UTF-8");
                let path_string = NSString::from_str(path_text);
                let file_url = NSURL::fileURLWithPath(&path_string);
                let delegate = MovieRecordingDelegateObject::new(
                    Arc::clone(&graph),
                    callback_queue,
                    destination,
                    completion.take().expect("completion is available"),
                );
                delegate_created = true;
                let protocol_delegate = ProtocolObject::from_ref(&*delegate);
                // SAFETY: The URL is a file URL made from the validated absolute UTF-8 path, the
                // session is running, and the delegate self-retains through terminal finalization.
                unsafe {
                    output.startRecordingToOutputFileURL_recordingDelegate(
                        &file_url,
                        protocol_delegate,
                    )
                };
            }));
            if result.is_err() {
                if !delegate_created {
                    let mut graph_state = graph
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let stop_session = graph_state.movie_recording.settle_terminal();
                    if stop_session {
                        // SAFETY: The catch is on the private queue and terminal state now owns
                        // session shutdown if Drop/stop was queued during setup.
                        unsafe { graph_state.session.stopRunning() };
                    }
                }
                if let Some(completion) = completion.take() {
                    invoke_video_completion(completion, Err(VideoRecordingError::RecordingFailed));
                }
            }
        });
        Ok(())
    }

    /// Request movie-file finalization without waiting for the terminal callback
    ///
    /// This is idempotent while idle or already stopping. The completion supplied to
    /// [`Self::start_video_recording`] runs only after AVFoundation has finalized the file.
    pub fn stop_video_recording(&self) {
        enqueue_movie_stop(&self.queue, &self.graph);
    }

    /// Capture one processed JPEG from the default camera without blocking the caller
    ///
    /// Call [`Self::start`] first and wait until the session is running before requesting a photo.
    /// A synchronous error means the completion does not run. After this returns `Ok(())`, the
    /// completion runs once on AVFoundation's callback queue after capture processing ends; if the
    /// session is not running or lacks its configured photo output, it runs on the private session
    /// queue instead. Neither queue is guaranteed to be main, so UIKit work must hop to main. The
    /// operation cannot be cancelled. It returns owned JPEG file bytes and does not write to Photos.
    /// Capture requires iOS 11 or later; session and preview setup keep their lower availability
    /// floor. Keep the camera session alive until the completion runs to avoid interrupting the
    /// native capture lifecycle. Compilation and linking do not prove camera or device runtime.
    pub fn capture_photo<F>(&self, completion: F) -> Result<(), PhotoCaptureError>
    where
        F: FnOnce(Result<Vec<u8>, PhotoCaptureError>) + Send + 'static,
    {
        if !objc2::available!(ios = 11.0, ..) {
            return Err(PhotoCaptureError::UnsupportedOperatingSystem);
        }

        let graph = Arc::clone(&self.graph);
        self.queue.exec_async(move || {
            let (output, is_running, pending_session_stop) = {
                let graph = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (
                    graph.photo_output.clone(),
                    unsafe { graph.session.isRunning() },
                    graph.movie_recording.pending_session_stop,
                )
            };
            if let Some(error) =
                photo_request_error(is_running, output.is_some(), pending_session_stop)
            {
                invoke_photo_completion(Box::new(completion), Err(error));
                return;
            }
            let Some(output) = output else {
                invoke_photo_completion(
                    Box::new(completion),
                    Err(PhotoCaptureError::PhotoOutputUnavailable),
                );
                return;
            };

            let delegate = PhotoCaptureDelegateObject::new(Box::new(completion));
            let settings = unsafe { AVCapturePhotoSettings::photoSettings() };
            let protocol_delegate = ProtocolObject::from_ref(&*delegate);
            // SAFETY: The session is running, the output belongs to its configured graph, settings
            // are fresh for this request, and the delegate is strongly self-retained to terminal.
            unsafe { output.capturePhotoWithSettings_delegate(&settings, protocol_delegate) };
        });
        Ok(())
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosCameraCaptureSession {
    fn drop(&mut self) {
        enqueue_stop(&self.queue, &self.graph);
    }
}

#[cfg(target_os = "ios")]
fn configure_session(graph: &mut CaptureGraph) -> Result<(), CameraCaptureError> {
    // SAFETY: This is AVFoundation's public video media-type constant. The constructor checks the
    // iOS 7.0 API floor before this function runs.
    let media_type =
        unsafe { AVMediaTypeVideo }.ok_or(CameraCaptureError::VideoMediaTypeUnavailable)?;
    // SAFETY: The media type is exactly AVMediaTypeVideo, as required by this public API.
    let device = unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) }
        .ok_or(CameraCaptureError::NoDefaultVideoDevice)?;
    // SAFETY: Authorization was checked before this queue was entered; AVFoundation returns an
    // error for a device that cannot be opened, including permission or device-state changes.
    let input = unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }
        .map_err(|_| CameraCaptureError::DeviceInputCreationFailed)?;
    // SAFETY: Both objects are valid AVFoundation instances, and addInput is called only after
    // canAddInput has established this input is accepted by the unstarted session.
    if !unsafe { graph.session.canAddInput(&input) } {
        return Err(CameraCaptureError::SessionRejectedInput);
    }
    // SAFETY: The immediately preceding canAddInput call accepted this exact input.
    unsafe { graph.session.addInput(&input) };

    if objc2::available!(ios = 10.0, ..) {
        // SAFETY: AVCapturePhotoOutput is a public AVFoundation class available from iOS 10.
        let output = unsafe { AVCapturePhotoOutput::new() };
        // SAFETY: The session is unstarted and the new output is a valid AVFoundation object.
        if !unsafe { graph.session.canAddOutput(&output) } {
            return Err(CameraCaptureError::SessionRejectedPhotoOutput);
        }
        // SAFETY: The immediately preceding canAddOutput call accepted this exact output.
        unsafe { graph.session.addOutput(&output) };
        graph.photo_output = Some(output);
    }

    // SAFETY: AVCaptureMovieFileOutput is a public AVFoundation class available from iOS 4.0;
    // this package's device and Simulator floors are iOS 10 and 14 respectively.
    let movie_output = unsafe { AVCaptureMovieFileOutput::new() };
    // SAFETY: The session is still unstarted and canAddOutput validates this exact output.
    if !unsafe { graph.session.canAddOutput(&movie_output) } {
        return Err(CameraCaptureError::SessionRejectedMovieOutput);
    }
    // SAFETY: The immediately preceding canAddOutput call accepted this exact output.
    unsafe { graph.session.addOutput(&movie_output) };
    graph.movie_output = Some(movie_output);
    Ok(())
}

#[cfg(target_os = "ios")]
fn enqueue_stop(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
) {
    let graph = Arc::clone(graph);
    queue.exec_async(move || {
        let mut graph = graph
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let defer_session_stop = graph.movie_recording.request_session_stop();
        if defer_session_stop {
            let stop_recording = graph.movie_recording.request_recording_stop();
            if stop_recording {
                if let Some(output) = graph.movie_output.as_ref() {
                    // SAFETY: The recording is active and stop runs on the private serial queue;
                    // session shutdown is deferred to the required terminal delegate callback.
                    unsafe { output.stopRecording() };
                }
            }
        } else {
            // SAFETY: This call runs on the same private serial queue as setup and start. Queue
            // order prevents shutdown from overlapping configuration or a prior start request.
            unsafe { graph.session.stopRunning() };
        }
    });
}

#[cfg(target_os = "ios")]
fn enqueue_movie_stop(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
) {
    let graph = Arc::clone(graph);
    queue.exec_async(move || {
        let mut graph = graph
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if graph.movie_recording.request_recording_stop() {
            if let Some(output) = graph.movie_output.as_ref() {
                // SAFETY: This call is serialized with capture/session operations. The active
                // state remains set until the delegate's terminal callback finalizes the file.
                unsafe { output.stopRecording() };
            }
        }
    });
}

#[cfg(target_os = "ios")]
fn invoke_photo_completion(completion: PhotoCaptureCompletion, result: PhotoCaptureResult) {
    let _ = catch_unwind(AssertUnwindSafe(|| completion(result)));
}

#[cfg(target_os = "ios")]
fn invoke_video_completion(completion: VideoRecordingCompletion, result: VideoRecordingResult) {
    let _ = catch_unwind(AssertUnwindSafe(|| completion(result)));
}

#[cfg(test)]
mod tests {
    use super::{
        MediaAuthorizationStatus, MovieRecordingState, PhotoCaptureError, PhotoCaptureResult,
        VideoRecordingError, VideoRecordingResult, may_create_device_input, photo_request_error,
        settle_movie_result, settle_photo_result, take_photo_completion, take_video_completion,
        validate_movie_destination,
    };
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn unused_movie_path(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "ios-camera-{label}-{}-{}.mov",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn device_input_requires_granted_camera_authorization() {
        assert!(may_create_device_input(
            MediaAuthorizationStatus::Authorized
        ));
        assert!(!may_create_device_input(
            MediaAuthorizationStatus::NotDetermined
        ));
        assert!(!may_create_device_input(
            MediaAuthorizationStatus::Restricted
        ));
        assert!(!may_create_device_input(MediaAuthorizationStatus::Denied));
        assert!(!may_create_device_input(MediaAuthorizationStatus::Unknown));
    }

    #[test]
    fn photo_capture_settles_success_with_owned_bytes() {
        let result = settle_photo_result(Some(Ok(vec![0xff, 0xd8, 0xff])), false);
        assert_eq!(result, Ok(vec![0xff, 0xd8, 0xff]));
    }

    #[test]
    fn photo_processing_failure_is_normalized() {
        let result = settle_photo_result(Some(Err(PhotoCaptureError::CaptureFailed)), false);
        assert_eq!(result, Err(PhotoCaptureError::CaptureFailed));
    }

    #[test]
    fn photo_terminal_error_precedes_processed_bytes() {
        let result = settle_photo_result(Some(Ok(vec![1, 2, 3])), true);
        assert_eq!(result, Err(PhotoCaptureError::CaptureFailed));
    }

    #[test]
    fn photo_missing_processed_data_is_reported() {
        let result: PhotoCaptureResult = settle_photo_result(None, false);
        assert_eq!(result, Err(PhotoCaptureError::PhotoDataUnavailable));
    }

    #[test]
    fn photo_completion_is_taken_and_invoked_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_completion = Arc::clone(&calls);
        let mut completion = Some(Box::new(move |result| {
            assert_eq!(result, Ok(vec![9_u8, 8, 7]));
            calls_for_completion.fetch_add(1, Ordering::SeqCst);
        }) as Box<dyn FnOnce(PhotoCaptureResult) + Send>);

        let first = take_photo_completion(&mut completion, Some(Ok(vec![9, 8, 7])), false);
        let second = take_photo_completion(&mut completion, Some(Ok(vec![9, 8, 7])), false);
        assert!(second.is_none());
        let (completion, result) = first.expect("first terminal callback settles the request");
        completion(result);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn movie_destination_requires_absolute_mov_and_rejects_existing_files() {
        let relative = PathBuf::from("relative.mov");
        assert_eq!(
            validate_movie_destination(&relative),
            Err(VideoRecordingError::InvalidDestination)
        );

        let path = unused_movie_path("existing");
        std::fs::write(&path, b"already exists").unwrap();
        assert_eq!(
            validate_movie_destination(&path),
            Err(VideoRecordingError::DestinationAlreadyExists)
        );
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn movie_destination_rejects_non_utf8_paths_and_existing_symlinks() {
        use std::os::unix::ffi::OsStringExt;

        let mut non_utf8_bytes = std::env::temp_dir().into_os_string().into_vec();
        non_utf8_bytes.extend_from_slice(b"/invalid-\xff.mov");
        let non_utf8 = PathBuf::from(std::ffi::OsString::from_vec(non_utf8_bytes));
        assert_eq!(
            validate_movie_destination(&non_utf8),
            Err(VideoRecordingError::InvalidPathEncoding)
        );

        let path = unused_movie_path("symlink-target");
        let link = unused_movie_path("symlink");
        std::fs::write(&path, b"target").unwrap();
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert_eq!(
            validate_movie_destination(&link),
            Err(VideoRecordingError::DestinationAlreadyExists)
        );
        std::fs::remove_file(link).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn movie_recording_state_keeps_active_until_terminal_and_defers_session_stop() {
        let mut state = MovieRecordingState::default();
        assert_eq!(state.begin(), Ok(()));
        assert_eq!(
            state.begin(),
            Err(VideoRecordingError::RecordingAlreadyActive)
        );
        assert!(state.request_recording_stop());
        assert!(!state.request_recording_stop());
        assert!(state.request_session_stop());
        assert!(!state.request_recording_stop());
        assert!(state.active);
        assert!(state.pending_session_stop);
        assert!(state.settle_terminal());
        assert!(!state.active);
        assert!(!state.pending_session_stop);
        assert!(!state.settle_terminal());
    }

    #[test]
    fn movie_start_is_rejected_while_deferred_session_shutdown_is_pending() {
        let mut state = MovieRecordingState::default();
        assert_eq!(state.begin(), Ok(()));
        assert!(state.request_session_stop());
        assert_eq!(state.begin(), Err(VideoRecordingError::SessionStopping));
        assert_eq!(
            photo_request_error(true, true, state.pending_session_stop),
            Some(PhotoCaptureError::SessionStopping)
        );
        assert_eq!(photo_request_error(true, true, false), None);
    }

    #[test]
    fn movie_terminal_error_precedes_a_nonempty_playable_file() {
        let path = unused_movie_path("terminal-error");
        std::fs::write(&path, b"playable movie bytes").unwrap();
        assert_eq!(
            settle_movie_result(&path, true),
            Err(VideoRecordingError::RecordingFailed)
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn movie_missing_or_empty_finalized_file_is_failure() {
        let missing = unused_movie_path("missing");
        let empty = unused_movie_path("empty");
        assert_eq!(
            settle_movie_result(&missing, false),
            Err(VideoRecordingError::FinalizedFileUnavailable)
        );
        std::fs::write(&empty, b"").unwrap();
        assert_eq!(
            settle_movie_result(&empty, false),
            Err(VideoRecordingError::FinalizedFileUnavailable)
        );
        std::fs::remove_file(empty).unwrap();
    }

    #[test]
    fn movie_terminal_claim_invokes_completion_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_completion = Arc::clone(&calls);
        let mut completion = Some(Box::new(move |result| {
            assert_eq!(result, Err(VideoRecordingError::RecordingFailed));
            calls_for_completion.fetch_add(1, Ordering::SeqCst);
        }) as Box<dyn FnOnce(VideoRecordingResult) + Send>);
        let mut terminal_claimed = false;
        let first = take_video_completion(&mut terminal_claimed, &mut completion);
        let second = take_video_completion(&mut terminal_claimed, &mut completion);
        assert!(second.is_none());
        first.expect("the first terminal callback claims the completion")(Err(
            VideoRecordingError::RecordingFailed,
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
