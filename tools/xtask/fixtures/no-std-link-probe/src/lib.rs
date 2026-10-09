#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc as alloc_crate;

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use core::future::Future;
use core::mem::{align_of, size_of};
use core::panic::PanicInfo;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Poll, Waker};
use framework_abi::{
    FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, FrameworkStr,
    framework_owned_buffer_destroy,
};
use framework_accessory::{AccessoryPresenceSnapshot, ExternalAccessoryBackend};
use framework_alloc::GenerationalSlab;
use framework_app::{Application, ApplicationBackend, ApplicationState, LifecycleEvent};
use framework_async::{Completion, OperationPhase, OperationState};
use framework_audio::HdrPlaybackEligibility;
use framework_auth::{
    AppTrackingAuthorizationBackend, AppTrackingAuthorizationStatus, AuthenticationPolicy,
    AuthenticationRequest,
};
use framework_background::AppRefreshTaskId;
use framework_background_execution::{
    BackgroundExecution, BackgroundExecutionBackend, BackgroundExecutionLease, ExpirySignal,
};
use framework_bluetooth::{
    Bluetooth, BluetoothAuthorization, BluetoothAuthorizationBackend, BluetoothCentral,
    BluetoothCentralBackend, BluetoothDiscovery, BluetoothPeripheralId, BluetoothScanError,
    BluetoothScanState,
};
use framework_calendar::{
    Calendar, CalendarAuthorizationBackend, CalendarAuthorizationStatus, CalendarError,
};
use framework_cloud::UbiquityIdentitySnapshot;
use framework_connection::{
    Endpoint, MAX_CHUNK_BYTES, NativeConnectionError, ReadChunk, ReadTerminal,
};
use framework_connectivity::{
    Connectivity, NetworkPathBackend, NetworkPathError, NetworkPathSnapshot, NetworkPathStatus,
};
use framework_contacts::{Contacts, ContactsAuthorization, ContactsBackend, ContactsError};
use framework_core::{CompactHandle, Error, ErrorKind, Generation, OperationId, PlatformErrorCode};
use framework_data::ByteView;
use framework_device_integrity::AvailabilitySnapshot;
use framework_files::{AppDirectory, AppPath};
use framework_format::Uri;
use framework_game::{
    LocalPlayer, LocalPlayerAuthenticationBackend, LocalPlayerAuthenticationStatus,
};
use framework_health_authorization::{
    HealthAuthorizationBackend, HealthAuthorizationCompletion, HealthAuthorizationError,
    HealthAuthorizationRequest, HealthDataType, HealthDataTypeKind,
};
use framework_image::{ImageDimensions, ImageMetadata};
use framework_key_support::{P256PublicKey, P256_PUBLIC_KEY_X963_LEN};
use framework_location::Coordinate;
use framework_maps::WorldTrackingSupport;
use framework_media::{
    HardwareDecodeSupport, MediaTime, OtherAudioPlaybackSnapshot, ReplayKitAvailability,
    SoundAnalysisSupportSnapshot, VideoCodecType,
};
use framework_media_authorization::{CaptureMedia, MediaAuthorization, MediaAuthorizationStatus};
use framework_metal::{MetalDevicePresence, MetalDevicePresenceBackend, MetalDeviceQuery};
use framework_motion::Acceleration;
use framework_nearby::{NearbyInteractionCapabilityBackend, NearbyInteractionCapabilitySnapshot};
use framework_network::HttpUrl;
use framework_nfc::{NfcReader, NfcReaderAvailability, NfcReaderAvailabilityBackend};
use framework_notifications::{NotificationError, NotificationId};
use framework_photos::{
    PhotoLibrary, PhotoLibraryAuthorizationBackend, PhotoLibraryAuthorizationStatus,
};
use framework_platform::{CurrentPlatform, PlatformMarker, is_current};
use framework_preferences::PreferenceKey;
use framework_resources::ResourcePath;
use framework_roomplan::RoomPlanDeviceSupport;
use framework_secure_storage::ServiceId;
use framework_sharing::ShareItem;
use framework_spritekit::SpriteNodePosition;
use framework_transfer::{
    DownloadRequest, TransferBackend, TransferError, TransferId, TransferSnapshot, TransferStatus,
    Transfers,
};
use framework_ui::Frame;
use framework_vision::TextRecognitionRevisionSupport;
use framework_watch_connectivity::{WatchConnectivityBackend, WatchConnectivitySupport};
use framework_web::{HttpsUrl, NavigationState};

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(pointer: *mut c_void);
    fn abort() -> !;
}

