#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Default-camera AVFoundation capture and preview for iOS"]

#[cfg(any(target_os = "ios", test))]
use framework_media::MediaTime;
#[cfg(any(target_os = "ios", test))]
use framework_media_authorization::MediaAuthorizationStatus;
#[cfg(target_os = "ios")]
use std::ffi::c_void;
#[cfg(target_os = "ios")]
use std::marker::PhantomData;
#[cfg(any(target_os = "ios", test))]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(any(target_os = "ios", test))]
use std::path::{Path, PathBuf};
#[cfg(target_os = "ios")]
use std::ptr::NonNull;
#[cfg(target_os = "ios")]
use std::rc::Rc;
#[cfg(any(target_os = "ios", test))]
use std::sync::atomic::{AtomicBool, Ordering};
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
    AVCaptureConnection, AVCaptureDevice, AVCaptureDeviceInput, AVCaptureDevicePosition,
    AVCaptureDeviceTypeBuiltInWideAngleCamera, AVCaptureExposureMode, AVCaptureFileOutput,
    AVCaptureFileOutputRecordingDelegate, AVCaptureFocusMode, AVCaptureMovieFileOutput,
    AVCaptureOutput, AVCapturePhoto, AVCapturePhotoCaptureDelegate, AVCapturePhotoOutput,
    AVCapturePhotoSettings, AVCaptureSession, AVCaptureTorchMode, AVCaptureVideoDataOutput,
    AVCaptureVideoDataOutputSampleBufferDelegate, AVCaptureVideoPreviewLayer,
    AVCaptureWhiteBalanceMode, AVMediaTypeAudio, AVMediaTypeVideo,
};
#[cfg(target_os = "ios")]
use objc2_core_foundation::CGPoint;
#[cfg(target_os = "ios")]
use objc2_core_media::{CMSampleBuffer, CMTime, CMTimeFlags};
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
    /// The app has not been granted microphone access
    MicrophoneAuthorizationNotGranted,
    /// The public AVFoundation audio media-type constant is unavailable
    AudioMediaTypeUnavailable,
    /// The system has no default audio capture device
    NoDefaultMicrophone,
    /// AVFoundation could not create an input for the selected audio device
    AudioInputCreationFailed,
    /// AVFoundation rejected the microphone input for the running session
    SessionRejectedAudioInput,
    /// A microphone input remains attached outside an active audio movie request
    AudioInputAlreadyAttached,
    /// AVFoundation reported a terminal error for the recording
    RecordingFailed,
    /// AVFoundation finalized no nonempty movie file at the requested destination
    FinalizedFileUnavailable,
}

/// Failure to set the orientation for a future camera movie recording
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovieOrientationError {
    /// The public AVFoundation video media-type constant is unavailable
    VideoMediaTypeUnavailable,
    /// The configured capture session does not have its movie-file output
    MovieOutputUnavailable,
    /// The movie-file output has no video connection
    VideoConnectionUnavailable,
    /// The video connection does not support the requested orientation
    OrientationUnsupported,
    /// A movie recording or its finalization is active
    RecordingActive,
    /// The session has a shutdown request pending
    SessionStopping,
}

#[cfg(any(target_os = "ios", test))]
type VideoRecordingResult = Result<PathBuf, VideoRecordingError>;

#[cfg(any(target_os = "ios", test))]
type VideoRecordingCompletion = Box<dyn FnOnce(VideoRecordingResult) + Send + 'static>;

/// Failure to start a default-camera video sample stream
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoFrameStreamError {
    /// The capture session has not completed startup or is no longer running
    SessionNotRunning,
    /// The session is stopping after a movie-file recording
    SessionStopping,
    /// AVFoundation rejected the video-data output for the running session
    OutputRejected,
    /// A video sample stream is already active
    AlreadyActive,
    /// The video sample stream has stopped and cannot restart on this session
    AlreadyStopped,
}

#[cfg(target_os = "ios")]
type VideoFrameCallback = Box<dyn FnMut(&CMSampleBuffer) + Send + 'static>;

#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum VideoFrameStreamState {
    #[default]
    NotStarted,
    Active,
    StopRequested,
    Stopped,
}

#[cfg(any(target_os = "ios", test))]
impl VideoFrameStreamState {
    fn begin(&mut self) -> Result<(), VideoFrameStreamError> {
        match self {
            Self::NotStarted => {
                *self = Self::Active;
                Ok(())
            }
            Self::Active => Err(VideoFrameStreamError::AlreadyActive),
            Self::StopRequested | Self::Stopped => Err(VideoFrameStreamError::AlreadyStopped),
        }
    }

    fn request_stop(&mut self) -> bool {
        if *self == Self::Active {
            *self = Self::StopRequested;
            true
        } else {
            false
        }
    }

    fn close_unstarted(&mut self) -> bool {
        if *self == Self::NotStarted {
            *self = Self::Stopped;
            true
        } else {
            false
        }
    }

    fn finish_stop(&mut self) -> bool {
        if *self == Self::StopRequested {
            *self = Self::Stopped;
            true
        } else {
            false
        }
    }
}

#[cfg(any(target_os = "ios", test))]
fn invoke_video_frame_callback<T, F: FnMut(&T)>(
    active: &AtomicBool,
    callback: &mut F,
    sample: &T,
) -> bool {
    if !active.load(Ordering::Acquire) {
        return false;
    }
    if catch_unwind(AssertUnwindSafe(|| callback(sample))).is_err() {
        active.store(false, Ordering::Release);
        return false;
    }
    true
}

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
        self.pending_session_stop
    }

    fn finish_session_stop(&mut self) {
        self.pending_session_stop = false;
    }
}

#[cfg(any(target_os = "ios", test))]
fn map_camera_video_orientation<T: Copy>(
    orientation: IosCameraVideoOrientation,
    portrait: T,
    portrait_upside_down: T,
    landscape_right: T,
    landscape_left: T,
) -> T {
    match orientation {
        IosCameraVideoOrientation::Portrait => portrait,
        IosCameraVideoOrientation::PortraitUpsideDown => portrait_upside_down,
        IosCameraVideoOrientation::LandscapeRight => landscape_right,
        IosCameraVideoOrientation::LandscapeLeft => landscape_left,
    }
}

#[cfg(any(target_os = "ios", test))]
fn run_on_capture_queue<T, F, S>(is_current_queue: bool, work: F, sync: S) -> T
where
    T: Send,
    F: FnOnce() -> T + Send,
    S: FnOnce(F) -> T,
{
    if is_current_queue { work() } else { sync(work) }
}

#[cfg(any(target_os = "ios", test))]
fn camera_position_switch_required(
    movie_state: &MovieRecordingState,
    current_position: Option<IosCameraPosition>,
    has_video_input: bool,
    requested: IosCameraPosition,
) -> Result<bool, CameraPositionSwitchError> {
    if movie_state.pending_session_stop {
        return Err(CameraPositionSwitchError::SessionStopping);
    }
    if movie_state.active {
        return Err(CameraPositionSwitchError::MovieRecordingActive);
    }
    if !has_video_input {
        return Err(CameraPositionSwitchError::NoSelectedCamera);
    }
    Ok(current_position != Some(requested))
}

#[cfg(any(target_os = "ios", test))]
fn replace_camera_session_input<T>(
    old_input: &T,
    new_input: T,
    mut can_add_input: impl FnMut(&T) -> bool,
    remove_input: impl FnOnce(&T),
    mut add_input: impl FnMut(&T),
    begin_configuration: impl FnOnce(),
    commit_configuration: impl FnOnce(),
) -> Result<T, CameraPositionSwitchError> {
    begin_configuration();
    remove_input(old_input);
    if can_add_input(&new_input) {
        add_input(&new_input);
        commit_configuration();
        Ok(new_input)
    } else if can_add_input(old_input) {
        add_input(old_input);
        commit_configuration();
        Err(CameraPositionSwitchError::SessionRejectedInput)
    } else {
        commit_configuration();
        Err(CameraPositionSwitchError::InputRollbackFailed)
    }
}

#[cfg(any(target_os = "ios", test))]
fn finish_camera_input_replacement<D, I>(
    device: &mut Option<D>,
    input: &mut Option<I>,
    current_position: &mut Option<IosCameraPosition>,
    requested: IosCameraPosition,
    replacement: Result<(D, I), CameraPositionSwitchError>,
) -> Result<(), CameraPositionSwitchError> {
    match replacement {
        Ok((new_device, new_input)) => {
            *input = Some(new_input);
            *device = Some(new_device);
            *current_position = Some(requested);
            Ok(())
        }
        Err(error) => {
            if error == CameraPositionSwitchError::InputRollbackFailed {
                *input = None;
                *device = None;
                *current_position = None;
            }
            Err(error)
        }
    }
}

#[cfg(any(target_os = "ios", test))]
fn set_supported_movie_orientation<C>(
    connection: &C,
    orientation: IosCameraVideoOrientation,
    is_supported: impl FnOnce(&C) -> bool,
    set_orientation: impl FnOnce(&C, IosCameraVideoOrientation),
) -> Result<(), MovieOrientationError> {
    if !is_supported(connection) {
        return Err(MovieOrientationError::OrientationUnsupported);
    }
    set_orientation(connection, orientation);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn configure_movie_orientation<T, C>(
    movie_state: &MovieRecordingState,
    movie_output: Option<&T>,
    orientation: IosCameraVideoOrientation,
    video_connection: impl FnOnce(&T) -> Result<Option<C>, MovieOrientationError>,
    set_connection_orientation: impl FnOnce(
        &C,
        IosCameraVideoOrientation,
    ) -> Result<(), MovieOrientationError>,
) -> Result<(), MovieOrientationError> {
    if movie_state.pending_session_stop {
        return Err(MovieOrientationError::SessionStopping);
    }
    if movie_state.active {
        return Err(MovieOrientationError::RecordingActive);
    }
    let output = movie_output.ok_or(MovieOrientationError::MovieOutputUnavailable)?;
    let connection =
        video_connection(output)?.ok_or(MovieOrientationError::VideoConnectionUnavailable)?;
    set_connection_orientation(&connection, orientation)
}

#[cfg(target_os = "ios")]
#[allow(deprecated)]
fn configure_avfoundation_movie_orientation(
    connection: &AVCaptureConnection,
    orientation: IosCameraVideoOrientation,
) -> Result<(), MovieOrientationError> {
    set_supported_movie_orientation(
        connection,
        orientation,
        // SAFETY: The public orientation support query is available from iOS 4.0, below both
        // package floors. It is checked before the setter to avoid AVFoundation exceptions.
        |connection| unsafe { connection.isVideoOrientationSupported() },
        // SAFETY: set_supported_movie_orientation invokes this only after AVFoundation confirms
        // support for the exact requested public orientation.
        |connection, orientation| unsafe {
            connection.setVideoOrientation(map_camera_video_orientation(
                orientation,
                objc2_av_foundation::AVCaptureVideoOrientation::Portrait,
                objc2_av_foundation::AVCaptureVideoOrientation::PortraitUpsideDown,
                objc2_av_foundation::AVCaptureVideoOrientation::LandscapeRight,
                objc2_av_foundation::AVCaptureVideoOrientation::LandscapeLeft,
            ))
        },
    )
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
struct VideoFrameDelegateState {
    callback: Option<VideoFrameCallback>,
    active: Arc<AtomicBool>,
    graph: std::sync::Weak<Mutex<CaptureGraph>>,
    session_queue: dispatch2::DispatchRetained<DispatchQueue>,
}

#[cfg(target_os = "ios")]
impl Drop for VideoFrameDelegateState {
    fn drop(&mut self) {
        drop_video_frame_callback_safely(self.callback.take());
    }
}

#[cfg(target_os = "ios")]
struct VideoFrameDelegateIvars {
    state: Mutex<VideoFrameDelegateState>,
}

#[cfg(target_os = "ios")]
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = AnyThread]
    #[ivars = VideoFrameDelegateIvars]
    struct VideoFrameDelegateObject;

    // SAFETY: NSObjectProtocol adds no methods; callback state is protected by its mutex.
    unsafe impl NSObjectProtocol for VideoFrameDelegateObject {}

    // SAFETY: AVFoundation calls this delegate only on its configured serial callback queue.
    // The sample buffer is borrowed only for the callback and callback panics never cross ObjC.
    unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for VideoFrameDelegateObject {
        #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
        #[allow(non_snake_case)]
        unsafe fn captureOutput_didOutputSampleBuffer_fromConnection(
            &self,
            _output: &AVCaptureOutput,
            sample_buffer: &CMSampleBuffer,
            _connection: &AVCaptureConnection,
        ) {
            let _receiver = unsafe { retain_video_frame_callback_receiver(self) };
            let mut state = self
                .ivars()
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let active = Arc::clone(&state.active);
            let stop_after_callback = state.callback.as_mut().is_some_and(|callback| {
                !invoke_video_frame_callback(&active, callback, sample_buffer)
            });
            let callback_to_drop = if stop_after_callback {
                state.callback.take()
            } else {
                None
            };
            if stop_after_callback {
                let graph = state.graph.clone();
                let session_queue = state.session_queue.clone();
                drop(state);
                enqueue_video_frame_stream_stop_after_callback(&session_queue, graph);
            }
            drop_video_frame_callback_safely(callback_to_drop);
        }
    }
);

#[cfg(target_os = "ios")]
impl VideoFrameDelegateObject {
    fn new(
        callback: VideoFrameCallback,
        active: Arc<AtomicBool>,
        graph: std::sync::Weak<Mutex<CaptureGraph>>,
        session_queue: dispatch2::DispatchRetained<DispatchQueue>,
    ) -> Retained<Self> {
        let ivars = VideoFrameDelegateIvars {
            state: Mutex::new(VideoFrameDelegateState {
                callback: Some(callback),
                active,
                graph,
                session_queue,
            }),
        };
        let allocated = Self::alloc().set_ivars(ivars);
        // SAFETY: The delegate ivars are initialized before NSObject's initializer runs.
        unsafe { msg_send![super(allocated), init] }
    }

    fn take_and_drop_callback(&self) {
        let callback = self
            .ivars()
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .callback
            .take();
        drop_video_frame_callback_safely(callback);
    }
}

#[cfg(any(target_os = "ios", test))]
fn drop_video_frame_callback_safely<T>(callback: Option<T>) {
    let _ = catch_unwind(AssertUnwindSafe(|| drop(callback)));
}

#[cfg(target_os = "ios")]
unsafe fn retain_video_frame_callback_receiver(
    receiver: &VideoFrameDelegateObject,
) -> Option<Retained<VideoFrameDelegateObject>> {
    // SAFETY: objc2 supplies a live callback receiver; this retain keeps its ivars valid through
    // callback completion and any deferred graph cleanup.
    unsafe {
        Retained::retain(
            receiver as *const VideoFrameDelegateObject as *mut VideoFrameDelegateObject,
        )
    }
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
                settle_movie_audio_on_queue(&mut graph_state)
            };
            invoke_video_completion(completion, result);
            if stop_session {
                let mut graph_state = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // SAFETY: File finalization and its user completion have reached terminal state;
                // this settlement runs on the graph's private serial queue.
                unsafe { graph_state.session.stopRunning() };
                graph_state.movie_recording.finish_session_stop();
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

#[cfg(any(target_os = "ios", test))]
fn with_authorized_microphone<T>(
    status: MediaAuthorizationStatus,
    create_input: impl FnOnce() -> Result<T, VideoRecordingError>,
) -> Result<T, VideoRecordingError> {
    if may_create_device_input(status) {
        create_input()
    } else {
        Err(VideoRecordingError::MicrophoneAuthorizationNotGranted)
    }
}

#[cfg(any(target_os = "ios", test))]
fn prepare_optional_movie_audio(
    enabled: bool,
    prepare: impl FnOnce() -> Result<(), VideoRecordingError>,
) -> Result<(), VideoRecordingError> {
    if enabled { prepare() } else { Ok(()) }
}

#[cfg(any(target_os = "ios", test))]
fn settle_movie_terminal<T>(
    state: &mut MovieRecordingState,
    audio_input: &mut Option<T>,
    remove_input: impl FnOnce(&T),
) -> bool {
    if let Some(input) = audio_input.as_ref() {
        remove_input(input);
        drop(audio_input.take());
    }
    state.settle_terminal()
}

#[cfg(target_os = "ios")]
fn settle_movie_audio_on_queue(graph: &mut CaptureGraph) -> bool {
    let CaptureGraph {
        session,
        movie_audio_input,
        movie_recording,
        ..
    } = graph;
    settle_movie_terminal(movie_recording, movie_audio_input, |input| {
        // SAFETY: Movie finalization is terminal and all graph changes use the private serial
        // queue. Remove the opt-in input before completion or deferred session stop.
        unsafe { session.beginConfiguration() };
        // SAFETY: This exact input was added after `canAddInput` accepted it.
        unsafe { session.removeInput(input) };
        // SAFETY: Every beginConfiguration call is paired with this commit.
        unsafe { session.commitConfiguration() };
    })
}

/// Front- or back-facing built-in wide-angle camera position
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCameraPosition {
    /// The user-facing built-in wide-angle camera
    Front,
    /// The subject-facing built-in wide-angle camera
    Back,
}

/// Failure to switch an existing session's selected built-in wide-angle camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraPositionSwitchError {
    /// The operating system predates the iOS 10 camera-position selector
    UnsupportedOperatingSystem,
    /// The public AVFoundation video media-type constant is unavailable
    VideoMediaTypeUnavailable,
    /// The requested built-in wide-angle camera is unavailable
    NoCameraAtPosition(IosCameraPosition),
    /// AVFoundation could not create an input for the requested camera
    DeviceInputCreationFailed,
    /// The capture session has no selected video device/input to replace
    NoSelectedCamera,
    /// AVFoundation rejected the requested input; the prior input was restored
    SessionRejectedInput,
    /// AVFoundation rejected the requested input and could not restore the prior input
    InputRollbackFailed,
    /// A movie recording or its terminal finalization is active
    MovieRecordingActive,
    /// Session shutdown is pending after an active movie recording
    SessionStopping,
}

