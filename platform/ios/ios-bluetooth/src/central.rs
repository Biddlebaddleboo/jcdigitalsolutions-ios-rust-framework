use core::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use crate::connection_policy::{
    ConnectionAction, ConnectionEffect, ConnectionPolicy, ManagerAvailability,
};
use crate::queue::{ConnectionQueue, DiscoveryQueue, ServiceDiscoveryQueue};
use crate::service_discovery_policy::ServiceDiscoveryPolicy;
use framework_bluetooth::{
    BluetoothCentralBackend, BluetoothConnectionBackend, BluetoothConnectionError,
    BluetoothConnectionEvent, BluetoothConnectionState, BluetoothDiscovery, BluetoothPeripheralId,
    BluetoothScanError, BluetoothScanState, BluetoothServiceDiscoveryBackend,
    BluetoothServiceDiscoveryError, BluetoothServiceDiscoveryEvent, BluetoothServiceDiscoveryState,
};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_bluetooth::{
    CBCentralManager, CBCentralManagerDelegate, CBManagerState, CBPeripheral, CBPeripheralDelegate,
    CBPeripheralState, CBService, CBUUID,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSUUID};

struct CentralState {
    requested: Cell<bool>,
    scan_state: Cell<BluetoothScanState>,
    manager_state: Cell<CBManagerState>,
    discoveries: RefCell<DiscoveryQueue>,
    connection: RefCell<ConnectionPolicy>,
    connection_events: RefCell<ConnectionQueue>,
    connection_peripheral: RefCell<Option<Retained<CBPeripheral>>>,
    service_discovery: RefCell<ServiceDiscoveryPolicy>,
    service_discovery_events: RefCell<ServiceDiscoveryQueue>,
}

impl CentralState {
    const fn new() -> Self {
        Self {
            requested: Cell::new(false),
            scan_state: Cell::new(BluetoothScanState::Stopped),
            manager_state: Cell::new(CBManagerState::Unknown),
            discoveries: RefCell::new(DiscoveryQueue::new()),
            connection: RefCell::new(ConnectionPolicy::new()),
            connection_events: RefCell::new(ConnectionQueue::new()),
            connection_peripheral: RefCell::new(None),
            service_discovery: RefCell::new(ServiceDiscoveryPolicy::new()),
            service_discovery_events: RefCell::new(ServiceDiscoveryQueue::new()),
        }
    }
}