struct CAllocator;

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alignment = layout.align().max(align_of::<usize>());
        let Some(size) = layout
            .size()
            .max(1)
            .checked_add(alignment - 1)
            .and_then(|size| size.checked_add(size_of::<usize>()))
        else {
            return ptr::null_mut();
        };
        let raw = unsafe { malloc(size) }.cast::<u8>();
        if raw.is_null() {
            return ptr::null_mut();
        }
        let Some(start) = (raw as usize).checked_add(size_of::<usize>()) else {
            unsafe { free(raw.cast()) };
            return ptr::null_mut();
        };
        let Some(aligned_base) = start.checked_add(alignment - 1) else {
            unsafe { free(raw.cast()) };
            return ptr::null_mut();
        };
        let aligned = aligned_base & !(alignment - 1);
        let header = (aligned - size_of::<usize>()) as *mut usize;
        unsafe { header.write(raw as usize) };
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, pointer: *mut u8, _layout: Layout) {
        if !pointer.is_null() {
            let header = unsafe { pointer.sub(size_of::<usize>()).cast::<usize>() };
            let raw = unsafe { header.read() } as *mut c_void;
            unsafe { free(raw) };
        }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CAllocator = CAllocator;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    unsafe { abort() }
}

#[unsafe(no_mangle)]
pub extern "C" fn framework_no_std_link_probe() -> i32 {
    if !check_core_and_platform() {
        return 1;
    }
    if !check_alloc_apis() {
        return 2;
    }
    if !check_async_api() {
        return 3;
    }
    if !check_abi_apis() {
        return 4;
    }
    if !check_explicit_allocator() {
        return 5;
    }
    if !check_framework_data() {
        return 6;
    }
    if !check_framework_app() {
        return 7;
    }
    if !check_framework_files() {
        return 8;
    }
    if !check_framework_preferences() {
        return 9;
    }
    if !check_framework_resources() {
        return 10;
    }
    if !check_framework_format() {
        return 11;
    }
    if !check_framework_network() {
        return 12;
    }
    if !check_framework_transfer() {
        return 13;
    }
    if !check_framework_secure_storage() {
        return 14;
    }
    if !check_framework_auth() {
        return 15;
    }
    if !check_framework_notifications() {
        return 16;
    }
    if !check_framework_location() {
        return 17;
    }
    if !check_framework_motion() {
        return 18;
    }
    if !check_framework_sharing() {
        return 19;
    }
    if !check_framework_ui() {
        return 20;
    }
    if !check_framework_connectivity() {
        return 21;
    }
    if !check_framework_media() {
        return 22;
    }
    if !check_framework_connection() {
        return 23;
    }
    if !check_framework_background() {
        return 24;
    }
    if !check_framework_image() {
        return 25;
    }
    if !check_framework_photos() {
        return 26;
    }
    if !check_framework_background_execution() {
        return 27;
    }
    if !check_framework_contacts() {
        return 28;
    }
    if !check_framework_calendar() {
        return 29;
    }
    if !check_framework_bluetooth() {
        return 30;
    }
    if !check_framework_health_authorization() {
        return 31;
    }
    if !check_framework_cloud() {
        return 32;
    }
    if !check_framework_web() {
        return 33;
    }
    if !check_framework_nfc() {
        return 34;
    }
    if !check_framework_media_authorization() {
        return 35;
    }
    if !check_framework_nearby() {
        return 36;
    }
    if !check_framework_metal() {
        return 37;
    }
    if !check_framework_device_integrity() {
        return 38;
    }
    if !check_framework_watch_connectivity() {
        return 39;
    }
    if !check_framework_accessory() {
        return 40;
    }
    if !check_framework_audio() {
        return 41;
    }
    if !check_framework_game() {
        return 42;
    }
    if !check_framework_maps() {
        return 43;
    }
    if !check_framework_vision() {
        return 44;
    }
    if !check_framework_roomplan() {
        return 45;
    }
    if !check_framework_key_support() {
        return 46;
    }
    if !check_framework_spritekit() {
        return 47;
    }
    0
}