/// Caller-selected orientation for a future movie-file recording
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCameraVideoOrientation {
    /// Video is vertical with the home button at the bottom
    Portrait,
    /// Video is vertical with the home button at the top
    PortraitUpsideDown,
    /// Video is horizontal with the home button at the right
    LandscapeRight,
    /// Video is horizontal with the home button at the left
    LandscapeLeft,
}

/// Failure to create or configure a camera capture session
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraCaptureError {
    /// The app has not been granted camera access
    AuthorizationNotGranted,
    /// The public AVFoundation video media-type constant is unavailable
    VideoMediaTypeUnavailable,
    /// The system has no default video capture device
    NoDefaultVideoDevice,
    /// The system has no built-in wide-angle video device at the requested position
    NoCameraAtPosition(IosCameraPosition),
    /// AVFoundation could not create an input for the selected video device
    DeviceInputCreationFailed,
    /// AVFoundation rejected the selected video input for the new session
    SessionRejectedInput,
    /// AVFoundation rejected the still-photo output for the new session
    SessionRejectedPhotoOutput,
    /// AVFoundation rejected the movie-file output for the new session
    SessionRejectedMovieOutput,
    /// The operating system predates a required AVFoundation API
    UnsupportedOperatingSystem,
}

/// Failure to control the torch on the session's selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TorchControlError {
    /// The session graph has no configured video device
    NoSelectedCamera,
    /// The selected camera has no torch
    NoTorch,
    /// The selected camera does not support the requested torch mode
    TorchModeUnsupported,
    /// The torch is not currently available to enable
    TorchUnavailable,
    /// AVFoundation could not lock the selected camera for configuration
    ConfigurationLockFailed,
}

/// Failure to set zoom on the session's selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoomControlError {
    /// The session graph has no configured video device
    NoSelectedCamera,
    /// The requested zoom factor is not finite
    InvalidZoomFactor,
    /// The requested factor is outside the supported inclusive range
    ZoomFactorOutOfRange,
    /// AVFoundation reported an invalid active-format zoom range
    InvalidSupportedRange,
    /// AVFoundation could not lock the selected camera for configuration
    ConfigurationLockFailed,
}

/// Public AVFoundation focus behavior for the selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCameraFocusMode {
    /// Autofocus once, then hold the resulting lens position
    Auto,
    /// Continue to autofocus when the camera determines it is needed
    ContinuousAuto,
    /// Hold the current lens position
    Locked,
}

/// A normalized point in the selected camera's image coordinates
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedFocusPoint {
    x: f64,
    y: f64,
}

#[cfg(any(target_os = "ios", test))]
impl NormalizedFocusPoint {
    /// Create a focus point; `set_focus` rejects non-finite or out-of-range coordinates
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Return the normalized horizontal coordinate
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Return the normalized vertical coordinate
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// Failure to configure focus on the session's selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusControlError {
    /// The session graph has no configured video device
    NoSelectedCamera,
    /// Either normalized point coordinate is non-finite or outside `[0.0, 1.0]`
    InvalidFocusPoint,
    /// A locked focus mode cannot apply a requested focus point
    LockedModeCannotUseFocusPoint,
    /// The selected camera does not support the requested focus mode
    FocusModeUnsupported,
    /// The selected camera does not support focus points of interest
    FocusPointUnsupported,
    /// AVFoundation could not lock the selected camera for configuration
    ConfigurationLockFailed,
}

/// Public AVFoundation white-balance behavior for the selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCameraWhiteBalanceMode {
    /// Hold the current white-balance values
    Locked,
    /// Adjust white balance once, then lock the resulting values
    AutoWhiteBalance,
    /// Continue to adjust white balance automatically
    ContinuousAutoWhiteBalance,
}

/// Failure to query or configure white balance on the session's selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhiteBalanceControlError {
    /// The session graph has no configured video device
    NoSelectedCamera,
    /// The selected camera does not support the requested white-balance mode
    WhiteBalanceModeUnsupported,
    /// AVFoundation could not lock the selected camera for configuration
    ConfigurationLockFailed,
}

/// Public AVFoundation exposure behavior for the selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCameraExposureMode {
    /// Determine exposure once, then hold the resulting exposure settings
    AutoExpose,
    /// Continue to adjust exposure as the camera determines it is needed
    ContinuousAutoExposure,
    /// Hold the current exposure settings
    Locked,
}

/// Inclusive exposure target-bias range reported by the selected camera, in EV
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosExposureTargetBiasRange {
    /// Minimum supported exposure target bias, in EV
    pub minimum: f32,
    /// Maximum supported exposure target bias, in EV
    pub maximum: f32,
}

/// Inclusive custom-exposure bounds for the selected camera's current format
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosCustomExposureRange {
    /// Minimum supported exposure duration as an exact rational number of seconds
    pub minimum_duration: MediaTime,
    /// Maximum supported exposure duration as an exact rational number of seconds
    pub maximum_duration: MediaTime,
    /// Minimum supported ISO value
    pub minimum_iso: f32,
    /// Maximum supported ISO value
    pub maximum_iso: f32,
}

/// Failure to configure exposure on the session's selected camera
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExposureControlError {
    /// The session graph has no configured video device
    NoSelectedCamera,
    /// Either normalized point coordinate is non-finite or outside `[0.0, 1.0]`
    InvalidExposurePoint,
    /// A locked exposure mode cannot apply a requested exposure point
    LockedModeCannotUseExposurePoint,
    /// The selected camera does not support the requested exposure mode
    ExposureModeUnsupported,
    /// The selected camera does not support exposure points of interest
    ExposurePointUnsupported,
    /// The requested exposure target bias is not finite
    InvalidExposureBias,
    /// The requested exposure target bias is outside the selected camera's inclusive range
    ExposureBiasOutOfRange,
    /// AVFoundation reported non-finite or reversed exposure target bias bounds
    InvalidSupportedExposureBiasRange,
    /// The custom exposure duration must be strictly positive
    InvalidCustomExposureDuration,
    /// The custom ISO value must be finite and strictly positive
    InvalidCustomExposureIso,
    /// The selected camera does not support custom exposure mode
    CustomExposureModeUnsupported,
    /// The requested exact duration is outside the active format's inclusive range
    CustomExposureDurationOutOfRange,
    /// The requested ISO value is outside the active format's inclusive range
    CustomExposureIsoOutOfRange,
    /// AVFoundation reported invalid duration or ISO bounds for its active format
    InvalidSupportedCustomExposureRange,
    /// AVFoundation could not lock the selected camera for configuration
    ConfigurationLockFailed,
}

#[cfg(any(target_os = "ios", test))]
struct UnlockDeviceConfiguration<F: FnOnce()>(Option<F>);

#[cfg(any(target_os = "ios", test))]
impl<F: FnOnce()> Drop for UnlockDeviceConfiguration<F> {
    fn drop(&mut self) {
        if let Some(unlock) = self.0.take() {
            unlock();
        }
    }
}