struct DelegateIvars {
    state: Rc<CentralState>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateIvars]
    struct CentralDelegate;

    // SAFETY: NSObjectProtocol adds no methods; this class is created, retained, called, and
    // released on the main thread used by its CBCentralManager.
    unsafe impl NSObjectProtocol for CentralDelegate {}

    // SAFETY: CoreBluetooth delivers events on the main queue because the manager is initialized
    // with a nil dispatch queue. The backend retains this weakly referenced delegate, its state is
    // main-thread-only, and every callback contains Rust panics before returning to Objective-C.
    #[allow(non_snake_case)]
    unsafe impl CBCentralManagerDelegate for CentralDelegate {
        #[unsafe(method(centralManagerDidUpdateState:))]
        unsafe fn centralManagerDidUpdateState(&self, central: &CBCentralManager) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: CoreBluetooth invokes this method with its live central manager.
                let manager_state = unsafe { central.state() };
                self.ivars().state.manager_state.set(manager_state);
                update_scan_state(&self.ivars().state, central, manager_state);
                update_connection_state(&self.ivars().state, central, manager_state);
            }));
            if outcome.is_err() {
                self.ivars()
                    .state
                    .scan_state
                    .set(BluetoothScanState::Unavailable);
                contain_connection_panic(&self.ivars().state, central);
                contain_service_discovery_panic(&self.ivars().state);
            }
        }

        #[unsafe(method(centralManager:didDiscoverPeripheral:advertisementData:RSSI:))]
        unsafe fn centralManager_didDiscoverPeripheral_advertisementData_RSSI(
            &self,
            _central: &CBCentralManager,
            peripheral: &CBPeripheral,
            _advertisement_data: &objc2_foundation::NSDictionary<
                NSString,
                objc2::runtime::AnyObject,
            >,
            rssi: &objc2_foundation::NSNumber,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let state = &self.ivars().state;
                if !state.requested.get() {
                    return;
                }
                // SAFETY: CoreBluetooth supplies a live discovered CBPeripheral. Only its
                // documented persistent peer UUID is copied; the native object is not retained.
                let identifier: Retained<NSUUID> = unsafe { peripheral.identifier() };
                let Some(peripheral) = peripheral_id_from_native(&identifier) else {
                    state.scan_state.set(BluetoothScanState::Unavailable);
                    return;
                };
                let event = BluetoothDiscovery {
                    peripheral,
                    rssi_dbm: crate::conversion::rssi_dbm_from_native(rssi.integerValue()),
                };
                state.discoveries.borrow_mut().push(event);
            }));
            if outcome.is_err() {
                self.ivars()
                    .state
                    .scan_state
                    .set(BluetoothScanState::Unavailable);
            }
        }

        #[unsafe(method(centralManager:didConnectPeripheral:))]
        unsafe fn centralManager_didConnectPeripheral(
            &self,
            central: &CBCentralManager,
            peripheral: &CBPeripheral,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
                if let Some(id) = peripheral_id_from_native(&native_id) {
                    let state = &self.ivars().state;
                    let effect = state.connection.borrow_mut().did_connect(id);
                    apply_connection_effect(state, central, effect);
                    let accepted = {
                        let connection = state.connection.borrow();
                        connection.target() == Some(id)
                            && connection.state() == BluetoothConnectionState::Connected
                    };
                    if accepted {
                        let delegate = ProtocolObject::from_ref(self);
                        // SAFETY: The selected connection is accepted; the backend retains this
                        // main-thread delegate and clears it before releasing the peer.
                        unsafe { peripheral.setDelegate(Some(delegate)) };
                    }
                }
            }));
            if outcome.is_err() {
                contain_connection_panic(&self.ivars().state, central);
            }
        }

        #[unsafe(method(centralManager:didFailToConnectPeripheral:error:))]
        unsafe fn centralManager_didFailToConnectPeripheral_error(
            &self,
            central: &CBCentralManager,
            peripheral: &CBPeripheral,
            _error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
                if let Some(id) = peripheral_id_from_native(&native_id) {
                    let state = &self.ivars().state;
                    let selected = state.connection.borrow().target() == Some(id);
                    let effect = state.connection.borrow_mut().did_fail_to_connect(id);
                    if selected {
                        terminate_service_discovery(
                            state,
                            id,
                            BluetoothServiceDiscoveryError::Disconnected,
                        );
                        // SAFETY: A selected peer's failed native connection cannot produce
                        // later service events; remove the weak delegate before releasing it.
                        unsafe { peripheral.setDelegate(None) };
                    }
                    apply_connection_effect(state, central, effect);
                    release_terminal_peripheral(state);
                }
            }));
            if outcome.is_err() {
                contain_connection_panic(&self.ivars().state, central);
            }
        }

        #[unsafe(method(centralManager:didDisconnectPeripheral:error:))]
        unsafe fn centralManager_didDisconnectPeripheral_error(
            &self,
            central: &CBCentralManager,
            peripheral: &CBPeripheral,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
                if let Some(id) = peripheral_id_from_native(&native_id) {
                    let state = &self.ivars().state;
                    let selected = state.connection.borrow().target() == Some(id);
                    let effect = state
                        .connection
                        .borrow_mut()
                        .did_disconnect(id, error.is_some());
                    if selected {
                        terminate_service_discovery(
                            state,
                            id,
                            BluetoothServiceDiscoveryError::Disconnected,
                        );
                        // SAFETY: This is the selected peer's terminal central callback; its
                        // service delegate is no longer needed after this point.
                        unsafe { peripheral.setDelegate(None) };
                    }
                    apply_connection_effect(state, central, effect);
                    release_terminal_peripheral(state);
                }
            }));
            if outcome.is_err() {
                contain_connection_panic(&self.ivars().state, central);
            }
        }
    }

    // SAFETY: CoreBluetooth invokes this callback on the same main queue as the central
    // delegate after an explicit request on the selected, connected peer. Native service objects
    // are used only for the callback duration and copied to fixed-width Rust values.
    #[allow(non_snake_case)]
    unsafe impl CBPeripheralDelegate for CentralDelegate {
        #[unsafe(method(peripheral:didDiscoverServices:))]
        unsafe fn peripheral_didDiscoverServices(
            &self,
            peripheral: &CBPeripheral,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                handle_service_discovery(&self.ivars().state, peripheral, error.is_some());
            }));
            if outcome.is_err() {
                contain_service_discovery_panic(&self.ivars().state);
            }
        }
    }
);