struct ProbeMetalDevicePresence;

impl MetalDevicePresenceBackend for ProbeMetalDevicePresence {
    fn snapshot(&self) -> MetalDevicePresence {
        MetalDevicePresence::Present
    }
}

fn check_framework_metal() -> bool {
    MetalDeviceQuery::new(ProbeMetalDevicePresence).snapshot() == MetalDevicePresence::Present
}

fn check_framework_device_integrity() -> bool {
    let snapshot = AvailabilitySnapshot::new(true, false);
    snapshot.device_check_supported() && !snapshot.app_attest_supported()
}

struct ProbeWatchConnectivity;

impl WatchConnectivityBackend for ProbeWatchConnectivity {
    fn session_support() -> WatchConnectivitySupport {
        WatchConnectivitySupport::Supported
    }
}

fn check_framework_watch_connectivity() -> bool {
    ProbeWatchConnectivity::session_support() == WatchConnectivitySupport::Supported
}

struct ProbeExternalAccessory;

impl ExternalAccessoryBackend for ProbeExternalAccessory {
    fn connected_accessory_presence() -> AccessoryPresenceSnapshot {
        AccessoryPresenceSnapshot::OneOrMoreAvailable
    }
}

fn check_framework_accessory() -> bool {
    ProbeExternalAccessory::connected_accessory_presence()
        == AccessoryPresenceSnapshot::OneOrMoreAvailable
}

fn check_framework_audio() -> bool {
    HdrPlaybackEligibility::new(true).is_eligible()
        && !HdrPlaybackEligibility::new(false).is_eligible()
}

fn check_framework_background() -> bool {
    match AppRefreshTaskId::new("org.example.no-std-link-probe") {
        Ok(task_id) => task_id.as_str() == "org.example.no-std-link-probe",
        Err(_) => false,
    }
}

struct ProbeBackgroundExecution;

struct ProbeBackgroundLease;

struct ProbeBackgroundExpiry;

impl ExpirySignal for ProbeBackgroundExpiry {
    fn is_expired(&self) -> bool {
        false
    }
}

impl BackgroundExecutionLease for ProbeBackgroundLease {
    type Expiry = ProbeBackgroundExpiry;

    fn expiry(&self) -> &Self::Expiry {
        static EXPIRY: ProbeBackgroundExpiry = ProbeBackgroundExpiry;
        &EXPIRY
    }

    fn end(self) {}
}

impl BackgroundExecutionBackend for ProbeBackgroundExecution {
    type Context = ();
    type Lease = ProbeBackgroundLease;

    fn begin(&self, _context: Self::Context) -> framework_core::Result<Self::Lease> {
        Ok(ProbeBackgroundLease)
    }
}

fn check_framework_background_execution() -> bool {
    let execution = BackgroundExecution::new(ProbeBackgroundExecution);
    let Ok(lease) = execution.begin(()) else {
        return false;
    };
    let not_expired = !lease.expiry().is_expired();
    lease.end();
    not_expired
}

struct ProbeContactsBackend;

impl ContactsBackend for ProbeContactsBackend {
    fn availability(&self) -> framework_core::Availability {
        framework_core::Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        ContactsAuthorization::Limited
    }

    type RequestAuthorizationFuture<'a> =
        core::future::Ready<core::result::Result<ContactsAuthorization, ContactsError>>;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        core::future::ready(Ok(ContactsAuthorization::Limited))
    }
}