#[cfg(any(target_os = "ios", test))]
fn configure_torch_enabled(
    enabled: bool,
    has_torch: impl FnOnce() -> bool,
    supports_mode: impl FnOnce(bool) -> bool,
    is_available: impl FnOnce() -> bool,
    lock: impl FnOnce() -> Result<(), ()>,
    set_mode: impl FnOnce(bool),
    unlock: impl FnOnce(),
) -> Result<(), TorchControlError> {
    if !has_torch() {
        return Err(TorchControlError::NoTorch);
    }
    if !supports_mode(enabled) {
        return Err(TorchControlError::TorchModeUnsupported);
    }
    if enabled && !is_available() {
        return Err(TorchControlError::TorchUnavailable);
    }
    lock().map_err(|()| TorchControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    set_mode(enabled);
    drop(unlock);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn require_selected_camera<T>(device: Option<T>) -> Result<T, ZoomControlError> {
    device.ok_or(ZoomControlError::NoSelectedCamera)
}

#[cfg(any(target_os = "ios", test))]
fn configure_video_zoom_factor(
    factor: f64,
    supported_max: impl FnOnce() -> f64,
    lock: impl FnOnce() -> Result<(), ()>,
    set_factor: impl FnOnce(f64),
    unlock: impl FnOnce(),
) -> Result<(), ZoomControlError> {
    if !factor.is_finite() {
        return Err(ZoomControlError::InvalidZoomFactor);
    }
    if factor < 1.0 {
        return Err(ZoomControlError::ZoomFactorOutOfRange);
    }
    lock().map_err(|()| ZoomControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    let supported_max = supported_max();
    if !supported_max.is_finite() || supported_max < 1.0 {
        return Err(ZoomControlError::InvalidSupportedRange);
    }
    if factor > supported_max {
        return Err(ZoomControlError::ZoomFactorOutOfRange);
    }
    set_factor(factor);
    drop(unlock);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn require_selected_focus_camera<T>(device: Option<T>) -> Result<T, FocusControlError> {
    device.ok_or(FocusControlError::NoSelectedCamera)
}

#[cfg(any(target_os = "ios", test))]
fn valid_focus_point(point: NormalizedFocusPoint) -> bool {
    point.x.is_finite()
        && point.y.is_finite()
        && (0.0..=1.0).contains(&point.x)
        && (0.0..=1.0).contains(&point.y)
}

#[cfg(any(target_os = "ios", test))]
fn configure_focus(
    mode: IosCameraFocusMode,
    point: Option<NormalizedFocusPoint>,
    validate_support: impl FnOnce(
        IosCameraFocusMode,
        Option<NormalizedFocusPoint>,
    ) -> Result<(), FocusControlError>,
    lock: impl FnOnce() -> Result<(), ()>,
    set_point: impl FnOnce(NormalizedFocusPoint),
    set_mode: impl FnOnce(IosCameraFocusMode),
    unlock: impl FnOnce(),
) -> Result<(), FocusControlError> {
    if point.is_some_and(|point| !valid_focus_point(point)) {
        return Err(FocusControlError::InvalidFocusPoint);
    }
    if mode == IosCameraFocusMode::Locked && point.is_some() {
        return Err(FocusControlError::LockedModeCannotUseFocusPoint);
    }
    validate_support(mode, point)?;
    lock().map_err(|()| FocusControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    if let Some(point) = point {
        set_point(point);
    }
    set_mode(mode);
    drop(unlock);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn with_selected_exposure_camera<T, R>(
    device: Option<T>,
    operation: impl FnOnce(T) -> Result<R, ExposureControlError>,
) -> Result<R, ExposureControlError> {
    operation(device.ok_or(ExposureControlError::NoSelectedCamera)?)
}

#[cfg(any(target_os = "ios", test))]
fn configure_exposure(
    mode: IosCameraExposureMode,
    point: Option<NormalizedFocusPoint>,
    validate_support: impl FnOnce(
        IosCameraExposureMode,
        Option<NormalizedFocusPoint>,
    ) -> Result<(), ExposureControlError>,
    lock: impl FnOnce() -> Result<(), ()>,
    set_point: impl FnOnce(NormalizedFocusPoint),
    set_mode: impl FnOnce(IosCameraExposureMode),
    unlock: impl FnOnce(),
) -> Result<(), ExposureControlError> {
    if point.is_some_and(|point| !valid_focus_point(point)) {
        return Err(ExposureControlError::InvalidExposurePoint);
    }
    if mode == IosCameraExposureMode::Locked && point.is_some() {
        return Err(ExposureControlError::LockedModeCannotUseExposurePoint);
    }
    validate_support(mode, point)?;
    lock().map_err(|()| ExposureControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    if let Some(point) = point {
        set_point(point);
    }
    set_mode(mode);
    drop(unlock);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn with_selected_white_balance_camera<T, R>(
    device: Option<T>,
    operation: impl FnOnce(T) -> Result<R, WhiteBalanceControlError>,
) -> Result<R, WhiteBalanceControlError> {
    operation(device.ok_or(WhiteBalanceControlError::NoSelectedCamera)?)
}

#[cfg(any(target_os = "ios", test))]
fn configure_white_balance_mode(
    mode: IosCameraWhiteBalanceMode,
    is_supported: impl FnOnce(IosCameraWhiteBalanceMode) -> bool,
    lock: impl FnOnce() -> Result<(), ()>,
    set_mode: impl FnOnce(IosCameraWhiteBalanceMode),
    unlock: impl FnOnce(),
) -> Result<(), WhiteBalanceControlError> {
    if !is_supported(mode) {
        return Err(WhiteBalanceControlError::WhiteBalanceModeUnsupported);
    }
    lock().map_err(|()| WhiteBalanceControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    set_mode(mode);
    drop(unlock);
    Ok(())
}

#[cfg(target_os = "ios")]
fn native_white_balance_mode(mode: IosCameraWhiteBalanceMode) -> AVCaptureWhiteBalanceMode {
    match mode {
        IosCameraWhiteBalanceMode::Locked => AVCaptureWhiteBalanceMode::Locked,
        IosCameraWhiteBalanceMode::AutoWhiteBalance => AVCaptureWhiteBalanceMode::AutoWhiteBalance,
        IosCameraWhiteBalanceMode::ContinuousAutoWhiteBalance => {
            AVCaptureWhiteBalanceMode::ContinuousAutoWhiteBalance
        }
    }
}

#[cfg(any(target_os = "ios", test))]
fn exposure_target_bias_range(
    minimum: f32,
    maximum: f32,
) -> Result<IosExposureTargetBiasRange, ExposureControlError> {
    if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
        return Err(ExposureControlError::InvalidSupportedExposureBiasRange);
    }
    Ok(IosExposureTargetBiasRange { minimum, maximum })
}

#[cfg(any(target_os = "ios", test))]
fn custom_exposure_duration_from_parts(
    value: i64,
    timescale: i32,
    valid: bool,
    finite: bool,
    epoch: i64,
) -> Result<MediaTime, ExposureControlError> {
    if !valid || !finite || epoch != 0 || value <= 0 {
        return Err(ExposureControlError::InvalidSupportedCustomExposureRange);
    }
    MediaTime::new(value, timescale)
        .map_err(|_| ExposureControlError::InvalidSupportedCustomExposureRange)
}

#[cfg(any(target_os = "ios", test))]
fn custom_exposure_range(
    minimum_duration: MediaTime,
    maximum_duration: MediaTime,
    minimum_iso: f32,
    maximum_iso: f32,
) -> Result<IosCustomExposureRange, ExposureControlError> {
    if minimum_duration.value() <= 0
        || maximum_duration.value() <= 0
        || minimum_duration > maximum_duration
        || !minimum_iso.is_finite()
        || !maximum_iso.is_finite()
        || minimum_iso <= 0.0
        || minimum_iso > maximum_iso
    {
        return Err(ExposureControlError::InvalidSupportedCustomExposureRange);
    }
    Ok(IosCustomExposureRange {
        minimum_duration,
        maximum_duration,
        minimum_iso,
        maximum_iso,
    })
}

#[cfg(any(target_os = "ios", test))]
fn configure_custom_exposure(
    duration: MediaTime,
    iso: f32,
    is_supported: impl FnOnce() -> bool,
    lock: impl FnOnce() -> Result<(), ()>,
    read_range: impl FnOnce() -> Result<IosCustomExposureRange, ExposureControlError>,
    set: impl FnOnce(MediaTime, f32),
    unlock: impl FnOnce(),
) -> Result<(), ExposureControlError> {
    if duration.value() <= 0 {
        return Err(ExposureControlError::InvalidCustomExposureDuration);
    }
    if !iso.is_finite() || iso <= 0.0 {
        return Err(ExposureControlError::InvalidCustomExposureIso);
    }
    if !is_supported() {
        return Err(ExposureControlError::CustomExposureModeUnsupported);
    }
    lock().map_err(|()| ExposureControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    let range = read_range()?;
    if duration < range.minimum_duration || duration > range.maximum_duration {
        return Err(ExposureControlError::CustomExposureDurationOutOfRange);
    }
    if iso < range.minimum_iso || iso > range.maximum_iso {
        return Err(ExposureControlError::CustomExposureIsoOutOfRange);
    }
    set(duration, iso);
    drop(unlock);
    Ok(())
}

#[cfg(target_os = "ios")]
fn native_custom_exposure_range(
    device: &AVCaptureDevice,
) -> Result<IosCustomExposureRange, ExposureControlError> {
    // SAFETY: The caller holds the retained selected device on the capture queue. The returned
    // format is retained while its public read-only exposure bounds are copied below.
    let format = unsafe { device.activeFormat() };
    // SAFETY: These public active-format exposure bounds are available below both package floors.
    let minimum = unsafe { format.minExposureDuration() };
    // SAFETY: Same retained-format and public-API conditions as the minimum bound above.
    let maximum = unsafe { format.maxExposureDuration() };
    let minimum_iso = unsafe { format.minISO() };
    let maximum_iso = unsafe { format.maxISO() };
    let media_time = |time: CMTime| {
        let value = time.value;
        let timescale = time.timescale;
        let flags = time.flags;
        let epoch = time.epoch;
        custom_exposure_duration_from_parts(
            value,
            timescale,
            flags.contains(CMTimeFlags::Valid),
            !flags.intersects(CMTimeFlags::ImpliedValueFlagsMask),
            epoch,
        )
    };
    custom_exposure_range(
        media_time(minimum)?,
        media_time(maximum)?,
        minimum_iso,
        maximum_iso,
    )
}

#[cfg(target_os = "ios")]
fn native_custom_exposure_duration(duration: MediaTime) -> CMTime {
    CMTime {
        value: duration.value(),
        timescale: duration.timescale(),
        flags: CMTimeFlags::Valid,
        epoch: 0,
    }
}

#[cfg(any(target_os = "ios", test))]
fn configure_exposure_target_bias(
    bias_ev: f32,
    read_range: impl FnOnce() -> (f32, f32),
    lock: impl FnOnce() -> Result<(), ()>,
    set_bias: impl FnOnce(f32),
    unlock: impl FnOnce(),
) -> Result<(), ExposureControlError> {
    if !bias_ev.is_finite() {
        return Err(ExposureControlError::InvalidExposureBias);
    }
    lock().map_err(|()| ExposureControlError::ConfigurationLockFailed)?;
    let unlock = UnlockDeviceConfiguration(Some(unlock));
    let (minimum, maximum) = read_range();
    let range = exposure_target_bias_range(minimum, maximum)?;
    if bias_ev < range.minimum || bias_ev > range.maximum {
        return Err(ExposureControlError::ExposureBiasOutOfRange);
    }
    set_bias(bias_ev);
    drop(unlock);
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn require_camera_at_position<T>(
    position: IosCameraPosition,
    device: Option<T>,
) -> Result<T, CameraCaptureError> {
    device.ok_or(CameraCaptureError::NoCameraAtPosition(position))
}

#[cfg(any(target_os = "ios", test))]
fn require_switch_camera<T>(
    position: IosCameraPosition,
    device: Option<T>,
) -> Result<T, CameraPositionSwitchError> {
    device.ok_or(CameraPositionSwitchError::NoCameraAtPosition(position))
}

#[cfg(target_os = "ios")]
struct CaptureGraph {
    session: Retained<AVCaptureSession>,
    device: Option<Retained<AVCaptureDevice>>,
    video_input: Option<Retained<AVCaptureDeviceInput>>,
    camera_position: Option<IosCameraPosition>,
    photo_output: Option<Retained<AVCapturePhotoOutput>>,
    movie_output: Option<Retained<AVCaptureMovieFileOutput>>,
    video_output: Option<Retained<AVCaptureVideoDataOutput>>,
    video_delegate: Option<Retained<VideoFrameDelegateObject>>,
    video_callback_queue: Option<dispatch2::DispatchRetained<DispatchQueue>>,
    video_callback_active: Arc<AtomicBool>,
    video_stream_state: VideoFrameStreamState,
    movie_audio_input: Option<Retained<AVCaptureDeviceInput>>,
    movie_recording: MovieRecordingState,
}

// SAFETY: The graph is private and every session, device, output, delegate, and state transition is
// protected by its mutex or the dedicated serial queue for that operation. Sample callbacks use
// their own serial queue and only access delegate-local state plus the atomic active flag. The only
// main-thread session access is the synchronous preview-layer initializer after queue setup and
// before publication. Published graph destruction occurs on the session queue after stop, callback
// queue drain, or the movie terminal callback. The preview layer is created/exposed only on main.
#[cfg(target_os = "ios")]
unsafe impl Send for CaptureGraph {}

#[cfg(target_os = "ios")]
fn capture_queue_specific_key(key: &Arc<AtomicBool>) -> NonNull<()> {
    NonNull::from(key.as_ref()).cast()
}

#[cfg(target_os = "ios")]
fn is_current_capture_queue(key: &Arc<AtomicBool>) -> bool {
    // SAFETY: This is the opaque key installed on this queue; the queue-specific destructor
    // retains its allocation for the queue lifetime, and the lookup does not dereference it.
    !unsafe { dispatch2::dispatch_get_specific(capture_queue_specific_key(key).cast::<c_void>()) }
        .is_null()
}

#[cfg(target_os = "ios")]
fn camera_position_for_device(device: &AVCaptureDevice) -> Option<IosCameraPosition> {
    // SAFETY: AVCaptureDevice.position is a public iOS 4.0 property, below the package floor.
    match unsafe { device.position() } {
        AVCaptureDevicePosition::Front => Some(IosCameraPosition::Front),
        AVCaptureDevicePosition::Back => Some(IosCameraPosition::Back),
        _ => None,
    }
}

/// An iOS default-camera session with a live preview-layer escape hatch
///
/// Creation requires camera authorization to be granted already and the host app to provide
/// `NSCameraUsageDescription`. This type does not request permission or mutate shared audio-session
/// state. Video-only movie recording is the default; the opt-in audio movie method also requires
/// current microphone authorization and host-owned audio-session setup. Create and use it on the main thread. Creation synchronously opens and
/// configures the default video input before returning, so it may block while AVFoundation prepares
/// the device. Its preview layer can be added to a UIKit view's layer hierarchy. Start and stop
/// requests run asynchronously on a private serial queue; returning from either method does not
/// prove that AVFoundation started or stopped the session. This wrapper does not observe
/// AVFoundation's native runtime-error notifications. Video recording writes one QuickTime movie
/// to a caller-owned `.mov` path and invokes its completion after AVFoundation finalizes the file.
/// Stopping the session during recording defers `stopRunning` until that terminal callback. The
/// host owns audio-session configuration and may import completed files into Photos separately.
/// An optional video sample stream invokes a Rust callback on a dedicated serial queue with a
/// callback-scoped borrowed `CMSampleBuffer`; callback work should be short because AVFoundation
/// drops late frames. Stop is nonblocking, an active callback may finish after the request, and a
/// stopped stream cannot restart on the same session.
///
/// Compilation and linking validate the native API boundary only; they do not prove camera access
/// or preview behavior on a simulator or physical device.
#[cfg(target_os = "ios")]
pub struct IosCameraCaptureSession {
    graph: Arc<Mutex<CaptureGraph>>,
    queue: dispatch2::DispatchRetained<DispatchQueue>,
    queue_key: Arc<AtomicBool>,
    video_callback_active: Arc<AtomicBool>,
    preview_layer: Retained<AVCaptureVideoPreviewLayer>,
    _main_thread_only: PhantomData<Rc<()>>,
}

#[cfg(target_os = "ios")]
impl IosCameraCaptureSession {
    fn run_capture_queue_sync<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> T {
        let queue = self.queue.clone();
        run_on_capture_queue(
            is_current_capture_queue(&self.queue_key),
            work,
            move |work| {
                let result = Mutex::new(None);
                queue.exec_sync(|| {
                    *result
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(work());
                });
                result
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .expect("capture queue returns a synchronous result")
            },
        )
    }

    /// Create a session for the already-authorized default video capture device
    ///
    /// This call does not show a permission prompt. Request camera authorization through the
    /// framework's media-authorization capability before calling this method.
    pub fn new(_main_thread: MainThreadMarker) -> Result<Self, CameraCaptureError> {
        Self::new_for_camera_position(_main_thread, None)
    }

    /// Create a session for the already-authorized front or back built-in wide-angle camera
    ///
    /// This construction-time choice requires iOS 10.0 or later. It does not enumerate devices or
    /// show a permission prompt. An existing session can later switch between these positions via
    /// [`Self::set_camera_position`]. Request camera authorization through the framework's
    /// media-authorization capability before calling this method.
    ///
    /// Returns `CameraCaptureError::NoCameraAtPosition` when no built-in wide-angle device is
    /// available at the requested position, or `CameraCaptureError::UnsupportedOperatingSystem`
    /// below iOS 10.0.
    pub fn new_with_camera_position(
        _main_thread: MainThreadMarker,
        position: IosCameraPosition,
    ) -> Result<Self, CameraCaptureError> {
        if !objc2::available!(ios = 10.0, ..) {
            return Err(CameraCaptureError::UnsupportedOperatingSystem);
        }

        Self::new_for_camera_position(_main_thread, Some(position))
    }

    /// Switch this session to the requested default built-in wide-angle camera
    ///
    /// The synchronous change requires iOS 10 or later and uses the system's default front or
    /// back video device. It does not enumerate devices, request authorization, or alter preview,
    /// photo, movie, sample, or audio settings. An already selected position is a no-op. A movie
    /// remains active through file finalization, so switching is rejected until its terminal
    /// callback settles. Calls from this session's movie completion callback run inline on the
    /// capture queue to avoid synchronous queue reentrancy.
    pub fn set_camera_position(
        &self,
        position: IosCameraPosition,
    ) -> Result<(), CameraPositionSwitchError> {
        if !objc2::available!(ios = 10.0, ..) {
            return Err(CameraPositionSwitchError::UnsupportedOperatingSystem);
        }

        let graph = Arc::clone(&self.graph);
        let queue = self.queue.clone();
        run_on_capture_queue(
            is_current_capture_queue(&self.queue_key),
            move || {
                let mut graph = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                switch_camera_position_on_queue(&mut graph, position)
            },
            move |work| {
                let result = Mutex::new(None);
                queue.exec_sync(|| {
                    *result
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(work());
                });
                result
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .expect("capture queue returns a camera-position result")
            },
        )
    }

    fn new_for_camera_position(
        _main_thread: MainThreadMarker,
        camera_position: Option<IosCameraPosition>,
    ) -> Result<Self, CameraCaptureError> {
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
        let queue_key = Arc::new(AtomicBool::new(false));
        let queue_key_for_destruction = Arc::clone(&queue_key);
        queue.set_specific(capture_queue_specific_key(&queue_key), move || {
            drop(queue_key_for_destruction);
        });
        let video_callback_active = Arc::new(AtomicBool::new(false));
        let graph = Arc::new(Mutex::new(CaptureGraph {
            session,
            device: None,
            video_input: None,
            camera_position: None,
            photo_output: None,
            movie_output: None,
            video_output: None,
            video_delegate: None,
            video_callback_queue: None,
            video_callback_active: Arc::clone(&video_callback_active),
            video_stream_state: VideoFrameStreamState::default(),
            movie_audio_input: None,
            movie_recording: MovieRecordingState::default(),
        }));

        let mut configuration = Ok(());
        let graph_for_configuration = Arc::clone(&graph);
        queue.exec_sync(|| {
            let mut graph = graph_for_configuration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            configuration = configure_session(&mut graph, camera_position);
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
            queue_key,
            video_callback_active,
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

    /// Enable or disable the torch on this session's already selected camera
    ///
    /// The operation is serialized with session work and locks the device only for the mode
    /// update. It does not select or switch cameras, request authorization, or change session or
    /// audio policy. Disabling is allowed when AVFoundation reports the torch temporarily
    /// unavailable; enabling returns `TorchControlError::TorchUnavailable` in that state.
    pub fn set_torch_enabled(&self, enabled: bool) -> Result<(), TorchControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(TorchControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(device) = graph.device.as_ref() else {
                return;
            };
            result = configure_torch_enabled(
                enabled,
                // SAFETY: This is the retained AVCaptureDevice used by the configured input; the
                // package's iOS 10.0 and Simulator 14.0 floors exceed the torch API's iOS 4.0 floor.
                || unsafe { device.hasTorch() },
                // SAFETY: The public mode-support query is available on this selected device.
                |enabled| unsafe {
                    device.isTorchModeSupported(if enabled {
                        AVCaptureTorchMode::On
                    } else {
                        AVCaptureTorchMode::Off
                    })
                },
                // SAFETY: The iOS 10.0 device floor exceeds isTorchAvailable's iOS 5.0 floor. The
                // helper calls this only when enabling, so disabling remains possible if unavailable.
                || unsafe { device.isTorchAvailable() },
                // SAFETY: lockForConfiguration is public and the selected device remains retained
                // under the graph mutex for this serialized operation.
                || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                // SAFETY: The mode was checked above and the device configuration lock was acquired.
                |enabled| unsafe {
                    device.setTorchMode(if enabled {
                        AVCaptureTorchMode::On
                    } else {
                        AVCaptureTorchMode::Off
                    })
                },
                // SAFETY: configure_torch_enabled invokes this only after a successful matching lock.
                || unsafe { device.unlockForConfiguration() },
            );
        });
        result
    }

    /// Set an exact zoom factor on the session's already selected camera
    ///
    /// The finite factor must be within the active format's inclusive range from `1.0` through
    /// `videoMaxZoomFactor`. This operation runs on the session queue, acquires the device
    /// configuration lock, and returns an error instead of clamping an unsupported value. It does
    /// not enumerate or explicitly switch devices, request authorization, or change session or
    /// audio policy. AVFoundation applies the factor immediately; this method does not ramp zoom.
    pub fn set_zoom_factor(&self, factor: f64) -> Result<(), ZoomControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(ZoomControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Ok(device) = require_selected_camera(graph.device.as_ref()) else {
                return;
            };
            result = configure_video_zoom_factor(
                factor,
                // SAFETY: The selected device remains retained under the graph mutex, its
                // configuration lock is held, and the iOS 10.0 floor exceeds this iOS 7.0 API.
                || {
                    let active_format = unsafe { device.activeFormat() };
                    unsafe { active_format.videoMaxZoomFactor() }
                },
                // SAFETY: The selected device remains retained under the graph mutex for this
                // serialized operation; lockForConfiguration is public AVFoundation API.
                || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                // SAFETY: The validated factor lies within the active format's public range and
                // the matching device configuration lock is held.
                |factor| unsafe { device.setVideoZoomFactor(factor) },
                // SAFETY: configure_video_zoom_factor invokes this only after a successful lock.
                || unsafe { device.unlockForConfiguration() },
            );
        });
        result
    }

    /// Set the selected camera's focus mode and optional normalized point of interest
    ///
    /// Coordinates use AVFoundation's image coordinate system: `(0.0, 0.0)` is the top-left
    /// and `(1.0, 1.0)` is the bottom-right. They are not UIKit preview-layer coordinates; callers
    /// must convert points from a rotated or mirrored preview before passing them. `None` leaves
    /// the native point unchanged. A point is accepted only with `Auto` or `ContinuousAuto`, since
    /// `Locked` holds the current lens position and a point alone does not start a focus operation.
    /// The request is checked against the selected camera before its configuration lock is taken,
    /// then the point (if any) and mode are set under that lock. This does not prompt, switch
    /// cameras, or alter capture-session or audio policy.
    pub fn set_focus(
        &self,
        mode: IosCameraFocusMode,
        point: Option<NormalizedFocusPoint>,
    ) -> Result<(), FocusControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(FocusControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Ok(device) = require_selected_focus_camera(graph.device.as_ref()) else {
                return;
            };
            let native_mode = |mode| match mode {
                IosCameraFocusMode::Auto => AVCaptureFocusMode::AutoFocus,
                IosCameraFocusMode::ContinuousAuto => AVCaptureFocusMode::ContinuousAutoFocus,
                IosCameraFocusMode::Locked => AVCaptureFocusMode::Locked,
            };
            result = configure_focus(
                mode,
                point,
                // SAFETY: These public focus modes are available from iOS 4.0, below the
                // package's iOS 10.0 device and iOS 14.0 Simulator floors. The optional point
                // support query is public and runs only after the requested mode is supported.
                |mode, point| {
                    if !unsafe { device.isFocusModeSupported(native_mode(mode)) } {
                        return Err(FocusControlError::FocusModeUnsupported);
                    }
                    if point.is_some() && !unsafe { device.isFocusPointOfInterestSupported() } {
                        return Err(FocusControlError::FocusPointUnsupported);
                    }
                    Ok(())
                },
                // SAFETY: The retained selected device remains protected by the graph mutex for
                // this serialized operation; lockForConfiguration is public AVFoundation API.
                || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                // SAFETY: The point was validated and is set only after support and lock checks.
                |point| unsafe {
                    device.setFocusPointOfInterest(CGPoint::new(point.x(), point.y()))
                },
                // SAFETY: The requested mode was checked before locking, and the lock is held.
                |mode| unsafe { device.setFocusMode(native_mode(mode)) },
                // SAFETY: configure_focus invokes this only after a successful matching lock.
                || unsafe { device.unlockForConfiguration() },
            );
        });
        result
    }

    /// Query whether the selected camera supports a white-balance mode
    ///
    /// This read-only query runs on the capture queue and does not acquire the device
    /// configuration lock. A camera mode may be unavailable on a particular device. Calls from
    /// this session's capture queue run inline to avoid synchronous queue reentrancy.
    pub fn is_white_balance_mode_supported(
        &self,
        mode: IosCameraWhiteBalanceMode,
    ) -> Result<bool, WhiteBalanceControlError> {
        let graph = Arc::clone(&self.graph);
        self.run_capture_queue_sync(move || {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            with_selected_white_balance_camera(graph.device.as_ref(), |device| {
                // SAFETY: This is a public AVFoundation query on the retained selected device;
                // the package's iOS 10.0 and Simulator 14.0 floors exceed its iOS 4.0 floor.
                Ok(unsafe { device.isWhiteBalanceModeSupported(native_white_balance_mode(mode)) })
            })
        })
    }

    /// Set white balance on the session's already selected camera
    ///
    /// `Locked` holds the current white balance, `AutoWhiteBalance` adjusts once then locks, and
    /// `ContinuousAutoWhiteBalance` continues automatic adjustment. Support is checked before the
    /// device configuration lock; the mode is set under that lock on the serialized capture queue.
    /// This does not set manual gains or temperature/tint, prompt, switch cameras, or alter session
    /// or audio policy. Calls from this session's capture queue run inline to avoid synchronous
    /// queue reentrancy.
    pub fn set_white_balance_mode(
        &self,
        mode: IosCameraWhiteBalanceMode,
    ) -> Result<(), WhiteBalanceControlError> {
        let graph = Arc::clone(&self.graph);
        self.run_capture_queue_sync(move || {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            with_selected_white_balance_camera(graph.device.as_ref(), |device| {
                configure_white_balance_mode(
                    mode,
                    // SAFETY: The public support query is available from iOS 4.0 and runs on the
                    // retained selected device before its configuration lock is acquired.
                    |mode| unsafe {
                        device.isWhiteBalanceModeSupported(native_white_balance_mode(mode))
                    },
                    // SAFETY: The selected device remains retained under the graph mutex for
                    // this serialized operation; lockForConfiguration is public AVFoundation API.
                    || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                    // SAFETY: The exact requested mode was checked above and the lock is held.
                    |mode| unsafe { device.setWhiteBalanceMode(native_white_balance_mode(mode)) },
                    // SAFETY: configure_white_balance_mode invokes this only after a successful
                    // matching lock.
                    || unsafe { device.unlockForConfiguration() },
                )
            })
        })
    }

    /// Set the selected camera's exposure mode and optional normalized point of interest
    ///
    /// Exposure points use AVFoundation's image coordinate system and reuse
    /// `NormalizedFocusPoint`; they are not UIKit preview-layer coordinates. `None` preserves the
    /// native point. Points are accepted only with automatic exposure modes, are checked against
    /// the selected camera before locking, and are set before the explicitly requested supported
    /// mode. `Locked` holds the current exposure and rejects a point. This does not expose custom
    /// ISO/duration controls, prompt, switch cameras, or alter capture-session or audio policy.
    pub fn set_exposure(
        &self,
        mode: IosCameraExposureMode,
        point: Option<NormalizedFocusPoint>,
    ) -> Result<(), ExposureControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(ExposureControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            result = with_selected_exposure_camera(graph.device.as_ref(), |device| {
                let native_mode = |mode| match mode {
                    IosCameraExposureMode::AutoExpose => AVCaptureExposureMode::AutoExpose,
                    IosCameraExposureMode::ContinuousAutoExposure => {
                        AVCaptureExposureMode::ContinuousAutoExposure
                    }
                    IosCameraExposureMode::Locked => AVCaptureExposureMode::Locked,
                };
                configure_exposure(
                    mode,
                    point,
                    // SAFETY: These public exposure APIs are available from iOS 4.0, below the
                    // package's iOS 10.0 device and iOS 14.0 Simulator floors.
                    |mode, point| {
                        if !unsafe { device.isExposureModeSupported(native_mode(mode)) } {
                            return Err(ExposureControlError::ExposureModeUnsupported);
                        }
                        if point.is_some()
                            && !unsafe { device.isExposurePointOfInterestSupported() }
                        {
                            return Err(ExposureControlError::ExposurePointUnsupported);
                        }
                        Ok(())
                    },
                    // SAFETY: The retained selected device remains protected by the graph mutex
                    // for this serialized operation; lockForConfiguration is public AVFoundation API.
                    || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                    // SAFETY: The normalized point was validated and point support checked before
                    // the selected device's configuration lock was acquired.
                    |point| unsafe {
                        device.setExposurePointOfInterest(CGPoint::new(point.x(), point.y()))
                    },
                    // SAFETY: The requested public mode was checked before locking, and the lock is held.
                    |mode| unsafe { device.setExposureMode(native_mode(mode)) },
                    // SAFETY: configure_exposure invokes this only after a successful matching lock.
                    || unsafe { device.unlockForConfiguration() },
                )
            });
        });
        result
    }

    /// Read the current active format's inclusive custom exposure duration and ISO bounds
    ///
    /// The returned `MediaTime` durations preserve AVFoundation's value and timescale exactly.
    /// This is a point-in-time snapshot; `set_custom_exposure` re-reads the active format while
    /// holding the device configuration lock. The query does not change exposure mode or settings.
    pub fn custom_exposure_range(&self) -> Result<IosCustomExposureRange, ExposureControlError> {
        let graph = Arc::clone(&self.graph);
        self.run_capture_queue_sync(move || {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            with_selected_exposure_camera(graph.device.as_ref(), |device| {
                native_custom_exposure_range(device)
            })
        })
    }

    /// Request one exact custom exposure duration and ISO on the selected camera
    ///
    /// Duration is a strictly positive exact rational number of seconds and ISO is a finite
    /// positive value. AVFoundation's custom-mode support is checked before the device lock; the
    /// active format's inclusive bounds are then re-read under that lock. Invalid native ranges
    /// and out-of-range requests return errors without clamping or mutation. A successful call
    /// sends a nil completion handler and means only that AVFoundation accepted the request; it
    /// does not prove that a captured frame used the values. This does not prompt, select a camera,
    /// change session policy, or alter still-photo quality prioritization.
    pub fn set_custom_exposure(
        &self,
        duration: MediaTime,
        iso: f32,
    ) -> Result<(), ExposureControlError> {
        let graph = Arc::clone(&self.graph);
        self.run_capture_queue_sync(move || {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            with_selected_exposure_camera(graph.device.as_ref(), |device| {
                configure_custom_exposure(
                    duration,
                    iso,
                    // SAFETY: This public exposure-mode support query precedes the device lock.
                    || unsafe { device.isExposureModeSupported(AVCaptureExposureMode::Custom) },
                    // SAFETY: The retained selected device is protected by the graph mutex and
                    // this operation runs on the session's serial capture queue.
                    || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                    // SAFETY: Called only after a successful device lock; activeFormat and its
                    // duration/ISO limits are public AVFoundation APIs.
                    || native_custom_exposure_range(device),
                    // SAFETY: The exact request was validated against the locked active format.
                    // AVFoundation explicitly permits nil when completion knowledge is not needed.
                    |duration, iso| unsafe {
                        device.setExposureModeCustomWithDuration_ISO_completionHandler(
                            native_custom_exposure_duration(duration),
                            iso,
                            None,
                        )
                    },
                    // SAFETY: configure_custom_exposure calls this after one successful lock.
                    || unsafe { device.unlockForConfiguration() },
                )
            })
        })
    }

    /// Read the selected camera's supported exposure target-bias range in EV
    ///
    /// The returned values are a read-only snapshot; device formats or availability may change
    /// later. This does not acquire a configuration lock or claim that a future bias request will
    /// be applied to a particular frame.
    pub fn exposure_target_bias_range(
        &self,
    ) -> Result<IosExposureTargetBiasRange, ExposureControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(ExposureControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            result = with_selected_exposure_camera(graph.device.as_ref(), |device| {
                // SAFETY: These public read-only bias bounds are available from iOS 8.0, below
                // both package deployment floors; the selected device remains retained under the graph lock.
                let minimum = unsafe { device.minExposureTargetBias() };
                // SAFETY: Same retained-device and public-API conditions as the minimum above.
                let maximum = unsafe { device.maxExposureTargetBias() };
                exposure_target_bias_range(minimum, maximum)
            });
        });
        result
    }

    /// Set an exact exposure target bias in EV on the selected camera
    ///
    /// The finite value must be within the device's inclusive `minExposureTargetBias` through
    /// `maxExposureTargetBias` range. The bounds are read under the configuration lock; unsupported
    /// values are rejected without clamping. `Ok(())` means AVFoundation accepted the request, not
    /// that a captured frame has used it. No completion callback or custom exposure is exposed.
    pub fn set_exposure_target_bias(&self, bias_ev: f32) -> Result<(), ExposureControlError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(ExposureControlError::NoSelectedCamera);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            result = with_selected_exposure_camera(graph.device.as_ref(), |device| {
                configure_exposure_target_bias(
                    bias_ev,
                    // SAFETY: The public read-only bounds are available from iOS 8.0. The helper
                    // reads them only after lockForConfiguration succeeds.
                    || unsafe {
                        (
                            device.minExposureTargetBias(),
                            device.maxExposureTargetBias(),
                        )
                    },
                    // SAFETY: The retained selected device remains protected by the graph mutex
                    // for this serialized operation; lockForConfiguration is public AVFoundation API.
                    || unsafe { device.lockForConfiguration().map_err(|_| ()) },
                    // SAFETY: The exact finite bias was checked against native bounds while
                    // locked; the nil completion handler is explicitly supported by the public API.
                    |bias| unsafe { device.setExposureTargetBias_completionHandler(bias, None) },
                    // SAFETY: configure_exposure_target_bias invokes this only after a successful lock.
                    || unsafe { device.unlockForConfiguration() },
                )
            });
        });
        result
    }

    /// Set the caller-selected orientation for subsequent movie-file recordings
    ///
    /// This changes only the video connection of the configured movie-file output; it does not
    /// rotate the preview layer, still photos, or video sample stream. The current AVFoundation
    /// default remains in effect until this method succeeds. The request runs synchronously on
    /// the private capture queue and fails if a movie is active or finalizing, session shutdown is
    /// pending, the output/connection is unavailable, or AVFoundation does not support the value.
    /// The value applies to both video-only and audio-enabled movies. A call ordered before a
    /// movie start affects that movie; a call ordered after its start is rejected until terminal
    /// finalization. This method does not inspect device or UI orientation.
    pub fn set_movie_orientation(
        &self,
        orientation: IosCameraVideoOrientation,
    ) -> Result<(), MovieOrientationError> {
        let graph = Arc::clone(&self.graph);
        let mut result = Err(MovieOrientationError::MovieOutputUnavailable);
        self.queue.exec_sync(|| {
            let graph = graph
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            result = configure_movie_orientation(
                &graph.movie_recording,
                graph.movie_output.as_deref(),
                orientation,
                |output| {
                    let video_media_type = unsafe { AVMediaTypeVideo }
                        .ok_or(MovieOrientationError::VideoMediaTypeUnavailable)?;
                    // SAFETY: This retained output belongs to the configured graph. The connection
                    // is selected by the public video media type and retained for this queue task.
                    Ok(unsafe { output.connectionWithMediaType(video_media_type) })
                },
                |connection, orientation| {
                    configure_avfoundation_movie_orientation(connection, orientation)
                },
            );
        });
        result
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
    /// camera session, does not request permission or attach microphone input, and does not write to Photos.
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
        self.start_movie_recording(destination, false, completion)
    }

    /// Record video and microphone audio to a new QuickTime movie file
    ///
    /// This explicit opt-in requires current `Authorized` microphone status and an input-capable
    /// audio session already configured and activated by the host. It never requests permission
    /// or calls or changes host-owned audio-session state. AVFoundation's automatic configuration of the
    /// shared application audio session is disabled on this capture session. The microphone input
    /// is attached only for this movie request and is removed on the private serial queue at
    /// terminal settlement, before completion or any deferred session stop. It is absent during
    /// preview and idle. All destination, completion, and failure semantics match
    /// [`Self::start_video_recording`].
    pub fn start_video_recording_with_audio<F>(
        &self,
        destination: impl AsRef<Path>,
        completion: F,
    ) -> Result<(), VideoRecordingError>
    where
        F: FnOnce(Result<PathBuf, VideoRecordingError>) + Send + 'static,
    {
        self.start_movie_recording(destination, true, completion)
    }

    fn start_movie_recording<F>(
        &self,
        destination: impl AsRef<Path>,
        with_audio: bool,
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
            let mut recording_claimed = false;
            let mut delegate_for_unwind = None;
            let result = catch_unwind(AssertUnwindSafe(|| {
                let setup = {
                    let mut graph_state = graph
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    (|| {
                        if graph_state.movie_recording.pending_session_stop {
                            return Err(VideoRecordingError::SessionStopping);
                        }
                        if graph_state.movie_recording.active {
                            return Err(VideoRecordingError::RecordingAlreadyActive);
                        }
                        // SAFETY: Session state is queried on its private serial queue.
                        if !unsafe { graph_state.session.isRunning() } {
                            return Err(VideoRecordingError::SessionNotRunning);
                        }
                        let output = graph_state
                            .movie_output
                            .clone()
                            .ok_or(VideoRecordingError::MovieOutputUnavailable)?;
                        graph_state.movie_recording.begin()?;
                        recording_claimed = true;
                        prepare_optional_movie_audio(with_audio, || {
                            configure_movie_audio_input(&mut graph_state)
                        })?;
                        Ok(output)
                    })()
                };
                let output = match setup {
                    Ok(output) => output,
                    Err(error) => {
                        let stop_session = if recording_claimed {
                            let mut graph_state = graph
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            settle_movie_audio_on_queue(&mut graph_state)
                        } else {
                            false
                        };
                        invoke_video_completion(
                            completion.take().expect("completion is available"),
                            Err(error),
                        );
                        if stop_session {
                            let graph_state = graph
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            // SAFETY: Failed setup reached terminal state on the private queue.
                            unsafe { graph_state.session.stopRunning() };
                        }
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
                delegate_for_unwind = Some(delegate.clone());
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
                if let Some(delegate) = delegate_for_unwind {
                    delegate.finish_after_panic();
                } else if recording_claimed {
                    let stop_session = {
                        let mut graph_state = graph
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        settle_movie_audio_on_queue(&mut graph_state)
                    };
                    if let Some(completion) = completion.take() {
                        invoke_video_completion(
                            completion,
                            Err(VideoRecordingError::RecordingFailed),
                        );
                    }
                    if stop_session {
                        let graph_state = graph
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        // SAFETY: The catch is on the private queue and terminal state now owns
                        // session shutdown if Drop/stop was queued during setup.
                        unsafe { graph_state.session.stopRunning() };
                    }
                } else if let Some(completion) = completion.take() {
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

    /// Start one borrowed video sample callback stream from the default camera
    ///
    /// Call [`Self::start`] first. Session-queue ordering runs its start request first; this call
    /// returns `SessionNotRunning` if AVFoundation is not running when checked. It may wait for
    /// earlier session-queue work while it adds the output. The callback runs on a dedicated serial
    /// queue, not main, and receives a `CMSampleBuffer` borrowed only until that callback returns.
    /// Keep callback work short: AVFoundation discards late frames, and this API does not impose a
    /// runtime limit on user callback time. The framework does not retain or copy samples. A callback
    /// that retains a sample is responsible for its memory pressure. Only one stream may start per
    /// session; explicit stop is nonblocking and terminal, including when called before stream
    /// start. A callback already claimed may finish after stop. Session stop/drop also disables
    /// active delivery. A successful return means the output and delegate were installed, not that
    /// AVFoundation delivers frames. This operation does not configure audio or write Photos.
    /// Compilation and linking do not prove that a device delivers frames.
    pub fn start_video_frame_stream<F>(&self, callback: F) -> Result<(), VideoFrameStreamError>
    where
        F: FnMut(&CMSampleBuffer) + Send + 'static,
    {
        let graph = Arc::clone(&self.graph);
        let active = Arc::clone(&self.video_callback_active);
        let queue = self.queue.clone();
        let outcome = Arc::new(Mutex::new(None));
        let outcome_for_queue = Arc::clone(&outcome);
        self.queue.exec_sync(move || {
            let mut callback = Some(Box::new(callback) as VideoFrameCallback);
            let result = start_video_frame_stream_on_queue(&queue, &graph, &active, &mut callback);
            drop_video_frame_callback_safely(callback.take());
            *outcome_for_queue
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(result);
        });
        outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("session queue returns a video stream start result")
    }

    /// Request nonblocking, terminal shutdown of the video sample callback stream
    ///
    /// This method returns before AVFoundation detaches the delegate. A callback already claimed may
    /// finish; queued frames observed after deactivation are discarded. The framework drains the
    /// dedicated serial queue before it releases the delegate and output. A stopped stream cannot
    /// restart on this session. Calling stop before start also closes the stream to future starts.
    pub fn stop_video_frame_stream(&self) {
        self.video_callback_active.store(false, Ordering::Release);
        enqueue_video_frame_stream_stop(&self.queue, &self.graph);
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
fn start_video_frame_stream_on_queue(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
    active: &Arc<AtomicBool>,
    callback: &mut Option<VideoFrameCallback>,
) -> Result<(), VideoFrameStreamError> {
    let mut graph_state = graph
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if graph_state.movie_recording.pending_session_stop {
        return Err(VideoFrameStreamError::SessionStopping);
    }
    match graph_state.video_stream_state {
        VideoFrameStreamState::NotStarted => {}
        VideoFrameStreamState::Active => return Err(VideoFrameStreamError::AlreadyActive),
        VideoFrameStreamState::StopRequested | VideoFrameStreamState::Stopped => {
            return Err(VideoFrameStreamError::AlreadyStopped);
        }
    }
    // SAFETY: `isRunning` is a thread-safe AVFoundation session status query; this helper runs on
    // the session's serial queue after all earlier start/stop requests.
    if !unsafe { graph_state.session.isRunning() } {
        return Err(VideoFrameStreamError::SessionNotRunning);
    }

    if graph_state.video_output.is_none() {
        // SAFETY: AVCaptureVideoDataOutput is public AVFoundation API and the package's lowest
        // linked target is newer than the output's iOS 4 availability floor.
        let output = unsafe { AVCaptureVideoDataOutput::new() };
        // SAFETY: This boolean property has no ownership or thread-affinity requirements.
        unsafe { output.setAlwaysDiscardsLateVideoFrames(true) };
        // SAFETY: Session configuration begins and commits on the private serial session queue.
        unsafe { graph_state.session.beginConfiguration() };
        let accepted = unsafe { graph_state.session.canAddOutput(&output) };
        if accepted {
            // SAFETY: The immediately preceding canAddOutput accepted this output.
            unsafe { graph_state.session.addOutput(&output) };
        }
        // SAFETY: Every beginConfiguration call is paired with this commit, including rejection.
        unsafe { graph_state.session.commitConfiguration() };
        if !accepted {
            return Err(VideoFrameStreamError::OutputRejected);
        }
        graph_state.video_output = Some(output);
    }

    let callback_queue = graph_state
        .video_callback_queue
        .get_or_insert_with(|| {
            DispatchQueue::new(
                "com.jcdigitalsolutions.ios-camera-video-samples",
                DispatchQueueAttr::SERIAL,
            )
        })
        .clone();
    let output = graph_state
        .video_output
        .as_ref()
        .expect("accepted video output remains in the capture graph")
        .clone();
    graph_state.video_stream_state.begin()?;
    let callback = callback
        .take()
        .expect("a new video stream owns its callback before delegate setup");
    let delegate = VideoFrameDelegateObject::new(
        callback,
        Arc::clone(active),
        Arc::downgrade(graph),
        queue.clone(),
    );
    let protocol_delegate = ProtocolObject::from_ref(&*delegate);
    graph_state.video_delegate = Some(delegate.clone());
    graph_state
        .video_callback_active
        .store(true, Ordering::Release);
    // SAFETY: The output is part of this session, the delegate is retained by the graph before
    // registration allows callbacks, and the dedicated queue is serial and lives in this graph.
    unsafe {
        output.setSampleBufferDelegate_queue(Some(protocol_delegate), Some(&callback_queue));
    }
    Ok(())
}

#[cfg(target_os = "ios")]
fn enqueue_video_frame_stream_stop(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
) {
    let graph = Arc::clone(graph);
    let queue_for_task = queue.clone();
    queue.exec_async(move || {
        let mut graph_state = graph
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        graph_state
            .video_callback_active
            .store(false, Ordering::Release);
        if graph_state.video_stream_state.close_unstarted() {
            return;
        }
        stop_video_frame_stream_on_queue(&queue_for_task, &graph, &mut graph_state);
    });
}

#[cfg(target_os = "ios")]
fn enqueue_video_frame_stream_stop_after_callback(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: std::sync::Weak<Mutex<CaptureGraph>>,
) {
    let queue_for_task = queue.clone();
    queue.exec_async(move || {
        let Some(graph) = graph.upgrade() else {
            return;
        };
        let mut graph_state = graph
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        graph_state
            .video_callback_active
            .store(false, Ordering::Release);
        stop_video_frame_stream_on_queue(&queue_for_task, &graph, &mut graph_state);
    });
}

#[cfg(target_os = "ios")]
fn stop_video_frame_stream_on_queue(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
    graph_state: &mut CaptureGraph,
) {
    if !graph_state.video_stream_state.request_stop() {
        return;
    }
    if let Some(output) = graph_state.video_output.as_ref() {
        // SAFETY: Delegate detachment is serialized with session configuration and does not wait
        // for user callback work. The callback queue remains retained through its barrier below.
        unsafe { output.setSampleBufferDelegate_queue(None, None) };
    }
    let callback_queue = graph_state
        .video_callback_queue
        .clone()
        .expect("an active video stream retains its callback queue");
    let callback_queue_for_barrier = callback_queue.clone();
    let callback_queue_for_release = callback_queue.clone();
    let graph = Arc::clone(graph);
    let session_queue = queue.clone();
    let session_queue_for_callback_drop = session_queue.clone();
    callback_queue.exec_async(move || {
        let _callback_queue = callback_queue_for_barrier;
        session_queue.exec_async(move || {
            let stopped = {
                let mut graph = graph
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if graph.video_stream_state.finish_stop() {
                    if let Some(output) = graph.video_output.as_ref() {
                        // SAFETY: Session mutations use this serial queue, and the callback-queue
                        // barrier proves no delegate callback remains active.
                        unsafe { graph.session.beginConfiguration() };
                        // SAFETY: This output was previously accepted by `canAddOutput`.
                        unsafe { graph.session.removeOutput(output) };
                        // SAFETY: Every beginConfiguration call is paired with this commit.
                        unsafe { graph.session.commitConfiguration() };
                    }
                    true
                } else {
                    false
                }
            };
            if !stopped {
                return;
            }
            let graph_for_callback_drop = Arc::clone(&graph);
            let session_queue_for_release = session_queue_for_callback_drop;
            callback_queue_for_release.exec_async(move || {
                let delegate = {
                    graph_for_callback_drop
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .video_delegate
                        .clone()
                };
                if let Some(delegate) = delegate {
                    delegate.take_and_drop_callback();
                }
                session_queue_for_release.exec_async(move || {
                    let retired = {
                        let mut graph = graph_for_callback_drop
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        (
                            graph.video_delegate.take(),
                            graph.video_output.take(),
                            graph.video_callback_queue.take(),
                        )
                    };
                    drop(retired);
                });
            });
        });
    });
}

#[cfg(target_os = "ios")]
fn configure_session(
    graph: &mut CaptureGraph,
    camera_position: Option<IosCameraPosition>,
) -> Result<(), CameraCaptureError> {
    // SAFETY: This is AVFoundation's public video media-type constant. The constructor checks the
    // iOS 7.0 API floor before this function runs.
    let media_type =
        unsafe { AVMediaTypeVideo }.ok_or(CameraCaptureError::VideoMediaTypeUnavailable)?;
    let device = if let Some(position) = camera_position {
        let native_position = match position {
            IosCameraPosition::Front => AVCaptureDevicePosition::Front,
            IosCameraPosition::Back => AVCaptureDevicePosition::Back,
        };
        // SAFETY: The new constructor checks the iOS 10.0 availability floor. The wide-angle
        // type and requested front/back position are valid AVFoundation criteria, and the media
        // type is exactly AVMediaTypeVideo.
        let device = unsafe {
            AVCaptureDevice::defaultDeviceWithDeviceType_mediaType_position(
                AVCaptureDeviceTypeBuiltInWideAngleCamera,
                Some(media_type),
                native_position,
            )
        };
        require_camera_at_position(position, device)?
    } else {
        // SAFETY: The media type is exactly AVMediaTypeVideo, as required by this public API.
        unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) }
            .ok_or(CameraCaptureError::NoDefaultVideoDevice)?
    };
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
    graph.camera_position = camera_position_for_device(&device).or(camera_position);
    graph.device = Some(device);
    graph.video_input = Some(input);

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
fn switch_camera_position_on_queue(
    graph: &mut CaptureGraph,
    requested: IosCameraPosition,
) -> Result<(), CameraPositionSwitchError> {
    let should_switch = camera_position_switch_required(
        &graph.movie_recording,
        graph.camera_position,
        graph.video_input.is_some() && graph.device.is_some(),
        requested,
    )?;
    if !should_switch {
        return Ok(());
    }

    let old_input = graph
        .video_input
        .as_ref()
        .ok_or(CameraPositionSwitchError::NoSelectedCamera)?
        .clone();
    if graph.device.is_none() {
        return Err(CameraPositionSwitchError::NoSelectedCamera);
    }
    let media_type =
        unsafe { AVMediaTypeVideo }.ok_or(CameraPositionSwitchError::VideoMediaTypeUnavailable)?;
    let native_position = match requested {
        IosCameraPosition::Front => AVCaptureDevicePosition::Front,
        IosCameraPosition::Back => AVCaptureDevicePosition::Back,
    };
    // SAFETY: The public type/media/position selector is available from iOS 10, checked by the
    // caller; the request is restricted to the built-in wide-angle video device.
    let device = unsafe {
        AVCaptureDevice::defaultDeviceWithDeviceType_mediaType_position(
            AVCaptureDeviceTypeBuiltInWideAngleCamera,
            Some(media_type),
            native_position,
        )
    };
    let device = require_switch_camera(requested, device)?;
    // SAFETY: Existing authorization was checked at session construction; AVFoundation reports
    // any later device-open failure through this public initializer.
    let input = unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }
        .map_err(|_| CameraPositionSwitchError::DeviceInputCreationFailed)?;

    let session = &graph.session;
    let replacement = replace_camera_session_input(
        &old_input,
        input,
        |candidate| {
            // SAFETY: The candidate is a valid input for the requested public video device.
            unsafe { session.canAddInput(candidate) }
        },
        |old| {
            // SAFETY: This is the exact video input retained by the current graph.
            unsafe { session.removeInput(old) }
        },
        |candidate| {
            // SAFETY: The immediately preceding canAddInput call accepted this exact input.
            unsafe { session.addInput(candidate) }
        },
        || {
            // SAFETY: This is the single serialized session configuration transaction.
            unsafe { session.beginConfiguration() }
        },
        || {
            // SAFETY: Every beginConfiguration call is paired with this commit.
            unsafe { session.commitConfiguration() }
        },
    )
    .map(|new_input| (device, new_input));
    finish_camera_input_replacement(
        &mut graph.device,
        &mut graph.video_input,
        &mut graph.camera_position,
        requested,
        replacement,
    )
}

#[cfg(target_os = "ios")]
fn configure_movie_audio_input(graph: &mut CaptureGraph) -> Result<(), VideoRecordingError> {
    if graph.movie_audio_input.is_some() {
        return Err(VideoRecordingError::AudioInputAlreadyAttached);
    }
    let authorization = IosMediaAuthorization::authorization_status(CaptureMedia::Microphone);
    with_authorized_microphone(authorization, || {
        // SAFETY: AVMediaTypeAudio is a public AVFoundation media-type constant.
        let media_type =
            unsafe { AVMediaTypeAudio }.ok_or(VideoRecordingError::AudioMediaTypeUnavailable)?;
        // SAFETY: This query selects only the public default audio capture device; no enumeration
        // or device persistence is used.
        let device = unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) }
            .ok_or(VideoRecordingError::NoDefaultMicrophone)?;
        // SAFETY: The current authorization status was checked immediately before this factory;
        // no permission request is made.
        let input = unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }
            .map_err(|_| VideoRecordingError::AudioInputCreationFailed)?;
        // SAFETY: This changes only AVCaptureSession's public auto-configuration policy. It does
        // not access host audio-session state; the host owns category, mode, activation, and route.
        unsafe {
            graph
                .session
                .setAutomaticallyConfiguresApplicationAudioSession(false)
        };
        // SAFETY: Configuration and graph changes run on the private serial session queue.
        unsafe { graph.session.beginConfiguration() };
        let accepted = unsafe { graph.session.canAddInput(&input) };
        if accepted {
            // SAFETY: The immediately preceding canAddInput call accepted this exact input.
            unsafe { graph.session.addInput(&input) };
        }
        // SAFETY: Every beginConfiguration call is paired with this commit, including rejection.
        unsafe { graph.session.commitConfiguration() };
        if !accepted {
            return Err(VideoRecordingError::SessionRejectedAudioInput);
        }
        graph.movie_audio_input = Some(input);
        Ok(())
    })
}

#[cfg(target_os = "ios")]
fn enqueue_stop(
    queue: &dispatch2::DispatchRetained<DispatchQueue>,
    graph: &Arc<Mutex<CaptureGraph>>,
) {
    let graph_for_task = Arc::clone(graph);
    let queue_for_task = queue.clone();
    queue.exec_async(move || {
        let mut graph = graph_for_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        graph.video_callback_active.store(false, Ordering::Release);
        stop_video_frame_stream_on_queue(&queue_for_task, &graph_for_task, &mut graph);
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
        CameraCaptureError, CameraPositionSwitchError, ExposureControlError, FocusControlError,
        IosCameraExposureMode, IosCameraFocusMode, IosCameraPosition, IosCameraVideoOrientation,
        IosCameraWhiteBalanceMode, MediaAuthorizationStatus, MediaTime, MovieOrientationError,
        MovieRecordingState, NormalizedFocusPoint, PhotoCaptureError, PhotoCaptureResult,
        TorchControlError, VideoFrameStreamError, VideoFrameStreamState, VideoRecordingError,
        VideoRecordingResult, WhiteBalanceControlError, ZoomControlError,
        camera_position_switch_required, configure_custom_exposure, configure_exposure,
        configure_exposure_target_bias, configure_focus, configure_movie_orientation,
        configure_torch_enabled, configure_video_zoom_factor, configure_white_balance_mode,
        custom_exposure_duration_from_parts, custom_exposure_range,
        drop_video_frame_callback_safely, exposure_target_bias_range,
        finish_camera_input_replacement, invoke_video_frame_callback, map_camera_video_orientation,
        may_create_device_input, photo_request_error, prepare_optional_movie_audio,
        replace_camera_session_input, require_camera_at_position, require_selected_camera,
        require_selected_focus_camera, require_switch_camera, run_on_capture_queue,
        set_supported_movie_orientation, settle_movie_result, settle_movie_terminal,
        settle_photo_result, take_photo_completion, take_video_completion,
        validate_movie_destination, with_authorized_microphone, with_selected_exposure_camera,
        with_selected_white_balance_camera,
    };
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    fn camera_time(value: i64, timescale: i32) -> MediaTime {
        MediaTime::new(value, timescale).expect("test media time is valid")
    }

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
    fn microphone_input_factory_requires_authorized_status() {
        for status in [
            MediaAuthorizationStatus::NotDetermined,
            MediaAuthorizationStatus::Restricted,
            MediaAuthorizationStatus::Denied,
            MediaAuthorizationStatus::Unknown,
        ] {
            assert_eq!(
                with_authorized_microphone::<()>(status, || {
                    panic!("unauthorized microphone status must not create an input")
                }),
                Err(VideoRecordingError::MicrophoneAuthorizationNotGranted)
            );
        }
        assert_eq!(
            with_authorized_microphone(MediaAuthorizationStatus::Authorized, || Ok(23_u8)),
            Ok(23)
        );
    }

    #[test]
    fn video_only_movie_request_never_prepares_audio_input() {
        assert_eq!(
            prepare_optional_movie_audio(false, || {
                panic!("video-only movie recording must not attach audio")
            }),
            Ok(())
        );
        let calls = std::cell::Cell::new(0);
        assert_eq!(
            prepare_optional_movie_audio(true, || {
                calls.set(calls.get() + 1);
                Ok(())
            }),
            Ok(())
        );
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn movie_terminal_settlement_removes_audio_before_completion_and_deferred_stop() {
        let mut state = MovieRecordingState::default();
        assert_eq!(state.begin(), Ok(()));
        assert!(state.request_session_stop());
        let mut audio_input = Some(7_u8);
        let events = std::cell::RefCell::new(Vec::new());
        let stop_session = settle_movie_terminal(&mut state, &mut audio_input, |input| {
            assert_eq!(*input, 7);
            events.borrow_mut().push("remove-input");
        });
        assert!(audio_input.is_none());
        assert!(state.pending_session_stop);
        assert_eq!(
            camera_position_switch_required(
                &state,
                Some(IosCameraPosition::Back),
                true,
                IosCameraPosition::Front,
            ),
            Err(CameraPositionSwitchError::SessionStopping)
        );
        events.borrow_mut().push("completion");
        if stop_session {
            events.borrow_mut().push("stop-session");
            state.finish_session_stop();
        }
        assert_eq!(
            *events.borrow(),
            ["remove-input", "completion", "stop-session"]
        );
        assert!(!state.active);
        assert!(!state.pending_session_stop);
    }

    #[test]
    fn camera_video_orientation_maps_each_value_to_avfoundation_order() {
        use IosCameraVideoOrientation as Orientation;

        assert_eq!(
            map_camera_video_orientation(Orientation::Portrait, 1, 2, 3, 4),
            1
        );
        assert_eq!(
            map_camera_video_orientation(Orientation::PortraitUpsideDown, 1, 2, 3, 4),
            2
        );
        assert_eq!(
            map_camera_video_orientation(Orientation::LandscapeRight, 1, 2, 3, 4),
            3
        );
        assert_eq!(
            map_camera_video_orientation(Orientation::LandscapeLeft, 1, 2, 3, 4),
            4
        );
    }

    #[test]
    fn movie_orientation_rejects_missing_output_and_connection_without_mutation() {
        use IosCameraVideoOrientation as Orientation;

        let state = MovieRecordingState::default();
        let current = std::cell::Cell::new(Orientation::LandscapeLeft);
        assert_eq!(
            configure_movie_orientation(
                &state,
                None::<&u8>,
                Orientation::Portrait,
                |_| Ok(Some(1_u8)),
                |_, requested| {
                    current.set(requested);
                    Ok(())
                },
            ),
            Err(MovieOrientationError::MovieOutputUnavailable)
        );
        assert_eq!(current.get(), Orientation::LandscapeLeft);

        let output = 7_u8;
        assert_eq!(
            configure_movie_orientation(
                &state,
                Some(&output),
                Orientation::Portrait,
                |_| -> Result<Option<u8>, MovieOrientationError> {
                    Err(MovieOrientationError::VideoMediaTypeUnavailable)
                },
                |_, requested| {
                    current.set(requested);
                    Ok(())
                },
            ),
            Err(MovieOrientationError::VideoMediaTypeUnavailable)
        );
        assert_eq!(current.get(), Orientation::LandscapeLeft);

        assert_eq!(
            configure_movie_orientation(
                &state,
                Some(&output),
                Orientation::Portrait,
                |_| Ok(None::<u8>),
                |_, requested| {
                    current.set(requested);
                    Ok(())
                },
            ),
            Err(MovieOrientationError::VideoConnectionUnavailable)
        );
        assert_eq!(current.get(), Orientation::LandscapeLeft);
    }

    #[test]
    fn unsupported_movie_orientation_is_checked_before_set_and_preserves_prior_value() {
        use IosCameraVideoOrientation as Orientation;

        let current = std::cell::Cell::new(Orientation::PortraitUpsideDown);
        let support_checks = std::cell::Cell::new(0);
        let setter_calls = std::cell::Cell::new(0);
        assert_eq!(
            set_supported_movie_orientation(
                &(),
                Orientation::LandscapeRight,
                |_| {
                    support_checks.set(support_checks.get() + 1);
                    false
                },
                |_, requested| {
                    setter_calls.set(setter_calls.get() + 1);
                    current.set(requested);
                },
            ),
            Err(MovieOrientationError::OrientationUnsupported)
        );
        assert_eq!(support_checks.get(), 1);
        assert_eq!(setter_calls.get(), 0);
        assert_eq!(current.get(), Orientation::PortraitUpsideDown);
    }

    #[test]
    fn supported_movie_orientation_sets_the_exact_requested_value_once() {
        use IosCameraVideoOrientation as Orientation;

        let current = std::cell::Cell::new(Orientation::Portrait);
        let setter_calls = std::cell::Cell::new(0);
        assert_eq!(
            set_supported_movie_orientation(
                &(),
                Orientation::LandscapeLeft,
                |_| true,
                |_, requested| {
                    setter_calls.set(setter_calls.get() + 1);
                    current.set(requested);
                },
            ),
            Ok(())
        );
        assert_eq!(setter_calls.get(), 1);
        assert_eq!(current.get(), Orientation::LandscapeLeft);
    }

    #[test]
    fn movie_orientation_rejects_start_finalization_and_pending_shutdown_queue_states() {
        use IosCameraVideoOrientation as Orientation;

        let output = 7_u8;
        let mut state = MovieRecordingState::default();
        assert_eq!(state.begin(), Ok(()));
        let current = std::cell::Cell::new(Orientation::Portrait);
        let attempt = |state: &MovieRecordingState| {
            configure_movie_orientation(
                state,
                Some(&output),
                Orientation::LandscapeRight,
                |output| Ok(Some(*output)),
                |_, requested| {
                    current.set(requested);
                    Ok(())
                },
            )
        };

        assert_eq!(attempt(&state), Err(MovieOrientationError::RecordingActive));
        assert_eq!(current.get(), Orientation::Portrait);
        assert!(state.request_recording_stop());
        assert!(state.active);
        assert_eq!(attempt(&state), Err(MovieOrientationError::RecordingActive));
        assert_eq!(current.get(), Orientation::Portrait);
        assert!(state.request_session_stop());
        assert_eq!(attempt(&state), Err(MovieOrientationError::SessionStopping));
        assert_eq!(current.get(), Orientation::Portrait);
    }

    #[test]
    fn camera_position_selection_maps_front_and_back_or_reports_unavailable() {
        for position in [IosCameraPosition::Front, IosCameraPosition::Back] {
            assert_eq!(require_camera_at_position(position, Some(17_u8)), Ok(17));
            assert_eq!(
                require_camera_at_position::<u8>(position, None),
                Err(CameraCaptureError::NoCameraAtPosition(position))
            );
        }
    }

    #[test]
    fn camera_switch_preflight_skips_same_position_and_rejects_busy_or_missing_graph() {
        use IosCameraPosition::{Back, Front};

        let mut state = MovieRecordingState::default();
        assert_eq!(
            camera_position_switch_required(&state, Some(Back), true, Back),
            Ok(false)
        );
        assert_eq!(
            camera_position_switch_required(&state, Some(Back), true, Front),
            Ok(true)
        );
        assert_eq!(
            camera_position_switch_required(&state, None, false, Front),
            Err(CameraPositionSwitchError::NoSelectedCamera)
        );

        assert_eq!(state.begin(), Ok(()));
        assert!(state.request_recording_stop());
        assert!(state.active);
        assert_eq!(
            camera_position_switch_required(&state, Some(Back), true, Front),
            Err(CameraPositionSwitchError::MovieRecordingActive)
        );
        assert!(state.request_session_stop());
        assert_eq!(
            camera_position_switch_required(&state, Some(Back), true, Front),
            Err(CameraPositionSwitchError::SessionStopping)
        );
    }

    #[test]
    fn camera_switch_queue_reentrancy_runs_inline_and_dispatches_other_calls_once() {
        let work_calls = Arc::new(AtomicUsize::new(0));
        let sync_calls = Arc::new(AtomicUsize::new(0));
        let inline_result = run_on_capture_queue(
            true,
            {
                let work_calls = Arc::clone(&work_calls);
                move || {
                    work_calls.fetch_add(1, Ordering::SeqCst);
                    17
                }
            },
            {
                let sync_calls = Arc::clone(&sync_calls);
                move |work| {
                    sync_calls.fetch_add(1, Ordering::SeqCst);
                    work()
                }
            },
        );
        assert_eq!(inline_result, 17);
        assert_eq!(work_calls.load(Ordering::SeqCst), 1);
        assert_eq!(sync_calls.load(Ordering::SeqCst), 0);

        let queued_result = run_on_capture_queue(
            false,
            {
                let work_calls = Arc::clone(&work_calls);
                move || {
                    work_calls.fetch_add(1, Ordering::SeqCst);
                    23
                }
            },
            {
                let sync_calls = Arc::clone(&sync_calls);
                move |work| {
                    sync_calls.fetch_add(1, Ordering::SeqCst);
                    work()
                }
            },
        );
        assert_eq!(queued_result, 23);
        assert_eq!(work_calls.load(Ordering::SeqCst), 2);
        assert_eq!(sync_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn camera_switch_rejects_missing_requested_device_without_mutation() {
        for position in [IosCameraPosition::Front, IosCameraPosition::Back] {
            assert_eq!(
                require_switch_camera::<u8>(position, None),
                Err(CameraPositionSwitchError::NoCameraAtPosition(position))
            );
        }
        assert_eq!(
            require_switch_camera(IosCameraPosition::Front, Some(31_u8)),
            Ok(31)
        );
    }

    #[test]
    fn camera_switch_replaces_inside_one_transaction_and_restores_rejected_input() {
        let events = std::cell::RefCell::new(Vec::new());
        assert_eq!(
            replace_camera_session_input(
                &1_u8,
                2_u8,
                |input| {
                    events.borrow_mut().push(if *input == 2 {
                        "check-new"
                    } else {
                        "check-old"
                    });
                    *input == 1
                },
                |input| {
                    assert_eq!(*input, 1);
                    events.borrow_mut().push("remove-old");
                },
                |input| events
                    .borrow_mut()
                    .push(if *input == 2 { "add-new" } else { "add-old" }),
                || events.borrow_mut().push("begin"),
                || events.borrow_mut().push("commit"),
            ),
            Err(CameraPositionSwitchError::SessionRejectedInput)
        );
        assert_eq!(
            *events.borrow(),
            [
                "begin",
                "remove-old",
                "check-new",
                "check-old",
                "add-old",
                "commit"
            ]
        );
    }

    #[test]
    fn camera_switch_adds_the_new_input_only_after_native_acceptance() {
        let events = std::cell::RefCell::new(Vec::new());
        assert_eq!(
            replace_camera_session_input(
                &1_u8,
                2_u8,
                |input| {
                    events.borrow_mut().push(if *input == 2 {
                        "check-new"
                    } else {
                        "check-old"
                    });
                    true
                },
                |input| {
                    assert_eq!(*input, 1);
                    events.borrow_mut().push("remove-old");
                },
                |input| {
                    assert_eq!(*input, 2);
                    events.borrow_mut().push("add-new");
                },
                || events.borrow_mut().push("begin"),
                || events.borrow_mut().push("commit"),
            ),
            Ok(2)
        );
        assert_eq!(
            *events.borrow(),
            ["begin", "remove-old", "check-new", "add-new", "commit"]
        );
    }

    #[test]
    fn camera_switch_reports_failed_rollback_and_clears_stale_selected_state() {
        let events = std::cell::RefCell::new(Vec::new());
        let replacement = replace_camera_session_input(
            &1_u8,
            2_u8,
            |input| {
                events.borrow_mut().push(if *input == 2 {
                    "check-new"
                } else {
                    "check-old"
                });
                false
            },
            |input| {
                assert_eq!(*input, 1);
                events.borrow_mut().push("remove-old");
            },
            |_| events.borrow_mut().push("unexpected-add"),
            || events.borrow_mut().push("begin"),
            || events.borrow_mut().push("commit"),
        );
        assert_eq!(
            replacement,
            Err(CameraPositionSwitchError::InputRollbackFailed)
        );
        assert_eq!(
            *events.borrow(),
            ["begin", "remove-old", "check-new", "check-old", "commit"]
        );

        let mut device = Some(3_u8);
        let mut input = Some(1_u8);
        let mut position = Some(IosCameraPosition::Back);
        assert_eq!(
            finish_camera_input_replacement(
                &mut device,
                &mut input,
                &mut position,
                IosCameraPosition::Front,
                Err(CameraPositionSwitchError::InputRollbackFailed),
            ),
            Err(CameraPositionSwitchError::InputRollbackFailed)
        );
        assert_eq!((device, input, position), (None, None, None));
    }

    #[test]
    fn camera_switch_commits_new_graph_state_only_after_success() {
        let mut device = Some(3_u8);
        let mut input = Some(1_u8);
        let mut position = Some(IosCameraPosition::Back);
        assert_eq!(
            finish_camera_input_replacement(
                &mut device,
                &mut input,
                &mut position,
                IosCameraPosition::Front,
                Ok((4_u8, 2_u8)),
            ),
            Ok(())
        );
        assert_eq!(
            (device, input, position),
            (Some(4), Some(2), Some(IosCameraPosition::Front))
        );

        assert_eq!(
            finish_camera_input_replacement(
                &mut device,
                &mut input,
                &mut position,
                IosCameraPosition::Back,
                Err(CameraPositionSwitchError::SessionRejectedInput),
            ),
            Err(CameraPositionSwitchError::SessionRejectedInput)
        );
        assert_eq!(
            (device, input, position),
            (Some(4), Some(2), Some(IosCameraPosition::Front))
        );
    }

    #[test]
    fn torch_control_rejects_devices_without_torch_or_requested_mode() {
        assert_eq!(
            configure_torch_enabled(
                true,
                || false,
                |_| panic!("mode support must not be queried without a torch"),
                || panic!("availability must not be queried without a torch"),
                || panic!("configuration must not lock without a torch"),
                |_| panic!("mode must not change without a torch"),
                || panic!("configuration must not unlock without a torch"),
            ),
            Err(TorchControlError::NoTorch)
        );

        assert_eq!(
            configure_torch_enabled(
                true,
                || true,
                |_| false,
                || panic!("availability is irrelevant for an unsupported mode"),
                || panic!("configuration must not lock for an unsupported mode"),
                |_| panic!("unsupported mode must not be set"),
                || panic!("configuration was never locked"),
            ),
            Err(TorchControlError::TorchModeUnsupported)
        );
    }

    #[test]
    fn torch_control_rejects_unavailable_enable_and_configuration_lock_failure() {
        assert_eq!(
            configure_torch_enabled(
                true,
                || true,
                |_| true,
                || false,
                || panic!("configuration must not lock while torch is unavailable"),
                |_| panic!("unavailable torch must not be enabled"),
                || panic!("configuration was never locked"),
            ),
            Err(TorchControlError::TorchUnavailable)
        );

        assert_eq!(
            configure_torch_enabled(
                false,
                || true,
                |_| true,
                || panic!("availability need not be queried to disable"),
                || Err(()),
                |_| panic!("mode must not change when locking fails"),
                || panic!("unlock is only valid after acquiring the lock"),
            ),
            Err(TorchControlError::ConfigurationLockFailed)
        );
    }

    #[test]
    fn torch_control_locks_sets_and_unlocks_and_allows_disable_when_unavailable() {
        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_torch_enabled(
                true,
                || {
                    events.borrow_mut().push("has-torch");
                    true
                },
                |enabled| {
                    assert!(enabled);
                    events.borrow_mut().push("supports-on");
                    true
                },
                || {
                    events.borrow_mut().push("available");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |enabled| {
                    assert!(enabled);
                    events.borrow_mut().push("set-on");
                },
                || events.borrow_mut().push("unlock"),
            ),
            Ok(())
        );
        assert_eq!(
            *events.borrow(),
            [
                "has-torch",
                "supports-on",
                "available",
                "lock",
                "set-on",
                "unlock"
            ]
        );

        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_torch_enabled(
                false,
                || true,
                |enabled| !enabled,
                || panic!("availability need not be queried to disable"),
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |enabled| events
                    .borrow_mut()
                    .push(if enabled { "set-on" } else { "set-off" }),
                || events.borrow_mut().push("unlock"),
            ),
            Ok(())
        );
        assert_eq!(*events.borrow(), ["lock", "set-off", "unlock"]);
    }

    #[test]
    fn torch_control_unlocks_configuration_during_unwind() {
        let events = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_torch_enabled(
                true,
                || true,
                |_| true,
                || true,
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| panic!("simulated mode-set panic"),
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["lock", "unlock"]);
    }

    #[test]
    fn zoom_control_requires_a_selected_camera() {
        assert_eq!(require_selected_camera(Some(17_u8)), Ok(17));
        assert_eq!(
            require_selected_camera::<u8>(None),
            Err(ZoomControlError::NoSelectedCamera)
        );
    }

    #[test]
    fn focus_control_requires_a_selected_camera() {
        assert_eq!(require_selected_focus_camera(Some(17_u8)), Ok(17));
        assert_eq!(
            require_selected_focus_camera::<u8>(None),
            Err(FocusControlError::NoSelectedCamera)
        );
    }

    #[test]
    fn focus_control_accepts_normalized_boundaries_and_rejects_invalid_points_before_native_calls()
    {
        for point in [
            NormalizedFocusPoint::new(0.0, 0.0),
            NormalizedFocusPoint::new(1.0, 1.0),
            NormalizedFocusPoint::new(0.0, 1.0),
            NormalizedFocusPoint::new(1.0, 0.0),
        ] {
            assert_eq!(
                configure_focus(
                    IosCameraFocusMode::Auto,
                    Some(point),
                    |_, _| Ok(()),
                    || Ok(()),
                    |_| {},
                    |_| {},
                    || {},
                ),
                Ok(())
            );
        }

        for point in [
            NormalizedFocusPoint::new(f64::NAN, 0.5),
            NormalizedFocusPoint::new(0.5, f64::INFINITY),
            NormalizedFocusPoint::new(f64::NEG_INFINITY, 0.5),
            NormalizedFocusPoint::new(-f64::EPSILON, 0.5),
            NormalizedFocusPoint::new(0.5, 1.0 + f64::EPSILON),
        ] {
            assert_eq!(
                configure_focus(
                    IosCameraFocusMode::Auto,
                    Some(point),
                    |_, _| panic!("invalid point must be rejected before support queries"),
                    || panic!("invalid point must not lock the device"),
                    |_| panic!("invalid point must not be set"),
                    |_| panic!("invalid point must not change mode"),
                    || panic!("device was not locked"),
                ),
                Err(FocusControlError::InvalidFocusPoint)
            );
        }
    }

    #[test]
    fn focus_control_rejects_locked_point_and_unsupported_mode_or_point_before_lock() {
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::Locked,
                Some(NormalizedFocusPoint::new(0.5, 0.5)),
                |_, _| panic!("locked focus point must be rejected before native checks"),
                || panic!("invalid combination must not lock the device"),
                |_| panic!("invalid combination must not set a point"),
                |_| panic!("invalid combination must not change mode"),
                || panic!("device was not locked"),
            ),
            Err(FocusControlError::LockedModeCannotUseFocusPoint)
        );
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::ContinuousAuto,
                None,
                |_, _| Err(FocusControlError::FocusModeUnsupported),
                || panic!("unsupported mode must not lock the device"),
                |_| panic!("unsupported mode must not set a point"),
                |_| panic!("unsupported mode must not be set"),
                || panic!("device was not locked"),
            ),
            Err(FocusControlError::FocusModeUnsupported)
        );
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::Auto,
                Some(NormalizedFocusPoint::new(0.5, 0.5)),
                |_, point| {
                    assert!(point.is_some());
                    Err(FocusControlError::FocusPointUnsupported)
                },
                || panic!("unsupported point must not lock the device"),
                |_| panic!("unsupported point must not be set"),
                |_| panic!("unsupported point must not change mode"),
                || panic!("device was not locked"),
            ),
            Err(FocusControlError::FocusPointUnsupported)
        );
    }

    #[test]
    fn focus_control_reports_lock_failure_and_sets_point_before_mode() {
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::Auto,
                None,
                |_, _| Ok(()),
                || Err(()),
                |_| panic!("device lock failed"),
                |_| panic!("device lock failed"),
                || panic!("device was not locked"),
            ),
            Err(FocusControlError::ConfigurationLockFailed)
        );

        let events = RefCell::new(Vec::new());
        let point = NormalizedFocusPoint::new(0.25, 0.75);
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::ContinuousAuto,
                Some(point),
                |mode, point| {
                    assert_eq!(mode, IosCameraFocusMode::ContinuousAuto);
                    events.borrow_mut().push("supports-mode");
                    if point.is_some() {
                        events.borrow_mut().push("supports-point");
                    }
                    Ok(())
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |requested| {
                    assert_eq!(requested, point);
                    events.borrow_mut().push("set-point");
                },
                |mode| {
                    assert_eq!(mode, IosCameraFocusMode::ContinuousAuto);
                    events.borrow_mut().push("set-mode");
                },
                || events.borrow_mut().push("unlock"),
            ),
            Ok(())
        );
        assert_eq!(
            *events.borrow(),
            [
                "supports-mode",
                "supports-point",
                "lock",
                "set-point",
                "set-mode",
                "unlock"
            ]
        );

        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_focus(
                IosCameraFocusMode::Locked,
                None,
                |_, point| {
                    assert!(point.is_none());
                    Ok(())
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| panic!("no point was requested"),
                |mode| {
                    assert_eq!(mode, IosCameraFocusMode::Locked);
                    events.borrow_mut().push("set-mode");
                },
                || events.borrow_mut().push("unlock"),
            ),
            Ok(())
        );
        assert_eq!(*events.borrow(), ["lock", "set-mode", "unlock"]);
    }

    #[test]
    fn focus_control_unlocks_after_unwind() {
        let events = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_focus(
                IosCameraFocusMode::Auto,
                Some(NormalizedFocusPoint::new(0.5, 0.5)),
                |_, _| Ok(()),
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| events.borrow_mut().push("set-point"),
                |_| panic!("simulated focus mode set panic"),
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["lock", "set-point", "unlock"]);
    }

    #[test]
    fn white_balance_control_requires_a_selected_camera() {
        assert_eq!(with_selected_white_balance_camera(Some(17_u8), Ok), Ok(17));
        assert_eq!(
            with_selected_white_balance_camera::<u8, ()>(None, |_| Ok(())),
            Err(WhiteBalanceControlError::NoSelectedCamera)
        );
    }

    #[test]
    fn white_balance_control_rejects_unsupported_mode_before_lock() {
        assert_eq!(
            configure_white_balance_mode(
                IosCameraWhiteBalanceMode::ContinuousAutoWhiteBalance,
                |mode| {
                    assert_eq!(mode, IosCameraWhiteBalanceMode::ContinuousAutoWhiteBalance);
                    false
                },
                || panic!("unsupported white-balance mode must not lock the device"),
                |_| panic!("unsupported white-balance mode must not be set"),
                || panic!("device was not locked"),
            ),
            Err(WhiteBalanceControlError::WhiteBalanceModeUnsupported)
        );
    }

    #[test]
    fn white_balance_control_reports_configuration_lock_failure() {
        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_white_balance_mode(
                IosCameraWhiteBalanceMode::Locked,
                |_| {
                    events.borrow_mut().push("supports");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Err(())
                },
                |_| panic!("failed device lock must not set white balance"),
                || panic!("failed device lock must not unlock"),
            ),
            Err(WhiteBalanceControlError::ConfigurationLockFailed)
        );
        assert_eq!(*events.borrow(), ["supports", "lock"]);
    }

    #[test]
    fn white_balance_control_checks_support_then_sets_and_unlocks() {
        for mode in [
            IosCameraWhiteBalanceMode::Locked,
            IosCameraWhiteBalanceMode::AutoWhiteBalance,
            IosCameraWhiteBalanceMode::ContinuousAutoWhiteBalance,
        ] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_white_balance_mode(
                    mode,
                    |requested| {
                        assert_eq!(requested, mode);
                        events.borrow_mut().push("supports");
                        true
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |requested| {
                        assert_eq!(requested, mode);
                        events.borrow_mut().push("set");
                    },
                    || events.borrow_mut().push("unlock"),
                ),
                Ok(())
            );
            assert_eq!(*events.borrow(), ["supports", "lock", "set", "unlock"]);
        }
    }

    #[test]
    fn white_balance_control_unlocks_after_unwind() {
        let events = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_white_balance_mode(
                IosCameraWhiteBalanceMode::AutoWhiteBalance,
                |_| {
                    events.borrow_mut().push("supports");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| {
                    events.borrow_mut().push("set");
                    panic!("simulated white-balance mode set panic")
                },
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["supports", "lock", "set", "unlock"]);
    }

    #[test]
    fn exposure_control_requires_a_selected_camera() {
        assert_eq!(with_selected_exposure_camera(Some(17_u8), Ok), Ok(17));
        assert_eq!(
            with_selected_exposure_camera::<u8, ()>(None, |_| {
                panic!("missing selected camera must not query or mutate native device state")
            }),
            Err(ExposureControlError::NoSelectedCamera)
        );
    }

    #[test]
    fn exposure_control_rejects_invalid_points_and_locked_points_before_native_calls() {
        for point in [
            NormalizedFocusPoint::new(f64::NAN, 0.5),
            NormalizedFocusPoint::new(0.5, f64::NAN),
            NormalizedFocusPoint::new(f64::INFINITY, 0.5),
            NormalizedFocusPoint::new(0.5, f64::NEG_INFINITY),
            NormalizedFocusPoint::new(-f64::EPSILON, 0.5),
            NormalizedFocusPoint::new(0.5, -f64::EPSILON),
            NormalizedFocusPoint::new(1.0 + f64::EPSILON, 0.5),
            NormalizedFocusPoint::new(0.5, 1.0 + f64::EPSILON),
        ] {
            assert_eq!(
                configure_exposure(
                    IosCameraExposureMode::AutoExpose,
                    Some(point),
                    |_, _| panic!("invalid point must be rejected before support queries"),
                    || panic!("invalid point must not lock the device"),
                    |_| panic!("invalid point must not be set"),
                    |_| panic!("invalid point must not change mode"),
                    || panic!("device was not locked"),
                ),
                Err(ExposureControlError::InvalidExposurePoint)
            );
        }
        assert_eq!(
            configure_exposure(
                IosCameraExposureMode::Locked,
                Some(NormalizedFocusPoint::new(0.5, 0.5)),
                |_, _| panic!("locked exposure point must be rejected before native checks"),
                || panic!("invalid combination must not lock the device"),
                |_| panic!("invalid combination must not set a point"),
                |_| panic!("invalid combination must not change mode"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::LockedModeCannotUseExposurePoint)
        );
    }

    #[test]
    fn exposure_control_rejects_unsupported_mode_or_point_before_lock() {
        assert_eq!(
            configure_exposure(
                IosCameraExposureMode::ContinuousAutoExposure,
                None,
                |mode, point| {
                    assert_eq!(mode, IosCameraExposureMode::ContinuousAutoExposure);
                    assert!(point.is_none());
                    Err(ExposureControlError::ExposureModeUnsupported)
                },
                || panic!("unsupported mode must not lock the device"),
                |_| panic!("unsupported mode must not set a point"),
                |_| panic!("unsupported mode must not change"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::ExposureModeUnsupported)
        );
        assert_eq!(
            configure_exposure(
                IosCameraExposureMode::AutoExpose,
                Some(NormalizedFocusPoint::new(0.25, 0.75)),
                |_, point| {
                    assert!(point.is_some());
                    Err(ExposureControlError::ExposurePointUnsupported)
                },
                || panic!("unsupported point must not lock the device"),
                |_| panic!("unsupported point must not be set"),
                |_| panic!("unsupported point must not change mode"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::ExposurePointUnsupported)
        );
    }

    #[test]
    fn exposure_control_sets_requested_point_before_exact_mode_and_none_preserves_point() {
        for mode in [
            IosCameraExposureMode::AutoExpose,
            IosCameraExposureMode::ContinuousAutoExposure,
            IosCameraExposureMode::Locked,
        ] {
            let events = RefCell::new(Vec::new());
            let point = (mode != IosCameraExposureMode::Locked)
                .then_some(NormalizedFocusPoint::new(0.25, 0.75));
            assert_eq!(
                configure_exposure(
                    mode,
                    point,
                    |requested, requested_point| {
                        assert_eq!(requested, mode);
                        assert_eq!(requested_point, point);
                        events.borrow_mut().push("supports-mode-and-point");
                        Ok(())
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |requested| {
                        assert_eq!(Some(requested), point);
                        events.borrow_mut().push("set-point");
                    },
                    |requested| {
                        assert_eq!(requested, mode);
                        events.borrow_mut().push("set-mode");
                    },
                    || events.borrow_mut().push("unlock"),
                ),
                Ok(())
            );
            let expected = if point.is_some() {
                vec![
                    "supports-mode-and-point",
                    "lock",
                    "set-point",
                    "set-mode",
                    "unlock",
                ]
            } else {
                vec!["supports-mode-and-point", "lock", "set-mode", "unlock"]
            };
            assert_eq!(*events.borrow(), expected);
        }
    }

    #[test]
    fn exposure_control_reports_lock_failure_and_unlocks_after_unwind() {
        assert_eq!(
            configure_exposure(
                IosCameraExposureMode::AutoExpose,
                None,
                |_, _| Ok(()),
                || Err(()),
                |_| panic!("device lock failed"),
                |_| panic!("device lock failed"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::ConfigurationLockFailed)
        );
        let events = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_exposure(
                IosCameraExposureMode::AutoExpose,
                Some(NormalizedFocusPoint::new(0.5, 0.5)),
                |_, _| Ok(()),
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| events.borrow_mut().push("set-point"),
                |_| panic!("simulated exposure mode set panic"),
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["lock", "set-point", "unlock"]);
    }

    #[test]
    fn custom_exposure_range_preserves_exact_rationals_and_rejects_invalid_native_limits() {
        let minimum = camera_time(1, 60);
        let maximum = camera_time(1, 15);
        let range = custom_exposure_range(minimum, maximum, 50.0, 1600.0).unwrap();
        assert_eq!(range.minimum_duration.value(), 1);
        assert_eq!(range.minimum_duration.timescale(), 60);
        assert_eq!(range.maximum_duration, maximum);
        assert_eq!(range.minimum_iso, 50.0);
        assert_eq!(range.maximum_iso, 1600.0);
        assert!(camera_time(2, 120) >= range.minimum_duration);
        assert!(camera_time(2, 120) <= range.minimum_duration);

        for (value, timescale, valid, finite, epoch) in [
            (1, 60, false, true, 0),
            (1, 60, true, false, 0),
            (1, 60, true, true, 1),
            (0, 60, true, true, 0),
            (-1, 60, true, true, 0),
            (1, 0, true, true, 0),
            (1, -60, true, true, 0),
        ] {
            assert_eq!(
                custom_exposure_duration_from_parts(value, timescale, valid, finite, epoch),
                Err(ExposureControlError::InvalidSupportedCustomExposureRange)
            );
        }

        for (minimum_iso, maximum_iso) in [
            (f32::NAN, 100.0),
            (50.0, f32::INFINITY),
            (f32::NEG_INFINITY, 100.0),
            (0.0, 100.0),
            (-1.0, 100.0),
            (100.0, 50.0),
        ] {
            assert_eq!(
                custom_exposure_range(minimum, maximum, minimum_iso, maximum_iso),
                Err(ExposureControlError::InvalidSupportedCustomExposureRange)
            );
        }
        assert_eq!(
            custom_exposure_range(maximum, minimum, 50.0, 100.0),
            Err(ExposureControlError::InvalidSupportedCustomExposureRange)
        );
    }

    #[test]
    fn custom_exposure_rejects_invalid_duration_and_iso_before_native_queries() {
        for duration in [camera_time(0, 1), camera_time(-1, 1)] {
            assert_eq!(
                configure_custom_exposure(
                    duration,
                    100.0,
                    || panic!("invalid duration must precede mode support"),
                    || panic!("invalid duration must not lock"),
                    || panic!("invalid duration must not read bounds"),
                    |_, _| panic!("invalid duration must not set exposure"),
                    || panic!("device was not locked"),
                ),
                Err(ExposureControlError::InvalidCustomExposureDuration)
            );
        }
        for iso in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            assert_eq!(
                configure_custom_exposure(
                    camera_time(1, 60),
                    iso,
                    || panic!("invalid ISO must precede mode support"),
                    || panic!("invalid ISO must not lock"),
                    || panic!("invalid ISO must not read bounds"),
                    |_, _| panic!("invalid ISO must not set exposure"),
                    || panic!("device was not locked"),
                ),
                Err(ExposureControlError::InvalidCustomExposureIso)
            );
        }
    }

    #[test]
    fn custom_exposure_requires_a_selected_camera_and_checks_support_before_lock() {
        assert_eq!(
            super::with_selected_exposure_camera::<u8, ()>(None, |_| {
                panic!("missing camera must not query custom exposure support")
            }),
            Err(ExposureControlError::NoSelectedCamera)
        );
        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_custom_exposure(
                camera_time(1, 60),
                100.0,
                || {
                    events.borrow_mut().push("support");
                    false
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                || panic!("unsupported custom mode must not read bounds"),
                |_, _| panic!("unsupported custom mode must not set exposure"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::CustomExposureModeUnsupported)
        );
        assert_eq!(*events.borrow(), ["support"]);
    }

    #[test]
    fn custom_exposure_rejects_invalid_range_and_out_of_range_requests_without_set() {
        let minimum = camera_time(1, 60);
        let maximum = camera_time(1, 15);
        let valid_range = custom_exposure_range(minimum, maximum, 50.0, 1600.0).unwrap();

        for (duration, iso, expected) in [
            (
                camera_time(1, 61),
                100.0,
                ExposureControlError::CustomExposureDurationOutOfRange,
            ),
            (
                camera_time(1, 60),
                49.0,
                ExposureControlError::CustomExposureIsoOutOfRange,
            ),
            (
                camera_time(1, 60),
                1601.0,
                ExposureControlError::CustomExposureIsoOutOfRange,
            ),
        ] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_custom_exposure(
                    duration,
                    iso,
                    || {
                        events.borrow_mut().push("support");
                        true
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    || {
                        events.borrow_mut().push("read-range");
                        Ok(valid_range)
                    },
                    |_, _| panic!("out-of-range request must not mutate exposure"),
                    || events.borrow_mut().push("unlock"),
                ),
                Err(expected)
            );
            assert_eq!(
                *events.borrow(),
                ["support", "lock", "read-range", "unlock"]
            );
        }

        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_custom_exposure(
                minimum,
                100.0,
                || {
                    events.borrow_mut().push("support");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                || {
                    events.borrow_mut().push("read-range");
                    Err(ExposureControlError::InvalidSupportedCustomExposureRange)
                },
                |_, _| panic!("invalid native bounds must not mutate exposure"),
                || events.borrow_mut().push("unlock"),
            ),
            Err(ExposureControlError::InvalidSupportedCustomExposureRange)
        );
        assert_eq!(
            *events.borrow(),
            ["support", "lock", "read-range", "unlock"]
        );
    }

    #[test]
    fn custom_exposure_accepts_exact_inclusive_endpoints_in_native_call_order() {
        let minimum = camera_time(1, 60);
        let maximum = camera_time(1, 15);
        let range = custom_exposure_range(minimum, maximum, 50.0, 1600.0).unwrap();
        for (duration, iso) in [(minimum, 50.0), (maximum, 1600.0)] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_custom_exposure(
                    duration,
                    iso,
                    || {
                        events.borrow_mut().push("support");
                        true
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    || {
                        events.borrow_mut().push("read-range");
                        Ok(range)
                    },
                    |actual_duration, actual_iso| {
                        assert_eq!(actual_duration, duration);
                        assert_eq!(actual_iso, iso);
                        events.borrow_mut().push("set");
                    },
                    || events.borrow_mut().push("unlock"),
                ),
                Ok(())
            );
            assert_eq!(
                *events.borrow(),
                ["support", "lock", "read-range", "set", "unlock"]
            );
        }
    }

    #[test]
    fn custom_exposure_lock_failure_and_unwind_preserve_unlock_contract() {
        let range =
            custom_exposure_range(camera_time(1, 60), camera_time(1, 15), 50.0, 1600.0).unwrap();
        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_custom_exposure(
                range.minimum_duration,
                range.minimum_iso,
                || {
                    events.borrow_mut().push("support");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Err(())
                },
                || panic!("failed lock must not read bounds"),
                |_, _| panic!("failed lock must not set exposure"),
                || panic!("failed lock must not unlock"),
            ),
            Err(ExposureControlError::ConfigurationLockFailed)
        );
        assert_eq!(*events.borrow(), ["support", "lock"]);

        let events = RefCell::new(Vec::new());
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_custom_exposure(
                range.maximum_duration,
                range.maximum_iso,
                || {
                    events.borrow_mut().push("support");
                    true
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                || {
                    events.borrow_mut().push("read-range");
                    Ok(range)
                },
                |_, _| {
                    events.borrow_mut().push("set");
                    panic!("simulated custom-exposure setter panic")
                },
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(unwind.is_err());
        assert_eq!(
            *events.borrow(),
            ["support", "lock", "read-range", "set", "unlock"]
        );
    }

    #[test]
    fn exposure_bias_rejects_nonfinite_values_before_native_calls() {
        for bias_ev in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                configure_exposure_target_bias(
                    bias_ev,
                    || panic!("non-finite bias must not read device bounds"),
                    || panic!("non-finite bias must not lock the device"),
                    |_| panic!("non-finite bias must not be set"),
                    || panic!("device was not locked"),
                ),
                Err(ExposureControlError::InvalidExposureBias)
            );
        }
    }

    #[test]
    fn exposure_bias_rejects_invalid_bounds_and_out_of_range_values_without_setting() {
        for bounds in [
            (f32::NAN, 1.0),
            (-1.0, f32::INFINITY),
            (f32::NEG_INFINITY, 1.0),
            (1.0, -1.0),
        ] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_exposure_target_bias(
                    0.0,
                    || {
                        events.borrow_mut().push("read-range");
                        bounds
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |_| panic!("invalid native range must not be set"),
                    || events.borrow_mut().push("unlock"),
                ),
                Err(ExposureControlError::InvalidSupportedExposureBiasRange)
            );
            assert_eq!(*events.borrow(), ["lock", "read-range", "unlock"]);
        }
        for bias_ev in [-2.01_f32, 2.01_f32] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_exposure_target_bias(
                    bias_ev,
                    || {
                        events.borrow_mut().push("read-range");
                        (-2.0, 2.0)
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |_| panic!("out-of-range bias must not be set"),
                    || events.borrow_mut().push("unlock"),
                ),
                Err(ExposureControlError::ExposureBiasOutOfRange)
            );
            assert_eq!(*events.borrow(), ["lock", "read-range", "unlock"]);
        }
    }

    #[test]
    fn exposure_bias_locks_reads_range_sets_exact_value_without_callback_and_unlocks() {
        for bias_ev in [-2.0_f32, 2.0_f32] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_exposure_target_bias(
                    bias_ev,
                    || {
                        events.borrow_mut().push("read-range");
                        (-2.0, 2.0)
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |set| {
                        assert_eq!(set, bias_ev);
                        events.borrow_mut().push("set-exact-bias-without-callback");
                    },
                    || events.borrow_mut().push("unlock"),
                ),
                Ok(())
            );
            assert_eq!(
                *events.borrow(),
                [
                    "lock",
                    "read-range",
                    "set-exact-bias-without-callback",
                    "unlock"
                ]
            );
        }
        assert_eq!(
            exposure_target_bias_range(-2.0, 2.0),
            Ok(super::IosExposureTargetBiasRange {
                minimum: -2.0,
                maximum: 2.0,
            })
        );
    }

    #[test]
    fn exposure_bias_lock_failure_and_unwind_do_not_leak_or_mutate() {
        assert_eq!(
            configure_exposure_target_bias(
                0.0,
                || panic!("lock failure must not query device bounds"),
                || Err(()),
                |_| panic!("lock failure must not set bias"),
                || panic!("device was not locked"),
            ),
            Err(ExposureControlError::ConfigurationLockFailed)
        );
        let events = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_exposure_target_bias(
                0.5,
                || {
                    events.borrow_mut().push("read-range");
                    (-1.0, 1.0)
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| panic!("simulated bias-set panic"),
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["lock", "read-range", "unlock"]);
    }

    #[test]
    fn exposure_bias_range_snapshot_rejects_invalid_native_bounds() {
        for (minimum, maximum) in [
            (f32::NAN, 1.0),
            (-1.0, f32::INFINITY),
            (f32::NEG_INFINITY, 1.0),
            (1.0, -1.0),
        ] {
            assert_eq!(
                exposure_target_bias_range(minimum, maximum),
                Err(ExposureControlError::InvalidSupportedExposureBiasRange)
            );
        }
    }

    #[test]
    fn zoom_control_rejects_nonfinite_and_below_minimum_factors_without_native_calls() {
        for factor in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                configure_video_zoom_factor(
                    factor,
                    || panic!("range must not be read for a non-finite factor"),
                    || panic!("device must not be locked for a non-finite factor"),
                    |_| panic!("non-finite factor must not be set"),
                    || panic!("device was not locked"),
                ),
                Err(ZoomControlError::InvalidZoomFactor)
            );
        }
        assert_eq!(
            configure_video_zoom_factor(
                0.5,
                || panic!("range must not be read below the known minimum"),
                || panic!("device must not be locked below the known minimum"),
                |_| panic!("below-minimum factor must not be set"),
                || panic!("device was not locked"),
            ),
            Err(ZoomControlError::ZoomFactorOutOfRange)
        );
    }

    #[test]
    fn zoom_control_rejects_supported_range_and_lock_errors() {
        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_video_zoom_factor(
                2.0,
                || {
                    events.borrow_mut().push("range");
                    1.0
                },
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| panic!("factor must not be set above the maximum"),
                || events.borrow_mut().push("unlock"),
            ),
            Err(ZoomControlError::ZoomFactorOutOfRange)
        );
        assert_eq!(*events.borrow(), ["lock", "range", "unlock"]);

        let events = RefCell::new(Vec::new());
        assert_eq!(
            configure_video_zoom_factor(
                2.0,
                || panic!("range must not be read after a failed lock"),
                || {
                    events.borrow_mut().push("lock");
                    Err(())
                },
                |_| panic!("factor must not be set after a failed lock"),
                || panic!("unlock is only valid after acquiring the lock"),
            ),
            Err(ZoomControlError::ConfigurationLockFailed)
        );
        assert_eq!(*events.borrow(), ["lock"]);

        for invalid_max in [f64::NAN, f64::INFINITY, 0.5] {
            assert_eq!(
                configure_video_zoom_factor(
                    1.0,
                    || invalid_max,
                    || Ok(()),
                    |_| panic!("invalid device range must not be used"),
                    || {},
                ),
                Err(ZoomControlError::InvalidSupportedRange)
            );
        }
    }

    #[test]
    fn zoom_control_sets_exact_factor_between_lock_and_unlock() {
        for factor in [1.0, 2.5, 5.0] {
            let events = RefCell::new(Vec::new());
            assert_eq!(
                configure_video_zoom_factor(
                    factor,
                    || {
                        events.borrow_mut().push("range");
                        5.0
                    },
                    || {
                        events.borrow_mut().push("lock");
                        Ok(())
                    },
                    |requested| {
                        assert_eq!(requested, factor);
                        events.borrow_mut().push("set");
                    },
                    || events.borrow_mut().push("unlock"),
                ),
                Ok(())
            );
            assert_eq!(*events.borrow(), ["lock", "range", "set", "unlock"]);
        }

        let events = RefCell::new(Vec::new());
        let factor = 2.5;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            configure_video_zoom_factor(
                factor,
                || 5.0,
                || {
                    events.borrow_mut().push("lock");
                    Ok(())
                },
                |_| panic!("simulated zoom-set panic"),
                || events.borrow_mut().push("unlock"),
            )
        }));
        assert!(result.is_err());
        assert_eq!(*events.borrow(), ["lock", "unlock"]);
    }

    #[test]
    fn video_frame_stream_allows_one_start_and_terminal_stop_only() {
        let mut state = VideoFrameStreamState::default();
        assert_eq!(state.begin(), Ok(()));
        assert_eq!(state, VideoFrameStreamState::Active);
        assert_eq!(state.begin(), Err(VideoFrameStreamError::AlreadyActive));
        assert!(state.request_stop());
        assert_eq!(state, VideoFrameStreamState::StopRequested);
        assert_eq!(state.begin(), Err(VideoFrameStreamError::AlreadyStopped));
        assert!(state.finish_stop());
        assert!(!state.finish_stop());
        assert_eq!(state.begin(), Err(VideoFrameStreamError::AlreadyStopped));

        let mut unstarted = VideoFrameStreamState::default();
        assert!(unstarted.close_unstarted());
        assert_eq!(unstarted, VideoFrameStreamState::Stopped);
        assert_eq!(
            unstarted.begin(),
            Err(VideoFrameStreamError::AlreadyStopped)
        );
    }

    #[test]
    fn video_frame_callback_skips_stopped_frames_and_contains_panics() {
        let active = AtomicBool::new(true);
        let mut seen = 0;
        assert!(invoke_video_frame_callback(
            &active,
            &mut |sample: &u8| seen += usize::from(*sample),
            &7,
        ));
        assert_eq!(seen, 7);
        active.store(false, Ordering::Release);
        assert!(!invoke_video_frame_callback(
            &active,
            &mut |_| panic!("inactive"),
            &0
        ));
        assert_eq!(seen, 7);

        active.store(true, Ordering::Release);
        assert!(!invoke_video_frame_callback(
            &active,
            &mut |_| panic!("callback"),
            &0
        ));
        assert!(!active.load(Ordering::Acquire));
    }

    #[test]
    fn video_frame_callback_capture_drop_panic_is_contained() {
        struct PanicOnDrop;

        impl Drop for PanicOnDrop {
            fn drop(&mut self) {
                panic!("callback capture drop");
            }
        }

        let capture = PanicOnDrop;
        let callback = move |_: &u8| {
            std::hint::black_box(&capture);
        };
        drop_video_frame_callback_safely(Some(callback));
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
        assert!(state.pending_session_stop);
        state.finish_session_stop();
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