impl CentralDelegate {
    fn new(marker: MainThreadMarker, state: Rc<CentralState>) -> Retained<Self> {
        let allocated: Allocated<Self> = marker.alloc::<Self>();
        let allocated = allocated.set_ivars(DelegateIvars { state });
        // SAFETY: The class derives directly from NSObject and its ivars are initialized before
        // invoking NSObject's designated initializer.
        unsafe { msg_send![super(allocated), init] }
    }
}

/// Caller-owned main-thread CoreBluetooth central scan, connection, and service-discovery backend.
///
/// Constructing this value creates no native manager and requests no authorization. Calling
/// [`start_unfiltered_scan`](BluetoothCentralBackend::start_unfiltered_scan) or
/// [`connect_peripheral`](BluetoothConnectionBackend::connect_peripheral) lazily creates a
/// `CBCentralManager`; either explicit operation may cause the operating system to request
/// Bluetooth authorization. CoreBluetooth callbacks use the main queue, so this backend and all
/// its methods must stay on the supplied main thread. Discovery values use a 32-entry bounded
/// queue and connection events use a separate 16-entry bounded queue; overflow drops the newest
/// event and increments its saturating count. Discovery does not retain `CBPeripheral`; an
/// explicit connection retains at most one selected peer until its terminal callback or backend
/// teardown. Service discovery copies UUIDs into a separate 32-event queue and retains no service
/// objects. No characteristic or peripheral data read/write is exposed.
pub struct IosBluetoothCentralBackend {
    state: Rc<CentralState>,
    manager: Option<Retained<CBCentralManager>>,
    delegate: Option<Retained<CentralDelegate>>,
    _marker: MainThreadMarker,
}

impl IosBluetoothCentralBackend {
    /// Creates an idle central backend on the supplied main-thread context.
    pub fn new(marker: MainThreadMarker) -> Self {
        Self {
            state: Rc::new(CentralState::new()),
            manager: None,
            delegate: None,
            _marker: marker,
        }
    }

    fn create_manager(&mut self) {
        let delegate = CentralDelegate::new(self._marker, Rc::clone(&self.state));
        let delegate_object = ProtocolObject::from_ref(&*delegate);
        // SAFETY: The delegate is retained in this backend because CBCentralManager.delegate is
        // weak. A nil queue is documented to deliver callbacks on the main queue, matching the
        // MainThreadMarker and MainThreadOnly delegate. The explicit scan request is already set.
        let manager = unsafe {
            CBCentralManager::initWithDelegate_queue(
                CBCentralManager::alloc(),
                Some(delegate_object),
                None,
            )
        };
        self.delegate = Some(delegate);
        self.manager = Some(manager);

        let manager = self.manager.as_deref().expect("manager just initialized");
        // SAFETY: The manager belongs to this main-thread backend; its current state is read only
        // to handle initialization that completed before the initial delegate callback.
        let manager_state = unsafe { manager.state() };
        self.state.manager_state.set(manager_state);
        update_scan_state(&self.state, manager, manager_state);
        update_connection_state(&self.state, manager, manager_state);
    }
}