fn check_framework_contacts() -> bool {
    let mut contacts = Contacts::new(ProbeContactsBackend);
    if contacts.authorization_status() != ContactsAuthorization::Limited
        || !contacts.authorization_status().allows_contact_access()
        || contacts.authorization_status().is_full_access()
    {
        return false;
    }
    let mut future = contacts.request_authorization();
    let mut context = Context::from_waker(Waker::noop());
    matches!(
        Pin::new(&mut future).poll(&mut context),
        Poll::Ready(Ok(ContactsAuthorization::Limited))
    )
}

struct ProbeCalendarBackend;

impl CalendarAuthorizationBackend for ProbeCalendarBackend {
    fn authorization_status(&self) -> CalendarAuthorizationStatus {
        CalendarAuthorizationStatus::WriteOnly
    }

    type RequestFullAccessFuture<'a> =
        core::future::Ready<core::result::Result<CalendarAuthorizationStatus, CalendarError>>;

    fn request_full_access<'a>(&'a mut self) -> Self::RequestFullAccessFuture<'a> {
        core::future::ready(Ok(CalendarAuthorizationStatus::FullAccess))
    }
}

fn check_framework_calendar() -> bool {
    let mut calendar = Calendar::new(ProbeCalendarBackend);
    if calendar.authorization_status() != CalendarAuthorizationStatus::WriteOnly
        || CalendarAuthorizationStatus::WriteOnly == CalendarAuthorizationStatus::FullAccess
    {
        return false;
    }
    let mut future = core::pin::pin!(calendar.request_full_access());
    let mut context = Context::from_waker(Waker::noop());
    matches!(
        future.as_mut().poll(&mut context),
        Poll::Ready(Ok(CalendarAuthorizationStatus::FullAccess))
    )
}

struct ProbeBluetoothBackend;

impl BluetoothAuthorizationBackend for ProbeBluetoothBackend {
    fn authorization_status(&self) -> BluetoothAuthorization {
        BluetoothAuthorization::Allowed
    }
}

fn check_framework_bluetooth() -> bool {
    let bluetooth = Bluetooth::new(ProbeBluetoothBackend);
    if bluetooth.authorization_status() != BluetoothAuthorization::Allowed {
        return false;
    }
    let discovery = BluetoothDiscovery {
        peripheral: BluetoothPeripheralId::from_uuid_bytes([0x2A; 16]),
        rssi_dbm: Some(-42),
    };
    let mut central = BluetoothCentral::new(ProbeBluetoothCentral {
        state: BluetoothScanState::Stopped,
        discovery: Some(discovery),
        dropped: 2,
    });
    if central.start_unfiltered_scan() != Ok(())
        || central.scan_state() != BluetoothScanState::Starting
        || central.try_next_discovery() != Some(discovery)
        || central.take_dropped_discovery_count() != 2
        || central.start_unfiltered_scan() != Err(BluetoothScanError::AlreadyActive)
    {
        return false;
    }
    central.stop_scan();
    central.scan_state() == BluetoothScanState::Stopped
}

struct ProbeBluetoothCentral {
    state: BluetoothScanState,
    discovery: Option<BluetoothDiscovery>,
    dropped: u32,
}

impl BluetoothCentralBackend for ProbeBluetoothCentral {
    fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError> {
        if self.state != BluetoothScanState::Stopped {
            return Err(BluetoothScanError::AlreadyActive);
        }
        self.state = BluetoothScanState::Starting;
        Ok(())
    }

    fn stop_scan(&mut self) {
        self.state = BluetoothScanState::Stopped;
    }

    fn scan_state(&self) -> BluetoothScanState {
        self.state
    }

    fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery> {
        self.discovery.take()
    }

    fn take_dropped_discovery_count(&mut self) -> u32 {
        core::mem::take(&mut self.dropped)
    }
}

fn check_framework_cloud() -> bool {
    UbiquityIdentitySnapshot::from_token_presence(true).token_present()
}

struct ProbeGameBackend;

impl LocalPlayerAuthenticationBackend for ProbeGameBackend {
    fn availability(&self) -> framework_core::Availability {
        framework_core::Availability::Available
    }

    fn authentication_status(&self) -> LocalPlayerAuthenticationStatus {
        LocalPlayerAuthenticationStatus::Authenticated
    }
}

fn check_framework_game() -> bool {
    let mut player = LocalPlayer::new(ProbeGameBackend);
    if player.availability() != framework_core::Availability::Available
        || player.authentication_status() != LocalPlayerAuthenticationStatus::Authenticated
        || player.backend().authentication_status()
            != LocalPlayerAuthenticationStatus::Authenticated
        || player.backend_mut().availability() != framework_core::Availability::Available
    {
        return false;
    }
    player.into_backend().authentication_status() == LocalPlayerAuthenticationStatus::Authenticated
}

fn check_framework_maps() -> bool {
    WorldTrackingSupport::from_system(true).is_supported()
        && !WorldTrackingSupport::from_system(false).is_supported()
}

fn check_framework_vision() -> bool {
    let supported = TextRecognitionRevisionSupport::from_system(1, true);
    let unsupported = TextRecognitionRevisionSupport::from_system(1, false);
    supported.revision() == 1
        && supported.is_supported()
        && unsupported.revision() == 1
        && !unsupported.is_supported()
}

fn check_framework_roomplan() -> bool {
    let supported = RoomPlanDeviceSupport::from_platform_query(true);
    let unsupported = RoomPlanDeviceSupport::from_platform_query(false);
    supported.is_supported() && !unsupported.is_supported()
}

fn check_framework_web() -> bool {
    let Ok(url) = HttpsUrl::new("https://example.com/path") else {
        return false;
    };
    url.as_str() == "https://example.com/path" && NavigationState::new(true, false).can_go_back()
}

struct ProbeNfcBackend;

impl NfcReaderAvailabilityBackend for ProbeNfcBackend {
    fn snapshot(&self) -> NfcReaderAvailability {
        NfcReaderAvailability::Supported
    }
}

fn check_framework_nfc() -> bool {
    NfcReader::new(ProbeNfcBackend).snapshot() == NfcReaderAvailability::Supported
}

struct ProbeMediaAuthorization;

impl MediaAuthorization for ProbeMediaAuthorization {
    fn authorization_status(_media: CaptureMedia) -> MediaAuthorizationStatus {
        MediaAuthorizationStatus::Authorized
    }
}

fn check_framework_media_authorization() -> bool {
    ProbeMediaAuthorization::authorization_status(CaptureMedia::Camera)
        == MediaAuthorizationStatus::Authorized
}

struct ProbeNearbyBackend(bool);

impl NearbyInteractionCapabilityBackend for ProbeNearbyBackend {
    fn snapshot(&self) -> NearbyInteractionCapabilitySnapshot {
        NearbyInteractionCapabilitySnapshot::new(self.0)
    }
}

fn check_framework_nearby() -> bool {
    ProbeNearbyBackend(true)
        .snapshot()
        .supports_precise_distance_measurement()
}

struct ProbeHealthAuthorizationBackend;

impl HealthAuthorizationBackend for ProbeHealthAuthorizationBackend {
    fn is_available(&self) -> bool {
        true
    }

    fn request_authorization<F>(
        &self,
        _request: &HealthAuthorizationRequest<'_, '_>,
        _completion: F,
    ) -> Result<(), HealthAuthorizationError>
    where
        F: FnOnce(HealthAuthorizationCompletion) + Send + 'static,
    {
        Ok(())
    }
}

fn check_framework_health_authorization() -> bool {
    let Ok(step_count) = HealthDataType::new(
        HealthDataTypeKind::Quantity,
        "HKQuantityTypeIdentifierStepCount",
    ) else {
        return false;
    };
    let read_types = [step_count];
    let Ok(request) = HealthAuthorizationRequest::new(&read_types, &[]) else {
        return false;
    };
    let backend = ProbeHealthAuthorizationBackend;
    backend.is_available()
        && backend
            .request_authorization(&request, |_completion| {})
            .is_ok()
}