impl BluetoothCentralBackend for IosBluetoothCentralBackend {
    fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError> {
        if self.state.requested.replace(true) {
            return Err(BluetoothScanError::AlreadyActive);
        }
        self.state.scan_state.set(BluetoothScanState::Starting);
        if self.manager.is_none() {
            self.create_manager();
        } else if let Some(manager) = self.manager.as_deref() {
            // SAFETY: The manager belongs to this main-thread backend; query its current state
            // instead of relying on a possibly stale callback value before issuing a scan.
            let manager_state = unsafe { manager.state() };
            self.state.manager_state.set(manager_state);
            update_scan_state(&self.state, manager, manager_state);
        }
        Ok(())
    }

    fn stop_scan(&mut self) {
        self.state.requested.set(false);
        self.state.scan_state.set(BluetoothScanState::Stopped);
        if let Some(manager) = self.manager.as_deref() {
            // SAFETY: This manager was created and is controlled on the backend's main thread.
            unsafe { manager.stopScan() };
        }
    }

    fn scan_state(&self) -> BluetoothScanState {
        self.state.scan_state.get()
    }

    fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery> {
        self.state.discoveries.borrow_mut().pop()
    }

    fn take_dropped_discovery_count(&mut self) -> u32 {
        self.state.discoveries.borrow_mut().take_dropped()
    }
}

impl BluetoothConnectionBackend for IosBluetoothCentralBackend {
    fn connect_peripheral(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> Result<(), BluetoothConnectionError> {
        let started = self.state.connection.borrow_mut().request(peripheral)?;
        enqueue_connection_event(&self.state, started.event);
        if self.manager.is_none() {
            self.create_manager();
        } else if let Some(manager) = self.manager.as_deref() {
            // SAFETY: The manager belongs to this main-thread backend; query its current state
            // rather than relying on a possibly stale delegate callback before lookup.
            let manager_state = unsafe { manager.state() };
            self.state.manager_state.set(manager_state);
            update_connection_state(&self.state, manager, manager_state);
        }
        Ok(())
    }

    fn cancel_connection(&mut self) {
        let discovering_peer = self.state.service_discovery.borrow().peripheral();
        if let Some(peripheral_id) = discovering_peer {
            if let Some(peripheral) = self.state.connection_peripheral.borrow().as_deref() {
                // SAFETY: This is the selected peer's explicit connection cancellation on the
                // backend's main thread; remove its weak delegate before the native cancel call.
                unsafe { peripheral.setDelegate(None) };
            }
            terminate_service_discovery(
                &self.state,
                peripheral_id,
                BluetoothServiceDiscoveryError::Disconnected,
            );
        }
        let effect = self.state.connection.borrow_mut().cancel();
        if let Some(manager) = self.manager.as_deref() {
            apply_connection_effect(&self.state, manager, effect);
        } else {
            enqueue_connection_event(&self.state, effect.event);
            release_terminal_peripheral(&self.state);
        }
    }

    fn connection_state(&self) -> BluetoothConnectionState {
        self.state.connection.borrow().state()
    }

    fn try_next_connection_event(&mut self) -> Option<BluetoothConnectionEvent> {
        self.state.connection_events.borrow_mut().pop()
    }

    fn take_dropped_connection_event_count(&mut self) -> u32 {
        self.state.connection_events.borrow_mut().take_dropped()
    }
}

impl BluetoothServiceDiscoveryBackend for IosBluetoothCentralBackend {
    fn discover_services(&mut self) -> Result<(), BluetoothServiceDiscoveryError> {
        let (connection_state, peripheral_id) = {
            let connection = self.state.connection.borrow();
            (connection.state(), connection.target())
        };
        let Some(peripheral_id) = peripheral_id else {
            return Err(BluetoothServiceDiscoveryError::NotConnected);
        };
        let connection_peripheral = self.state.connection_peripheral.borrow();
        let Some(peripheral) = connection_peripheral.as_deref() else {
            return Err(BluetoothServiceDiscoveryError::NotConnected);
        };
        // SAFETY: The peer was retrieved and retained by this backend; its state and identity are
        // queried on the required main thread before starting service discovery.
        let native_connected = unsafe { peripheral.state() } == CBPeripheralState::Connected;
        let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
        if peripheral_id_from_native(&native_id) != Some(peripheral_id) {
            return Err(BluetoothServiceDiscoveryError::NotConnected);
        }
        self.state.service_discovery.borrow_mut().request(
            peripheral_id,
            connection_state,
            native_connected,
        )?;
        let Some(delegate) = self.delegate.as_deref() else {
            let event = self
                .state
                .service_discovery
                .borrow_mut()
                .fail(peripheral_id, BluetoothServiceDiscoveryError::Unavailable);
            enqueue_service_discovery_event(&self.state, event);
            return Err(BluetoothServiceDiscoveryError::Unavailable);
        };
        let delegate = ProtocolObject::from_ref(delegate);
        // SAFETY: The selected connected peer and retained delegate are both owned by this
        // main-thread backend; CoreBluetooth stores the delegate weakly and discovery is explicit.
        unsafe {
            peripheral.setDelegate(Some(delegate));
            peripheral.discoverServices(None);
        }
        Ok(())
    }