fn check_framework_image() -> bool {
    let Ok(dimensions) = ImageDimensions::new(640, 480) else {
        return false;
    };
    let Ok(metadata) = ImageMetadata::new(dimensions, 3) else {
        return false;
    };
    metadata.dimensions().pixel_count() == 307_200 && metadata.image_count() == 3
}

struct ProbePhotosBackend;

impl PhotoLibraryAuthorizationBackend for ProbePhotosBackend {
    type RequestAuthorizationFuture<'a> = core::future::Ready<PhotoLibraryAuthorizationStatus>;

    fn authorization_status(&self) -> PhotoLibraryAuthorizationStatus {
        PhotoLibraryAuthorizationStatus::Limited
    }

    fn request_authorization(&mut self) -> Self::RequestAuthorizationFuture<'_> {
        core::future::ready(PhotoLibraryAuthorizationStatus::Limited)
    }
}

fn check_framework_photos() -> bool {
    let mut photos = PhotoLibrary::new(ProbePhotosBackend);
    if photos.authorization_status() != PhotoLibraryAuthorizationStatus::Limited
        || PhotoLibraryAuthorizationStatus::Limited == PhotoLibraryAuthorizationStatus::Authorized
    {
        return false;
    }
    let mut future = photos.request_authorization();
    let mut context = Context::from_waker(Waker::noop());
    matches!(
        Pin::new(&mut future).poll(&mut context),
        Poll::Ready(PhotoLibraryAuthorizationStatus::Limited)
    )
}

fn check_core_and_platform() -> bool {
    let Some(operation_id) = OperationId::new(7) else {
        return false;
    };
    let Some(generation) = Generation::new(3) else {
        return false;
    };
    let handle = CompactHandle::new(11, generation);
    let error =
        Error::new(ErrorKind::Timeout).with_platform_code(match PlatformErrorCode::new(-42) {
            Some(code) => code,
            None => return false,
        });
    operation_id.get() == 7
        && handle.index() == 11
        && handle.generation().get() == 3
        && FrameworkStatus::from_error(error).code() == FrameworkStatus::TIMEOUT.code()
        && CurrentPlatform::PLATFORM == framework_core::Platform::current()
        && is_current::<CurrentPlatform>()
}

fn check_alloc_apis() -> bool {
    let slab = GenerationalSlab::<u32>::new();
    slab.is_empty() && slab.len() == 0 && slab.capacity() == 0
}

fn check_async_api() -> bool {
    let state = OperationState::<u32, Error>::new();
    if !state.start() || !state.complete(Ok(29)) || state.phase() != OperationPhase::Completed {
        return false;
    }
    let mut future = match state.future() {
        Ok(future) => future,
        Err(_) => return false,
    };
    let mut context = Context::from_waker(Waker::noop());
    matches!(
        Pin::new(&mut future).poll(&mut context),
        Poll::Ready(Completion::Success(29))
    )
}

fn check_abi_apis() -> bool {
    let bytes = FrameworkSlice::from_bytes(b"no-std");
    let string = FrameworkStr::from_utf8("no_std");
    if bytes.map(FrameworkSlice::length) != Some(6) || string.map(FrameworkStr::length) != Some(6) {
        return false;
    }
    let mut buffer = match FrameworkOwnedBuffer::try_from_vec(alloc_crate::vec::Vec::new()) {
        Ok(buffer) => buffer,
        Err(_) => return false,
    };
    if buffer.length() != 0 || buffer.capacity() != 0 {
        return false;
    }
    unsafe { framework_owned_buffer_destroy(&mut buffer) };
    buffer.length() == 0 && buffer.capacity() == 0
}

fn check_explicit_allocator() -> bool {
    let layout = match Layout::from_size_align(32, 64) {
        Ok(layout) => layout,
        Err(_) => return false,
    };
    let pointer = unsafe { alloc_crate::alloc::alloc(layout) };
    if pointer.is_null() {
        return false;
    }
    unsafe {
        ptr::write_bytes(pointer, 0x5a, layout.size());
        alloc_crate::alloc::dealloc(pointer, layout);
    }
    true
}

fn check_framework_data() -> bool {
    match ByteView::new(b"valid utf-8").try_as_utf8() {
        Ok(text) => text.as_str() == "valid utf-8",
        Err(_) => false,
    }
}

struct ProbeApplication;

impl ApplicationBackend for ProbeApplication {
    fn availability(&self) -> framework_core::Availability {
        framework_core::Availability::Available
    }

    fn state(&self) -> framework_core::Result<ApplicationState> {
        Ok(ApplicationState::Foreground)
    }

    fn poll_event(&mut self) -> framework_core::Result<Option<LifecycleEvent>> {
        Ok(None)
    }
}

fn check_framework_app() -> bool {
    Application::new(ProbeApplication).availability() == framework_core::Availability::Available
}

fn check_framework_files() -> bool {
    match AppPath::new(AppDirectory::Documents, "probe/data") {
        Ok(path) => path.relative() == "probe/data",
        Err(_) => false,
    }
}

fn check_framework_preferences() -> bool {
    match PreferenceKey::new("probe-key") {
        Ok(key) => key.as_str() == "probe-key",
        Err(_) => false,
    }
}

fn check_framework_resources() -> bool {
    match ResourcePath::new("images/probe") {
        Ok(path) => path.relative() == "images/probe",
        Err(_) => false,
    }
}

fn check_framework_format() -> bool {
    match Uri::new("https://example.invalid/probe") {
        Ok(uri) => uri.scheme() == "https" && uri.path() == "/probe",
        Err(_) => false,
    }
}

fn check_framework_key_support() -> bool {
    let mut bytes = [0_u8; P256_PUBLIC_KEY_X963_LEN];
    bytes[0] = 0x04;
    match P256PublicKey::from_x963_uncompressed(&bytes) {
        Ok(key) => key.as_x963_uncompressed_bytes()[0] == 0x04,
        Err(_) => false,
    }
}

fn check_framework_spritekit() -> bool {
    match SpriteNodePosition::new(-2.5, 3.0) {
        Ok(position) => position.x() == -2.5 && position.y() == 3.0,
        Err(_) => false,
    }
}

fn check_framework_network() -> bool {
    match HttpUrl::new("https://example.invalid/probe") {
        Ok(url) => url.as_str() == "https://example.invalid/probe",
        Err(_) => false,
    }
}

struct ProbeConnectivity;

impl NetworkPathBackend for ProbeConnectivity {
    type CurrentPathFuture<'a>
        = core::future::Ready<Result<NetworkPathSnapshot, NetworkPathError>>
    where
        Self: 'a;

    fn current_path<'a>(&'a mut self) -> Self::CurrentPathFuture<'a> {
        core::future::ready(Ok(NetworkPathSnapshot::new(NetworkPathStatus::Satisfied)))
    }
}

fn check_framework_connectivity() -> bool {
    let mut connectivity = Connectivity::new(ProbeConnectivity);
    let mut future = core::pin::pin!(connectivity.current_path());
    let mut context = Context::from_waker(Waker::noop());
    matches!(
        future.as_mut().poll(&mut context),
        Poll::Ready(Ok(snapshot)) if snapshot.status() == NetworkPathStatus::Satisfied
    )
}

fn check_framework_media() -> bool {
    let half = match MediaTime::new(1, 2) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let three_sixths = match MediaTime::new(3, 6) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let quarter = match MediaTime::new(1, 4) {
        Ok(value) => value,
        Err(_) => return false,
    };
    half == three_sixths
        && half.compare(three_sixths) == core::cmp::Ordering::Equal
        && half.compare(quarter) == core::cmp::Ordering::Greater
        && MediaTime::new(1, 0).is_err()
        && ReplayKitAvailability::new(true).is_available_for_recording()
        && SoundAnalysisSupportSnapshot::new(true).is_supported()
        && OtherAudioPlaybackSnapshot::new(true).is_other_audio_playing()
        && VideoCodecType::from_fourcc(*b"avc1").to_fourcc() == *b"avc1"
        && VideoCodecType::from_fourcc(*b"avc1").to_raw() == u32::from_be_bytes(*b"avc1")
        && HardwareDecodeSupport::from_system(true).is_supported()
}