    fn service_discovery_state(&self) -> BluetoothServiceDiscoveryState {
        self.state.service_discovery.borrow().state()
    }

    fn try_next_service_discovery_event(&mut self) -> Option<BluetoothServiceDiscoveryEvent> {
        self.state.service_discovery_events.borrow_mut().pop()
    }

    fn take_dropped_service_discovery_event_count(&mut self) -> u32 {
        self.state
            .service_discovery_events
            .borrow_mut()
            .take_dropped()
    }
}

impl Drop for IosBluetoothCentralBackend {
    fn drop(&mut self) {
        self.state.requested.set(false);
        let teardown = self.state.connection.borrow_mut().teardown();
        if let Some(manager) = self.manager.as_deref() {
            // SAFETY: Drop runs on the same main thread enforced by the stored marker. Stopping
            // the scan, cancelling the selected peer, and clearing CoreBluetooth's weak delegate
            // prevent callbacks from using backend state after its retained fields are released.
            unsafe {
                manager.stopScan();
                if teardown != ConnectionAction::None {
                    if let Some(peripheral) = self.state.connection_peripheral.borrow().as_deref() {
                        manager.cancelPeripheralConnection(peripheral);
                    }
                }
                if let Some(peripheral) = self.state.connection_peripheral.borrow().as_deref() {
                    peripheral.setDelegate(None);
                }
                manager.setDelegate(None);
            }
        }
        self.state.connection_peripheral.borrow_mut().take();
    }
}

fn update_connection_state(
    state: &CentralState,
    central: &CBCentralManager,
    manager_state: CBManagerState,
) {
    let availability = match manager_state {
        CBManagerState::PoweredOn => ManagerAvailability::PoweredOn,
        CBManagerState::PoweredOff => ManagerAvailability::PoweredOff,
        CBManagerState::Unauthorized => ManagerAvailability::Unauthorized,
        CBManagerState::Unsupported => ManagerAvailability::Unsupported,
        CBManagerState::Resetting | CBManagerState::Unknown => ManagerAvailability::Other,
        _ => ManagerAvailability::Other,
    };
    let effect = state.connection.borrow_mut().manager_update(availability);
    apply_connection_effect(state, central, effect);
}

fn apply_connection_effect(
    state: &CentralState,
    central: &CBCentralManager,
    effect: ConnectionEffect,
) {
    enqueue_connection_event(state, effect.event);
    match effect.action {
        ConnectionAction::None => release_terminal_peripheral(state),
        ConnectionAction::Lookup(id) => lookup_peripheral(state, central, id),
        ConnectionAction::Connect(id) => {
            if let Some(peripheral) = state.connection_peripheral.borrow().as_deref() {
                // SAFETY: The retained peripheral was retrieved for this caller-selected UUID,
                // and both it and the manager are used on the backend's main thread.
                unsafe { central.connectPeripheral_options(peripheral, None) };
            } else {
                let failure = state.connection.borrow_mut().native_unavailable(id);
                enqueue_connection_event(state, failure.event);
            }
        }
        ConnectionAction::Cancel(id) => {
            if let Some(peripheral) = state.connection_peripheral.borrow().as_deref() {
                // SAFETY: This is the retained peer for the active request and the manager is
                // called on the same main thread. Cancellation is nonblocking.
                unsafe { central.cancelPeripheralConnection(peripheral) };
            } else {
                let failure = state.connection.borrow_mut().did_disconnect(id, false);
                enqueue_connection_event(state, failure.event);
            }
        }
    }
}

fn lookup_peripheral(state: &CentralState, central: &CBCentralManager, id: BluetoothPeripheralId) {
    let string_bytes = crate::conversion::uuid_ascii_from_id(id);
    let uuid_text = core::str::from_utf8(&string_bytes).expect("UUID encoder emits ASCII");
    let uuid_string = NSString::from_str(uuid_text);
    let Some(identifier) = NSUUID::from_string(&uuid_string) else {
        let effect = state.connection.borrow_mut().lookup_result(id, false);
        apply_connection_effect(state, central, effect);
        return;
    };
    let identifiers: Retained<NSArray<NSUUID>> = NSArray::from_slice(&[identifier.as_ref()]);
    // SAFETY: The array contains one valid, public NSUUID for the caller's peer ID.
    let peers = unsafe { central.retrievePeripheralsWithIdentifiers(&identifiers) };
    let mut found = None;
    for index in 0..peers.count() {
        let candidate = peers.objectAtIndex(index);
        // SAFETY: CoreBluetooth returned this live peer object from its identifier lookup.
        let native_id: Retained<NSUUID> = unsafe { candidate.identifier() };
        if peripheral_id_from_native(&native_id) == Some(id) {
            found = Some(candidate);
            break;
        }
    }
    if let Some(peripheral) = found {
        *state.connection_peripheral.borrow_mut() = Some(peripheral);
        let effect = state.connection.borrow_mut().lookup_result(id, true);
        apply_connection_effect(state, central, effect);
    } else {
        let effect = state.connection.borrow_mut().lookup_result(id, false);
        apply_connection_effect(state, central, effect);
    }
}

fn enqueue_connection_event(state: &CentralState, event: Option<BluetoothConnectionEvent>) {
    if let Some(event) = event {
        state.connection_events.borrow_mut().push(event);
    }
}

fn release_terminal_peripheral(state: &CentralState) {
    if state.connection.borrow().target().is_none() {
        state.connection_peripheral.borrow_mut().take();
    }
}

fn contain_connection_panic(state: &CentralState, central: &CBCentralManager) {
    let effect = state.connection.borrow_mut().callback_panicked();
    apply_connection_effect(state, central, effect);
}

fn handle_service_discovery(state: &CentralState, peripheral: &CBPeripheral, had_error: bool) {
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        return;
    };
    if !state
        .service_discovery
        .borrow()
        .is_active_for(peripheral_id)
        || state.connection.borrow().target() != Some(peripheral_id)
    {
        return;
    }
    // SAFETY: CoreBluetooth supplies the selected live peripheral on the main queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        terminate_service_discovery(
            state,
            peripheral_id,
            BluetoothServiceDiscoveryError::Disconnected,
        );
        return;
    }
    if had_error {
        terminate_service_discovery(
            state,
            peripheral_id,
            BluetoothServiceDiscoveryError::NativeFailure,
        );
        return;
    }
    // SAFETY: The `didDiscoverServices` callback reports completion; CoreBluetooth owns this
    // array for the duration of the callback and each service UUID is copied immediately.
    let Some(services) = (unsafe { peripheral.services() }) else {
        terminate_service_discovery(
            state,
            peripheral_id,
            BluetoothServiceDiscoveryError::Unavailable,
        );
        return;
    };
    for index in 0..services.count() {
        let service: Retained<CBService> = services.objectAtIndex(index);
        // SAFETY: CoreBluetooth returned a live discovered CBService from this peripheral.
        let uuid: Retained<CBUUID> = unsafe { service.UUID() };
        let Some(service) = service_uuid_from_native(&uuid) else {
            terminate_service_discovery(
                state,
                peripheral_id,
                BluetoothServiceDiscoveryError::Unavailable,
            );
            return;
        };
        state.service_discovery_events.borrow_mut().push(
            BluetoothServiceDiscoveryEvent::ServiceDiscovered {
                peripheral: peripheral_id,
                service,
            },
        );
    }
    let completed = state.service_discovery.borrow_mut().complete(peripheral_id);
    enqueue_service_discovery_event(state, completed);
}