fn check_framework_connection() -> bool {
    let endpoint = match Endpoint::new("example.invalid", 443) {
        Ok(endpoint) => endpoint,
        Err(_) => return false,
    };
    let error = NativeConnectionError::new(3, -9807);
    let chunk = match ReadChunk::new(
        alloc_crate::vec![1, 2, 3],
        ReadTerminal::Error(framework_connection::ConnectionError::Native(error)),
    ) {
        Ok(chunk) => chunk,
        Err(_) => return false,
    };
    endpoint.host() == "example.invalid"
        && endpoint.port().get() == 443
        && Endpoint::new("example.invalid", 0).is_err()
        && MAX_CHUNK_BYTES == 1_048_576
        && chunk.bytes() == [1, 2, 3]
        && error.domain() == 3
        && error.code() == -9807
}

struct ProbeTransfer;

impl TransferBackend for ProbeTransfer {
    fn availability(&self) -> framework_core::Availability {
        framework_core::Availability::Available
    }

    fn start_download(&mut self, _request: DownloadRequest<'_>) -> Result<(), TransferError> {
        Ok(())
    }

    fn status(&mut self, _id: TransferId) -> Result<Option<TransferSnapshot>, TransferError> {
        Ok(None)
    }

    fn cancel(&mut self, _id: TransferId) -> Result<(), TransferError> {
        Ok(())
    }

    fn forget(&mut self, _id: TransferId) -> Result<(), TransferError> {
        Ok(())
    }
}

fn check_framework_transfer() -> bool {
    match TransferId::new(7) {
        Some(id) => {
            id.get() == 7
                && Transfers::new(ProbeTransfer).availability()
                    == framework_core::Availability::Available
                && !TransferStatus::Queued.is_terminal()
        }
        None => false,
    }
}

fn check_framework_secure_storage() -> bool {
    match ServiceId::new("probe-service") {
        Ok(service) => service.as_str() == "probe-service",
        Err(_) => false,
    }
}

fn check_framework_auth() -> bool {
    let request_matches =
        match AuthenticationRequest::new(AuthenticationPolicy::DeviceOwner, "Probe request") {
            Ok(request) => request.reason() == "Probe request",
            Err(_) => false,
        };
    request_matches && ProbeTrackingBackend.status() == AppTrackingAuthorizationStatus::Authorized
}

struct ProbeTrackingBackend;

impl AppTrackingAuthorizationBackend for ProbeTrackingBackend {
    fn status(&self) -> AppTrackingAuthorizationStatus {
        AppTrackingAuthorizationStatus::Authorized
    }
}

fn check_framework_notifications() -> bool {
    matches!(
        NotificationId::new(alloc_crate::string::String::new()),
        Err(NotificationError::InvalidIdentifier)
    )
}

fn check_framework_location() -> bool {
    match Coordinate::new(0.0, 0.0) {
        Ok(coordinate) => coordinate.latitude_degrees() == 0.0,
        Err(_) => false,
    }
}

fn check_framework_motion() -> bool {
    match Acceleration::new(0.0, 0.0, 0.0) {
        Ok(acceleration) => acceleration.x_meters_per_second_squared() == 0.0,
        Err(_) => false,
    }
}

fn check_framework_sharing() -> bool {
    ShareItem::text(alloc_crate::string::String::new())
        .as_str()
        .is_empty()
}

fn check_framework_ui() -> bool {
    match Frame::new(0.0, 0.0, 1.0, 1.0) {
        Ok(frame) => frame.width() == 1.0 && frame.height() == 1.0,
        Err(_) => false,
    }
}