fn service_uuid_from_native(uuid: &CBUUID) -> Option<framework_bluetooth::BluetoothServiceUuid> {
    // SAFETY: CBUUID exposes a public NSString representation containing a 16-, 32-, or 128-bit
    // UUID. The converter rejects non-ASCII, invalid widths, and malformed hyphens.
    let string = unsafe { uuid.UUIDString() };
    let length = string.length();
    if length > 36 {
        return None;
    }
    let mut ascii = [0; 36];
    for (index, byte) in ascii[..length].iter_mut().enumerate() {
        let character = string.characterAtIndex(index as _);
        if character > 0x7f {
            return None;
        }
        *byte = character as u8;
    }
    crate::conversion::service_uuid_from_uuid_ascii(&ascii[..length])
}

fn enqueue_service_discovery_event(
    state: &CentralState,
    event: Option<BluetoothServiceDiscoveryEvent>,
) {
    if let Some(event) = event {
        state.service_discovery_events.borrow_mut().push(event);
    }
}

fn terminate_service_discovery(
    state: &CentralState,
    peripheral: BluetoothPeripheralId,
    error: BluetoothServiceDiscoveryError,
) {
    let event = state.service_discovery.borrow_mut().fail(peripheral, error);
    enqueue_service_discovery_event(state, event);
}

fn contain_service_discovery_panic(state: &CentralState) {
    if let Some(peripheral) = state.service_discovery.borrow().peripheral() {
        terminate_service_discovery(
            state,
            peripheral,
            BluetoothServiceDiscoveryError::Unavailable,
        );
    }
}

fn update_scan_state(
    state: &CentralState,
    central: &CBCentralManager,
    manager_state: CBManagerState,
) {
    if !state.requested.get() {
        state.scan_state.set(BluetoothScanState::Stopped);
        return;
    }

    match manager_state {
        CBManagerState::PoweredOn => {
            if state.scan_state.get() != BluetoothScanState::Scanning {
                // SAFETY: This call is made only after CoreBluetooth reports PoweredOn. Both
                // optional arguments are nil, requesting the documented unfiltered scan with
                // default duplicate filtering and no generic option dictionary.
                unsafe { central.scanForPeripheralsWithServices_options(None, None) };
            }
            state.scan_state.set(BluetoothScanState::Scanning);
        }
        CBManagerState::PoweredOff => {
            state.scan_state.set(BluetoothScanState::WaitingForPower);
        }
        CBManagerState::Resetting | CBManagerState::Unknown => {
            state.scan_state.set(BluetoothScanState::Starting);
        }
        CBManagerState::Unauthorized => {
            state.scan_state.set(BluetoothScanState::Unauthorized);
        }
        CBManagerState::Unsupported => {
            state.scan_state.set(BluetoothScanState::Unsupported);
        }
        _ => state.scan_state.set(BluetoothScanState::Unavailable),
    }
}

fn peripheral_id_from_native(identifier: &NSUUID) -> Option<BluetoothPeripheralId> {
    let uuid_string = identifier.UUIDString();
    if uuid_string.length() != 36 {
        return None;
    }
    let mut bytes = [0; 36];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let value = uuid_string.characterAtIndex(index as _);
        if value > 0x7f {
            return None;
        }
        *byte = value as u8;
    }
    crate::conversion::peripheral_id_from_uuid_ascii(&bytes)
}
