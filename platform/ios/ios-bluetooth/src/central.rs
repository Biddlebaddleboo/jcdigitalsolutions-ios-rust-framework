use core::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use crate::characteristic_notification_policy::{
    CharacteristicNotificationPolicy, NotificationDecision, NotificationRequest,
};
use crate::characteristic_read_policy::{CharacteristicReadPolicy, ReadDecision, ReadRequest};
use crate::characteristic_read_selection::{UniqueMatch, unique_match};
use crate::characteristic_write_policy::{CharacteristicWritePolicy, WriteDecision, WriteRequest};
use crate::connection_policy::{
    ConnectionAction, ConnectionEffect, ConnectionPolicy, ManagerAvailability,
};
use crate::queue::{
    CharacteristicNotificationQueue, CharacteristicReadQueue, CharacteristicWriteQueue,
    ConnectionQueue, DiscoveryQueue, ServiceDiscoveryQueue,
};
use crate::service_discovery_policy::ServiceDiscoveryPolicy;
use framework_bluetooth::{
    BluetoothCentralBackend, BluetoothCharacteristicNotificationBackend,
    BluetoothCharacteristicNotificationError, BluetoothCharacteristicNotificationEvent,
    BluetoothCharacteristicNotificationState, BluetoothCharacteristicReadBackend,
    BluetoothCharacteristicReadError, BluetoothCharacteristicReadEvent,
    BluetoothCharacteristicReadState, BluetoothCharacteristicUuid, BluetoothCharacteristicValue,
    BluetoothCharacteristicWriteBackend, BluetoothCharacteristicWriteError,
    BluetoothCharacteristicWriteEvent, BluetoothCharacteristicWriteState,
    BluetoothConnectionBackend, BluetoothConnectionError, BluetoothConnectionEvent,
    BluetoothConnectionState, BluetoothDiscovery, BluetoothPeripheralId, BluetoothScanError,
    BluetoothScanState, BluetoothServiceDiscoveryBackend, BluetoothServiceDiscoveryError,
    BluetoothServiceDiscoveryEvent, BluetoothServiceDiscoveryState, BluetoothServiceUuid,
};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_core_bluetooth::{
    CBCentralManager, CBCentralManagerDelegate, CBCharacteristic, CBCharacteristicProperties,
    CBCharacteristicWriteType, CBManagerState, CBPeripheral, CBPeripheralDelegate,
    CBPeripheralState, CBService, CBUUID,
};
use objc2_foundation::{NSArray, NSData, NSObject, NSObjectProtocol, NSString, NSUUID};

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
    characteristic_read: RefCell<CharacteristicReadPolicy>,
    characteristic_read_events: RefCell<CharacteristicReadQueue>,
    characteristic_read_service: RefCell<Option<Retained<CBService>>>,
    characteristic_read_characteristic: RefCell<Option<Retained<CBCharacteristic>>>,
    characteristic_write: RefCell<CharacteristicWritePolicy>,
    characteristic_write_events: RefCell<CharacteristicWriteQueue>,
    characteristic_write_service: RefCell<Option<Retained<CBService>>>,
    characteristic_write_characteristic: RefCell<Option<Retained<CBCharacteristic>>>,
    characteristic_write_data: RefCell<Option<Retained<NSData>>>,
    characteristic_notification: RefCell<CharacteristicNotificationPolicy>,
    characteristic_notification_events: RefCell<CharacteristicNotificationQueue>,
    characteristic_notification_service: RefCell<Option<Retained<CBService>>>,
    characteristic_notification_characteristic: RefCell<Option<Retained<CBCharacteristic>>>,
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
            characteristic_read: RefCell::new(CharacteristicReadPolicy::new()),
            characteristic_read_events: RefCell::new(CharacteristicReadQueue::new()),
            characteristic_read_service: RefCell::new(None),
            characteristic_read_characteristic: RefCell::new(None),
            characteristic_write: RefCell::new(CharacteristicWritePolicy::new()),
            characteristic_write_events: RefCell::new(CharacteristicWriteQueue::new()),
            characteristic_write_service: RefCell::new(None),
            characteristic_write_characteristic: RefCell::new(None),
            characteristic_write_data: RefCell::new(None),
            characteristic_notification: RefCell::new(CharacteristicNotificationPolicy::new()),
            characteristic_notification_events: RefCell::new(CharacteristicNotificationQueue::new()),
            characteristic_notification_service: RefCell::new(None),
            characteristic_notification_characteristic: RefCell::new(None),
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
                        state.service_discovery.borrow_mut().invalidate(id);
                        terminate_service_discovery(
                            state,
                            id,
                            BluetoothServiceDiscoveryError::Disconnected,
                        );
                        terminate_characteristic_read(
                            state,
                            id,
                            BluetoothCharacteristicReadError::Disconnected,
                        );
                        terminate_characteristic_write(
                            state,
                            id,
                            BluetoothCharacteristicWriteError::Disconnected,
                        );
                        terminate_characteristic_notification(
                            state,
                            id,
                            BluetoothCharacteristicNotificationError::Disconnected,
                        );
                        // SAFETY: A selected peer's failed native connection cannot produce
                        // later service events; remove the weak delegate before releasing it.
                        unsafe { peripheral.setDelegate(None) };
                        clear_quarantined_characteristic_write_objects(state);
                        clear_quarantined_characteristic_notification_objects(state);
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
                        state.service_discovery.borrow_mut().invalidate(id);
                        terminate_service_discovery(
                            state,
                            id,
                            BluetoothServiceDiscoveryError::Disconnected,
                        );
                        terminate_characteristic_read(
                            state,
                            id,
                            BluetoothCharacteristicReadError::Disconnected,
                        );
                        terminate_characteristic_write(
                            state,
                            id,
                            BluetoothCharacteristicWriteError::Disconnected,
                        );
                        terminate_characteristic_notification(
                            state,
                            id,
                            BluetoothCharacteristicNotificationError::Disconnected,
                        );
                        // SAFETY: This is the selected peer's terminal central callback; its
                        // service delegate is no longer needed after this point.
                        unsafe { peripheral.setDelegate(None) };
                        clear_quarantined_characteristic_write_objects(state);
                        clear_quarantined_characteristic_notification_objects(state);
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

        #[unsafe(method(peripheral:didDiscoverCharacteristicsForService:error:))]
        unsafe fn peripheral_didDiscoverCharacteristicsForService_error(
            &self,
            peripheral: &CBPeripheral,
            service: &CBService,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                handle_characteristic_discovery(
                    &self.ivars().state,
                    self,
                    peripheral,
                    service,
                    error.is_some(),
                );
            }));
            if outcome.is_err() {
                contain_characteristic_operation_panic(&self.ivars().state);
            }
        }

        #[unsafe(method(peripheral:didUpdateValueForCharacteristic:error:))]
        unsafe fn peripheral_didUpdateValueForCharacteristic_error(
            &self,
            peripheral: &CBPeripheral,
            characteristic: &CBCharacteristic,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                handle_characteristic_value(
                    &self.ivars().state,
                    peripheral,
                    characteristic,
                    error.is_some(),
                );
            }));
            if outcome.is_err() {
                contain_characteristic_operation_panic(&self.ivars().state);
            }
        }

        #[unsafe(method(peripheral:didUpdateNotificationStateForCharacteristic:error:))]
        unsafe fn peripheral_didUpdateNotificationStateForCharacteristic_error(
            &self,
            peripheral: &CBPeripheral,
            characteristic: &CBCharacteristic,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                handle_characteristic_notification_state(
                    &self.ivars().state,
                    peripheral,
                    characteristic,
                    error.is_some(),
                );
            }));
            if outcome.is_err() {
                contain_characteristic_notification_panic(&self.ivars().state);
            }
        }

        #[unsafe(method(peripheral:didWriteValueForCharacteristic:error:))]
        unsafe fn peripheral_didWriteValueForCharacteristic_error(
            &self,
            peripheral: &CBPeripheral,
            characteristic: &CBCharacteristic,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                handle_characteristic_write(
                    &self.ivars().state,
                    peripheral,
                    characteristic,
                    error.is_some(),
                );
            }));
            if outcome.is_err() {
                contain_characteristic_write_panic(&self.ivars().state);
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

/// Caller-owned main-thread CoreBluetooth central scan, connection, and GATT backend.
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
/// objects. A characteristic read or write is limited to one UUID-unique operation at a time;
/// read results and write completion events use separate four-entry FIFOs. Writes use only the
/// public write-with-response operation and retain copied bytes until its terminal callback.
/// Notifications and peripheral-role APIs are not exposed.
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
        self.state
            .service_discovery
            .borrow_mut()
            .invalidate(peripheral);
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
        let connection_peer = self.state.connection.borrow().target();
        let discovering_peer = self.state.service_discovery.borrow().peripheral();
        let reading_peer = self.state.characteristic_read.borrow().peripheral();
        let writing_peer = self.state.characteristic_write.borrow().peripheral();
        let notification_peer = self.state.characteristic_notification.borrow().peripheral();
        if let Some(peripheral) = connection_peer {
            self.state
                .service_discovery
                .borrow_mut()
                .invalidate(peripheral);
        }
        if discovering_peer.is_some()
            || reading_peer.is_some()
            || writing_peer.is_some()
            || notification_peer.is_some()
        {
            if let Some(peripheral) = self.state.connection_peripheral.borrow().as_deref() {
                // SAFETY: This is the selected peer's explicit connection cancellation on the
                // backend's main thread; remove its weak delegate before the native cancel call.
                unsafe { peripheral.setDelegate(None) };
            }
        }
        if let Some(peripheral_id) = discovering_peer {
            terminate_service_discovery(
                &self.state,
                peripheral_id,
                BluetoothServiceDiscoveryError::Disconnected,
            );
        }
        if let Some(peripheral_id) = reading_peer {
            terminate_characteristic_read(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicReadError::Disconnected,
            );
        }
        if let Some(peripheral_id) = writing_peer {
            terminate_characteristic_write(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicWriteError::Disconnected,
            );
        }
        if let Some(peripheral_id) = notification_peer {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Disconnected,
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
        if self.state.characteristic_read.borrow().is_active()
            || self.state.characteristic_write.borrow().is_active()
            || self.state.characteristic_notification.borrow().is_active()
        {
            return Err(BluetoothServiceDiscoveryError::AlreadyActive);
        }
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

impl BluetoothCharacteristicReadBackend for IosBluetoothCentralBackend {
    fn read_characteristic(
        &mut self,
        service_uuid: BluetoothServiceUuid,
        characteristic_uuid: BluetoothCharacteristicUuid,
    ) -> Result<(), BluetoothCharacteristicReadError> {
        if self
            .state
            .characteristic_notification
            .borrow()
            .callback_ambiguous()
        {
            return Err(BluetoothCharacteristicReadError::Unavailable);
        }
        if self.state.characteristic_read.borrow().is_active()
            || self.state.characteristic_write.borrow().is_active()
            || self.state.characteristic_notification.borrow().is_active()
            || self.state.service_discovery.borrow().state()
                == BluetoothServiceDiscoveryState::Discovering
        {
            return Err(BluetoothCharacteristicReadError::AlreadyActive);
        }
        let (connection_state, peripheral_id) = {
            let connection = self.state.connection.borrow();
            (connection.state(), connection.target())
        };
        let Some(peripheral_id) = peripheral_id else {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        };
        let connection_peripheral = self.state.connection_peripheral.borrow();
        let Some(peripheral) = connection_peripheral.as_deref() else {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        };
        // SAFETY: The peer is retained by this backend and queried on its required main thread.
        let native_connected = unsafe { peripheral.state() } == CBPeripheralState::Connected;
        let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
        if peripheral_id_from_native(&native_id) != Some(peripheral_id) {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        }
        let Some(manager) = self.manager.as_deref() else {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        };
        // SAFETY: This manager was created by this backend and is used on the same main thread.
        let manager_state = unsafe { manager.state() };
        if manager_state == CBManagerState::Unsupported {
            return Err(BluetoothCharacteristicReadError::Unsupported);
        }
        if manager_state != CBManagerState::PoweredOn {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        }
        // SAFETY: Service discovery completed for this peer; CoreBluetooth owns this live service
        // array and the selected objects remain retained by the peripheral.
        let Some(services) = (unsafe { peripheral.services() }) else {
            return Err(BluetoothCharacteristicReadError::ServiceDiscoveryRequired);
        };
        let service_match = unique_match(services.count(), |index| {
            let service = services.objectAtIndex(index);
            // SAFETY: This is a live CBService returned by the selected peripheral.
            let native_uuid: Retained<CBUUID> = unsafe { service.UUID() };
            service_uuid_from_native(&native_uuid)
                .map(|uuid| uuid == service_uuid)
                .ok_or(())
        });
        let (service_state, service_discovered_for_peer) = {
            let discovery = self.state.service_discovery.borrow();
            (discovery.state(), discovery.completed_for(peripheral_id))
        };
        let service_index = self
            .state
            .characteristic_read
            .borrow_mut()
            .begin(ReadRequest {
                peripheral: peripheral_id,
                service: service_uuid,
                characteristic: characteristic_uuid,
                connection_state,
                native_connected,
                manager_supported: true,
                service_state,
                service_discovered_for_peer,
                service_match,
            })?;
        let service = services.objectAtIndex(service_index);
        *self.state.characteristic_read_service.borrow_mut() = Some(service.clone());
        let Some(delegate) = self.delegate.as_deref() else {
            terminate_characteristic_read(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicReadError::Unavailable,
            );
            return Err(BluetoothCharacteristicReadError::Unavailable);
        };
        let delegate_object = ProtocolObject::from_ref(delegate);
        // SAFETY: The selected peer is connected, B592 discovery completed, exactly one target
        // service was found, and CoreBluetooth retains the weak delegate in this backend.
        unsafe {
            peripheral.setDelegate(Some(delegate_object));
            peripheral.discoverCharacteristics_forService(None, &service);
        }
        Ok(())
    }

    fn characteristic_read_state(&self) -> BluetoothCharacteristicReadState {
        self.state.characteristic_read.borrow().state()
    }

    fn try_next_characteristic_read_event(&mut self) -> Option<BluetoothCharacteristicReadEvent> {
        self.state.characteristic_read_events.borrow_mut().pop()
    }

    fn take_dropped_characteristic_read_event_count(&mut self) -> u32 {
        self.state
            .characteristic_read_events
            .borrow_mut()
            .take_dropped()
    }
}

impl BluetoothCharacteristicWriteBackend for IosBluetoothCentralBackend {
    fn write_characteristic(
        &mut self,
        service_uuid: BluetoothServiceUuid,
        characteristic_uuid: BluetoothCharacteristicUuid,
        value: &[u8],
    ) -> Result<(), BluetoothCharacteristicWriteError> {
        if self.state.characteristic_read.borrow().is_active()
            || self.state.characteristic_write.borrow().is_active()
            || self
                .state
                .characteristic_notification
                .borrow()
                .is_transitioning()
            || self.state.service_discovery.borrow().state()
                == BluetoothServiceDiscoveryState::Discovering
        {
            return Err(BluetoothCharacteristicWriteError::AlreadyActive);
        }
        if value.len() > framework_bluetooth::BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES {
            return Err(BluetoothCharacteristicWriteError::PayloadTooLarge);
        }
        let (connection_state, peripheral_id) = {
            let connection = self.state.connection.borrow();
            (connection.state(), connection.target())
        };
        let Some(peripheral_id) = peripheral_id else {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        };
        let connection_peripheral = self.state.connection_peripheral.borrow();
        let Some(peripheral) = connection_peripheral.as_deref() else {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        };
        // SAFETY: The peer is retained by this backend and queried on its required main thread.
        let native_connected = unsafe { peripheral.state() } == CBPeripheralState::Connected;
        let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
        if peripheral_id_from_native(&native_id) != Some(peripheral_id) {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        }
        let Some(manager) = self.manager.as_deref() else {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        };
        // SAFETY: This manager was created by this backend and is used on the same main thread.
        let manager_state = unsafe { manager.state() };
        if manager_state == CBManagerState::Unsupported {
            return Err(BluetoothCharacteristicWriteError::Unsupported);
        }
        if manager_state != CBManagerState::PoweredOn {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        }
        let maximum_write_len = if peripheral
            .respondsToSelector(sel!(maximumWriteValueLengthForType:))
        {
            // SAFETY: The selector is available from iOS 9.0; respondsToSelector: guards the
            // existing iOS 8.0 package floor. CoreBluetooth reports the current negotiated limit.
            unsafe {
                peripheral.maximumWriteValueLengthForType(CBCharacteristicWriteType::WithResponse)
            }
        } else {
            // The ATT minimum MTU is 23 bytes, leaving at most 20 value bytes after ATT headers.
            20
        };
        let Some(services) = (unsafe { peripheral.services() }) else {
            return Err(BluetoothCharacteristicWriteError::ServiceDiscoveryRequired);
        };
        let service_match = unique_match(services.count(), |index| {
            let service = services.objectAtIndex(index);
            // SAFETY: This is a live CBService returned by the selected peripheral.
            let native_uuid: Retained<CBUUID> = unsafe { service.UUID() };
            service_uuid_from_native(&native_uuid)
                .map(|uuid| uuid == service_uuid)
                .ok_or(())
        });
        let (service_state, service_discovered_for_peer) = {
            let discovery = self.state.service_discovery.borrow();
            (discovery.state(), discovery.completed_for(peripheral_id))
        };
        let service_index = self
            .state
            .characteristic_write
            .borrow_mut()
            .begin(WriteRequest {
                peripheral: peripheral_id,
                service: service_uuid,
                characteristic: characteristic_uuid,
                connection_state,
                native_connected,
                manager_supported: true,
                service_state,
                service_discovered_for_peer,
                service_match,
                payload_len: value.len(),
                maximum_write_len,
            })?;
        *self.state.characteristic_write_service.borrow_mut() =
            Some(services.objectAtIndex(service_index));
        *self.state.characteristic_write_data.borrow_mut() = Some(NSData::with_bytes(value));
        let Some(delegate) = self.delegate.as_deref() else {
            terminate_characteristic_write(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicWriteError::Unavailable,
            );
            return Err(BluetoothCharacteristicWriteError::Unavailable);
        };
        let delegate_object = ProtocolObject::from_ref(delegate);
        let selected_service = self
            .state
            .characteristic_write_service
            .borrow()
            .as_ref()
            .expect("selected write service is retained")
            .clone();
        // SAFETY: The selected peer is connected, B592 discovery completed, exactly one target
        // service was found, and CoreBluetooth retains this backend's main-thread delegate weakly.
        unsafe {
            peripheral.setDelegate(Some(delegate_object));
            peripheral.discoverCharacteristics_forService(None, &selected_service);
        }
        Ok(())
    }

    fn characteristic_write_state(&self) -> BluetoothCharacteristicWriteState {
        self.state.characteristic_write.borrow().state()
    }

    fn try_next_characteristic_write_event(&mut self) -> Option<BluetoothCharacteristicWriteEvent> {
        self.state.characteristic_write_events.borrow_mut().pop()
    }

    fn take_dropped_characteristic_write_event_count(&mut self) -> u32 {
        self.state
            .characteristic_write_events
            .borrow_mut()
            .take_dropped()
    }
}

impl BluetoothCharacteristicNotificationBackend for IosBluetoothCentralBackend {
    fn enable_characteristic_notifications(
        &mut self,
        service_uuid: BluetoothServiceUuid,
        characteristic_uuid: BluetoothCharacteristicUuid,
    ) -> Result<(), BluetoothCharacteristicNotificationError> {
        if self.state.characteristic_read.borrow().is_active()
            || self.state.characteristic_write.borrow().is_active()
            || self.state.characteristic_notification.borrow().is_active()
            || self.state.service_discovery.borrow().state()
                == BluetoothServiceDiscoveryState::Discovering
        {
            return Err(BluetoothCharacteristicNotificationError::AlreadyActive);
        }
        let (connection_state, peripheral_id) = {
            let connection = self.state.connection.borrow();
            (connection.state(), connection.target())
        };
        let Some(peripheral_id) = peripheral_id else {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        };
        let connection_peripheral = self.state.connection_peripheral.borrow();
        let Some(peripheral) = connection_peripheral.as_deref() else {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        };
        // SAFETY: The selected peer is retained by this backend and queried on its main thread.
        let native_connected = unsafe { peripheral.state() } == CBPeripheralState::Connected;
        let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
        if peripheral_id_from_native(&native_id) != Some(peripheral_id) {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        }
        let Some(manager) = self.manager.as_deref() else {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        };
        // SAFETY: This manager was created by this backend and is used on the same main thread.
        let manager_state = unsafe { manager.state() };
        if manager_state == CBManagerState::Unsupported {
            return Err(BluetoothCharacteristicNotificationError::Unsupported);
        }
        if manager_state != CBManagerState::PoweredOn {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        }
        // SAFETY: The selected peer's successful B592 service discovery owns this live array.
        let Some(services) = (unsafe { peripheral.services() }) else {
            return Err(BluetoothCharacteristicNotificationError::ServiceDiscoveryRequired);
        };
        let service_match = unique_match(services.count(), |index| {
            let service = services.objectAtIndex(index);
            // SAFETY: This is a live CBService returned by the selected peripheral.
            let native_uuid: Retained<CBUUID> = unsafe { service.UUID() };
            service_uuid_from_native(&native_uuid)
                .map(|uuid| uuid == service_uuid)
                .ok_or(())
        });
        let (service_state, service_discovered_for_peer) = {
            let discovery = self.state.service_discovery.borrow();
            (discovery.state(), discovery.completed_for(peripheral_id))
        };
        let service_index =
            self.state
                .characteristic_notification
                .borrow_mut()
                .begin(NotificationRequest {
                    peripheral: peripheral_id,
                    service: service_uuid,
                    characteristic: characteristic_uuid,
                    connection_state,
                    native_connected,
                    manager_supported: true,
                    service_state,
                    service_discovered_for_peer,
                    service_match,
                })?;
        *self.state.characteristic_notification_service.borrow_mut() =
            Some(services.objectAtIndex(service_index));
        let Some(delegate) = self.delegate.as_deref() else {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Unavailable,
            );
            return Err(BluetoothCharacteristicNotificationError::Unavailable);
        };
        let delegate_object = ProtocolObject::from_ref(delegate);
        let selected_service = self
            .state
            .characteristic_notification_service
            .borrow()
            .as_ref()
            .expect("selected notification service is retained")
            .clone();
        // SAFETY: The peer is connected, service discovery is peer-scoped, and the retained weak
        // delegate remains alive for this main-thread CoreBluetooth callback sequence.
        unsafe {
            peripheral.setDelegate(Some(delegate_object));
            peripheral.discoverCharacteristics_forService(None, &selected_service);
        }
        Ok(())
    }

    fn disable_characteristic_notifications(
        &mut self,
    ) -> Result<(), BluetoothCharacteristicNotificationError> {
        let (peripheral_id, _service_uuid, _characteristic_uuid) = self
            .state
            .characteristic_notification
            .borrow_mut()
            .begin_disable()?;
        let connection_peripheral = self.state.connection_peripheral.borrow();
        let Some(peripheral) = connection_peripheral.as_deref() else {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Disconnected,
            );
            return Err(BluetoothCharacteristicNotificationError::Disconnected);
        };
        // SAFETY: The retained peripheral is queried and mutated on the backend's main thread.
        let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
        if peripheral_id_from_native(&native_id) != Some(peripheral_id)
            || unsafe { peripheral.state() } != CBPeripheralState::Connected
        {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Disconnected,
            );
            return Err(BluetoothCharacteristicNotificationError::Disconnected);
        }
        let Some(delegate) = self.delegate.as_deref() else {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Unavailable,
            );
            return Err(BluetoothCharacteristicNotificationError::Unavailable);
        };
        let notification_characteristic = self
            .state
            .characteristic_notification_characteristic
            .borrow();
        let Some(characteristic) = notification_characteristic.as_deref() else {
            terminate_characteristic_notification(
                &self.state,
                peripheral_id,
                BluetoothCharacteristicNotificationError::Unavailable,
            );
            return Err(BluetoothCharacteristicNotificationError::Unavailable);
        };
        let delegate_object = ProtocolObject::from_ref(delegate);
        // SAFETY: The selected notify-capable characteristic is retained by this backend and
        // CoreBluetooth delivers the asynchronous state result through the retained delegate.
        unsafe {
            peripheral.setDelegate(Some(delegate_object));
            peripheral.setNotifyValue_forCharacteristic(false, characteristic);
        }
        Ok(())
    }

    fn characteristic_notification_state(&self) -> BluetoothCharacteristicNotificationState {
        self.state.characteristic_notification.borrow().state()
    }

    fn last_characteristic_notification_error(
        &self,
    ) -> Option<BluetoothCharacteristicNotificationError> {
        self.state.characteristic_notification.borrow().last_error()
    }

    fn try_next_characteristic_notification_event(
        &mut self,
    ) -> Option<BluetoothCharacteristicNotificationEvent> {
        self.state
            .characteristic_notification_events
            .borrow_mut()
            .pop()
    }

    fn take_dropped_characteristic_notification_event_count(&mut self) -> u32 {
        self.state
            .characteristic_notification_events
            .borrow_mut()
            .take_dropped()
    }
}

impl Drop for IosBluetoothCentralBackend {
    fn drop(&mut self) {
        self.state.requested.set(false);
        self.state.characteristic_read.borrow_mut().teardown();
        self.state.characteristic_write.borrow_mut().teardown();
        self.state
            .characteristic_notification
            .borrow_mut()
            .teardown();
        let teardown = self.state.connection.borrow_mut().teardown();
        if let Some(manager) = self.manager.as_deref() {
            // SAFETY: Drop runs on the same main thread enforced by the stored marker. Stopping
            // the scan, cancelling the selected peer, and clearing CoreBluetooth's weak delegate
            // prevent callbacks from using backend state after its retained fields are released.
            unsafe {
                manager.stopScan();
                if let (Some(peripheral), Some(characteristic)) = (
                    self.state.connection_peripheral.borrow().as_deref(),
                    self.state
                        .characteristic_notification_characteristic
                        .borrow()
                        .as_deref(),
                ) {
                    if peripheral.state() == CBPeripheralState::Connected
                        && characteristic.isNotifying()
                    {
                        peripheral.setNotifyValue_forCharacteristic(false, characteristic);
                    }
                }
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
        self.state.characteristic_read_service.borrow_mut().take();
        self.state
            .characteristic_read_characteristic
            .borrow_mut()
            .take();
        self.state.characteristic_write_service.borrow_mut().take();
        self.state
            .characteristic_write_characteristic
            .borrow_mut()
            .take();
        self.state.characteristic_write_data.borrow_mut().take();
        self.state
            .characteristic_notification_service
            .borrow_mut()
            .take();
        self.state
            .characteristic_notification_characteristic
            .borrow_mut()
            .take();
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
    if let Some(peripheral) = state.connection.borrow().target() {
        state.service_discovery.borrow_mut().invalidate(peripheral);
    }
    if let Some(peripheral) = state.service_discovery.borrow().peripheral() {
        terminate_service_discovery(
            state,
            peripheral,
            BluetoothServiceDiscoveryError::Unavailable,
        );
    }
    contain_characteristic_read_panic(state);
    contain_characteristic_write_panic(state);
    let effect = state.connection.borrow_mut().callback_panicked();
    apply_connection_effect(state, central, effect);
}

fn terminate_characteristic_read(
    state: &CentralState,
    peripheral: BluetoothPeripheralId,
    error: BluetoothCharacteristicReadError,
) {
    let decision = state
        .characteristic_read
        .borrow_mut()
        .terminate(peripheral, error);
    apply_characteristic_read_decision(state, decision, None);
}

fn contain_characteristic_read_panic(state: &CentralState) {
    let decision = state.characteristic_read.borrow_mut().callback_panicked();
    apply_characteristic_read_decision(state, decision, None);
}

fn terminate_characteristic_write(
    state: &CentralState,
    peripheral: BluetoothPeripheralId,
    error: BluetoothCharacteristicWriteError,
) {
    let decision = if error == BluetoothCharacteristicWriteError::Disconnected {
        state
            .characteristic_write
            .borrow_mut()
            .disconnected(peripheral)
    } else {
        state
            .characteristic_write
            .borrow_mut()
            .terminate(peripheral, error)
    };
    apply_characteristic_write_decision(state, decision);
}

fn terminate_characteristic_notification(
    state: &CentralState,
    peripheral: BluetoothPeripheralId,
    error: BluetoothCharacteristicNotificationError,
) {
    let decision = if error == BluetoothCharacteristicNotificationError::Disconnected {
        state
            .characteristic_notification
            .borrow_mut()
            .disconnected(peripheral)
    } else {
        state
            .characteristic_notification
            .borrow_mut()
            .terminate(peripheral, error)
    };
    apply_characteristic_notification_decision(state, decision);
}

fn contain_characteristic_write_panic(state: &CentralState) {
    let decision = state.characteristic_write.borrow_mut().callback_panicked();
    apply_characteristic_write_decision(state, decision);
}

fn contain_characteristic_notification_panic(state: &CentralState) {
    let decision = state
        .characteristic_notification
        .borrow_mut()
        .callback_panicked();
    apply_characteristic_notification_decision(state, decision);
}

fn contain_characteristic_operation_panic(state: &CentralState) {
    contain_characteristic_read_panic(state);
    contain_characteristic_write_panic(state);
    contain_characteristic_notification_panic(state);
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

fn handle_characteristic_discovery(
    state: &CentralState,
    delegate: &CentralDelegate,
    peripheral: &CBPeripheral,
    service: &CBService,
    had_error: bool,
) {
    let read_discovery = state.characteristic_read.borrow().state()
        == BluetoothCharacteristicReadState::DiscoveringCharacteristics;
    let write_discovery = state.characteristic_write.borrow().state()
        == BluetoothCharacteristicWriteState::DiscoveringCharacteristics;
    let notification_discovery = state.characteristic_notification.borrow().is_discovering();
    match (read_discovery, write_discovery, notification_discovery) {
        (true, false, false) => {
            handle_characteristic_read_discovery(state, delegate, peripheral, service, had_error)
        }
        (false, true, false) => {
            handle_characteristic_write_discovery(state, delegate, peripheral, service, had_error)
        }
        (false, false, true) => handle_characteristic_notification_discovery(
            state, delegate, peripheral, service, had_error,
        ),
        (false, false, false) => {}
        _ => contain_characteristic_operation_panic(state),
    }
}

fn handle_characteristic_read_discovery(
    state: &CentralState,
    delegate: &CentralDelegate,
    peripheral: &CBPeripheral,
    service: &CBService,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_read_panic(state);
        return;
    };
    let (expected_service, active) = {
        let read = state.characteristic_read.borrow();
        (
            read.service(),
            read.peripheral() == Some(peripheral_id) && read.is_active(),
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    // SAFETY: CoreBluetooth invokes this delegate on the configured main queue; do not accept
    // a stale notification-state result after the selected peripheral disconnects.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        let decision = state
            .characteristic_notification
            .borrow_mut()
            .disconnected(peripheral_id);
        apply_characteristic_notification_decision(state, decision);
        return;
    }
    let selected_service = state
        .characteristic_read_service
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, service));
    if !selected_service {
        return;
    }
    // SAFETY: CoreBluetooth supplies the selected live peripheral on its main callback queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        apply_characteristic_read_decision(
            state,
            state
                .characteristic_read
                .borrow_mut()
                .disconnected(peripheral_id),
            None,
        );
        return;
    }
    let Some(expected_service) = expected_service else {
        contain_characteristic_read_panic(state);
        return;
    };
    // SAFETY: The callback's CBService remains live for this invocation.
    let native_service_uuid: Retained<CBUUID> = unsafe { service.UUID() };
    if service_uuid_from_native(&native_service_uuid) != Some(expected_service) {
        contain_characteristic_read_panic(state);
        return;
    }
    if had_error {
        let decision = state
            .characteristic_read
            .borrow_mut()
            .characteristics_discovered(peripheral_id, true, Ok((UniqueMatch::Missing, false)));
        apply_characteristic_read_decision(state, decision, None);
        return;
    }
    // SAFETY: CoreBluetooth owns the service's discovered characteristic array; items are retained
    // while selected and the UUID/property snapshots are copied during this callback.
    let Some(characteristics) = (unsafe { service.characteristics() }) else {
        let decision = state
            .characteristic_read
            .borrow_mut()
            .characteristics_discovered(peripheral_id, false, Err(()));
        apply_characteristic_read_decision(state, decision, None);
        return;
    };
    let expected_characteristic = state.characteristic_read.borrow().characteristic();
    let Some(expected_characteristic) = expected_characteristic else {
        contain_characteristic_read_panic(state);
        return;
    };
    let selection = unique_match(characteristics.count(), |index| {
        let candidate = characteristics.objectAtIndex(index);
        // SAFETY: The characteristic is a live member of the selected service's native array.
        let native_uuid: Retained<CBUUID> = unsafe { candidate.UUID() };
        characteristic_uuid_from_native(&native_uuid)
            .map(|uuid| uuid == expected_characteristic)
            .ok_or(())
    });
    let selected = match selection {
        Ok(UniqueMatch::One(index)) => Some(characteristics.objectAtIndex(index)),
        _ => None,
    };
    let readable = selected.as_deref().is_some_and(|characteristic| {
        // SAFETY: This is the unique characteristic returned by CoreBluetooth for the service.
        unsafe { characteristic.properties() }.contains(CBCharacteristicProperties::Read)
    });
    let decision = state
        .characteristic_read
        .borrow_mut()
        .characteristics_discovered(peripheral_id, false, selection.map(|m| (m, readable)));
    match decision {
        ReadDecision::Read => {
            let Some(characteristic) = selected else {
                contain_characteristic_read_panic(state);
                return;
            };
            *state.characteristic_read_characteristic.borrow_mut() = Some(characteristic.clone());
            let delegate_object = ProtocolObject::from_ref(delegate);
            // SAFETY: One UUID-unique characteristic has the public Read property. The selected
            // peer/delegate and retained characteristic all share this main-thread lifetime.
            unsafe {
                peripheral.setDelegate(Some(delegate_object));
                peripheral.readValueForCharacteristic(&characteristic);
            }
        }
        decision => apply_characteristic_read_decision(state, decision, None),
    }
}

fn handle_characteristic_write_discovery(
    state: &CentralState,
    delegate: &CentralDelegate,
    peripheral: &CBPeripheral,
    service: &CBService,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_write_panic(state);
        return;
    };
    let (expected_service, active) = {
        let write = state.characteristic_write.borrow();
        (
            write.service(),
            write.peripheral() == Some(peripheral_id)
                && write.state() == BluetoothCharacteristicWriteState::DiscoveringCharacteristics,
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    let selected_service = state
        .characteristic_write_service
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, service));
    if !selected_service {
        return;
    }
    // SAFETY: CoreBluetooth supplies the selected live peripheral on its main callback queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        let decision = state
            .characteristic_write
            .borrow_mut()
            .disconnected(peripheral_id);
        apply_characteristic_write_decision(state, decision);
        return;
    }
    let Some(expected_service) = expected_service else {
        contain_characteristic_write_panic(state);
        return;
    };
    // SAFETY: The callback's CBService remains live for this invocation.
    let native_service_uuid: Retained<CBUUID> = unsafe { service.UUID() };
    if service_uuid_from_native(&native_service_uuid) != Some(expected_service) {
        contain_characteristic_write_panic(state);
        return;
    }
    if had_error {
        let decision = state
            .characteristic_write
            .borrow_mut()
            .characteristics_discovered(peripheral_id, true, Ok((UniqueMatch::Missing, false)));
        apply_characteristic_write_decision(state, decision);
        return;
    }
    // SAFETY: CoreBluetooth owns the service's discovered characteristic array; items are retained
    // while selected and UUID/property snapshots are copied during this callback.
    let Some(characteristics) = (unsafe { service.characteristics() }) else {
        let decision = state
            .characteristic_write
            .borrow_mut()
            .characteristics_discovered(peripheral_id, false, Err(()));
        apply_characteristic_write_decision(state, decision);
        return;
    };
    let expected_characteristic = state.characteristic_write.borrow().characteristic();
    let Some(expected_characteristic) = expected_characteristic else {
        contain_characteristic_write_panic(state);
        return;
    };
    let selection = unique_match(characteristics.count(), |index| {
        let candidate = characteristics.objectAtIndex(index);
        // SAFETY: The characteristic is a live member of the selected service's native array.
        let native_uuid: Retained<CBUUID> = unsafe { candidate.UUID() };
        characteristic_uuid_from_native(&native_uuid)
            .map(|uuid| uuid == expected_characteristic)
            .ok_or(())
    });
    let selected = match selection {
        Ok(UniqueMatch::One(index)) => Some(characteristics.objectAtIndex(index)),
        _ => None,
    };
    let writable = selected.as_deref().is_some_and(|characteristic| {
        // SAFETY: This is the unique characteristic returned by CoreBluetooth for the service.
        unsafe { characteristic.properties() }.contains(CBCharacteristicProperties::Write)
    });
    let decision = state
        .characteristic_write
        .borrow_mut()
        .characteristics_discovered(peripheral_id, false, selection.map(|m| (m, writable)));
    match decision {
        WriteDecision::Write => {
            let Some(characteristic) = selected else {
                contain_characteristic_write_panic(state);
                return;
            };
            let data = state.characteristic_write_data.borrow().clone();
            let Some(data) = data else {
                contain_characteristic_write_panic(state);
                return;
            };
            *state.characteristic_write_characteristic.borrow_mut() = Some(characteristic.clone());
            let delegate_object = ProtocolObject::from_ref(delegate);
            // SAFETY: One UUID-unique characteristic has the public Write property. The copied
            // NSData, peer, and callback delegate remain retained for the acknowledged operation.
            unsafe {
                peripheral.setDelegate(Some(delegate_object));
                peripheral.writeValue_forCharacteristic_type(
                    &data,
                    &characteristic,
                    CBCharacteristicWriteType::WithResponse,
                );
            }
        }
        decision => apply_characteristic_write_decision(state, decision),
    }
}

fn handle_characteristic_notification_discovery(
    state: &CentralState,
    delegate: &CentralDelegate,
    peripheral: &CBPeripheral,
    service: &CBService,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_notification_panic(state);
        return;
    };
    let (expected_service, active) = {
        let notification = state.characteristic_notification.borrow();
        (
            notification.service(),
            notification.peripheral() == Some(peripheral_id) && notification.is_discovering(),
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    let selected_service = state
        .characteristic_notification_service
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, service));
    if !selected_service {
        return;
    }
    // SAFETY: CoreBluetooth supplies the selected live peripheral on its main callback queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        let decision = state
            .characteristic_notification
            .borrow_mut()
            .disconnected(peripheral_id);
        apply_characteristic_notification_decision(state, decision);
        return;
    }
    let Some(expected_service) = expected_service else {
        contain_characteristic_notification_panic(state);
        return;
    };
    // SAFETY: The callback's CBService remains live for this invocation.
    let native_service_uuid: Retained<CBUUID> = unsafe { service.UUID() };
    if service_uuid_from_native(&native_service_uuid) != Some(expected_service) {
        contain_characteristic_notification_panic(state);
        return;
    }
    if had_error {
        let decision = state
            .characteristic_notification
            .borrow_mut()
            .characteristics_discovered(peripheral_id, true, Ok((UniqueMatch::Missing, false)));
        apply_characteristic_notification_decision(state, decision);
        return;
    }
    // SAFETY: CoreBluetooth owns this service's live characteristic array for the callback.
    let Some(characteristics) = (unsafe { service.characteristics() }) else {
        let decision = state
            .characteristic_notification
            .borrow_mut()
            .characteristics_discovered(peripheral_id, false, Err(()));
        apply_characteristic_notification_decision(state, decision);
        return;
    };
    let expected_characteristic = state.characteristic_notification.borrow().characteristic();
    let Some(expected_characteristic) = expected_characteristic else {
        contain_characteristic_notification_panic(state);
        return;
    };
    let selection = unique_match(characteristics.count(), |index| {
        let candidate = characteristics.objectAtIndex(index);
        // SAFETY: The characteristic is a live member of the selected service's native array.
        let native_uuid: Retained<CBUUID> = unsafe { candidate.UUID() };
        characteristic_uuid_from_native(&native_uuid)
            .map(|uuid| uuid == expected_characteristic)
            .ok_or(())
    });
    let selected = match selection {
        Ok(UniqueMatch::One(index)) => Some(characteristics.objectAtIndex(index)),
        _ => None,
    };
    let notifiable = selected.as_deref().is_some_and(|characteristic| {
        // SAFETY: This is the unique characteristic returned by CoreBluetooth for the service.
        let properties = unsafe { characteristic.properties() };
        properties.contains(CBCharacteristicProperties::Notify)
            || properties.contains(CBCharacteristicProperties::Indicate)
    });
    let decision = state
        .characteristic_notification
        .borrow_mut()
        .characteristics_discovered(peripheral_id, false, selection.map(|m| (m, notifiable)));
    match decision {
        NotificationDecision::Enable(_) => {
            let Some(characteristic) = selected else {
                contain_characteristic_notification_panic(state);
                return;
            };
            *state
                .characteristic_notification_characteristic
                .borrow_mut() = Some(characteristic.clone());
            let delegate_object = ProtocolObject::from_ref(delegate);
            // SAFETY: The characteristic uniquely supports Notify or Indicate and the retained
            // native objects/delegate remain live through the asynchronous state callback.
            unsafe {
                peripheral.setDelegate(Some(delegate_object));
                peripheral.setNotifyValue_forCharacteristic(true, &characteristic);
            }
        }
        decision => apply_characteristic_notification_decision(state, decision),
    }
}

fn handle_characteristic_notification_state(
    state: &CentralState,
    peripheral: &CBPeripheral,
    characteristic: &CBCharacteristic,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_notification_panic(state);
        return;
    };
    let selected_characteristic = state
        .characteristic_notification_characteristic
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, characteristic));
    if !selected_characteristic {
        return;
    }
    let (expected, active) = {
        let notification = state.characteristic_notification.borrow();
        (
            notification.characteristic(),
            notification.peripheral() == Some(peripheral_id) && notification.is_transitioning(),
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    let Some(expected) = expected else {
        contain_characteristic_notification_panic(state);
        return;
    };
    // SAFETY: CoreBluetooth supplies the selected live characteristic for this callback.
    let native_uuid: Retained<CBUUID> = unsafe { characteristic.UUID() };
    if characteristic_uuid_from_native(&native_uuid) != Some(expected) {
        contain_characteristic_notification_panic(state);
        return;
    }
    // SAFETY: The selected peripheral is live and this callback runs on the manager's main queue.
    let is_notifying = unsafe { characteristic.isNotifying() };
    let decision = state
        .characteristic_notification
        .borrow_mut()
        .notification_state_changed(peripheral_id, expected, is_notifying, had_error);
    apply_characteristic_notification_decision(state, decision);
}

fn handle_characteristic_write(
    state: &CentralState,
    peripheral: &CBPeripheral,
    characteristic: &CBCharacteristic,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_write_panic(state);
        return;
    };
    let (expected_characteristic, active) = {
        let write = state.characteristic_write.borrow();
        (
            write.characteristic(),
            write.peripheral() == Some(peripheral_id)
                && write.state() == BluetoothCharacteristicWriteState::Writing,
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    let selected_characteristic = state
        .characteristic_write_characteristic
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, characteristic));
    if !selected_characteristic {
        return;
    }
    // SAFETY: The selected peripheral is live and callback-bound to the main queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        let decision = state
            .characteristic_write
            .borrow_mut()
            .disconnected(peripheral_id);
        apply_characteristic_write_decision(state, decision);
        return;
    }
    let Some(expected_characteristic) = expected_characteristic else {
        contain_characteristic_write_panic(state);
        return;
    };
    // SAFETY: CoreBluetooth provides the selected live characteristic for this callback.
    let native_uuid: Retained<CBUUID> = unsafe { characteristic.UUID() };
    if characteristic_uuid_from_native(&native_uuid) != Some(expected_characteristic) {
        contain_characteristic_write_panic(state);
        return;
    }
    let decision = state.characteristic_write.borrow_mut().value_written(
        peripheral_id,
        expected_characteristic,
        had_error,
    );
    apply_characteristic_write_decision(state, decision);
}

fn handle_characteristic_value(
    state: &CentralState,
    peripheral: &CBPeripheral,
    characteristic: &CBCharacteristic,
    had_error: bool,
) {
    let notification_characteristic = state
        .characteristic_notification_characteristic
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, characteristic));
    if notification_characteristic {
        handle_characteristic_notification_value(state, peripheral, characteristic, had_error);
        return;
    }
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_read_panic(state);
        return;
    };
    let (expected_characteristic, active) = {
        let read = state.characteristic_read.borrow();
        (
            read.characteristic(),
            read.is_active() && read.peripheral() == Some(peripheral_id),
        )
    };
    if !active || state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    let selected_characteristic = state
        .characteristic_read_characteristic
        .borrow()
        .as_deref()
        .is_some_and(|selected| core::ptr::eq(selected, characteristic));
    if !selected_characteristic {
        return;
    }
    // SAFETY: The selected peripheral is live and callback-bound to the main queue.
    if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        apply_characteristic_read_decision(
            state,
            state
                .characteristic_read
                .borrow_mut()
                .disconnected(peripheral_id),
            None,
        );
        return;
    }
    let Some(expected_characteristic) = expected_characteristic else {
        contain_characteristic_read_panic(state);
        return;
    };
    // SAFETY: CoreBluetooth provides the selected live characteristic for this callback.
    let native_uuid: Retained<CBUUID> = unsafe { characteristic.UUID() };
    if characteristic_uuid_from_native(&native_uuid) != Some(expected_characteristic) {
        contain_characteristic_read_panic(state);
        return;
    }
    let value = if had_error {
        Err(BluetoothCharacteristicReadError::NativeFailure)
    } else {
        // SAFETY: CoreBluetooth owns the immutable NSData for the callback duration; getBytes
        // copies the exact bounded payload into Rust-owned storage before the callback returns.
        let native_value: Option<Retained<NSData>> = unsafe { characteristic.value() };
        native_value
            .as_deref()
            .map(copy_characteristic_value)
            .unwrap_or(Err(BluetoothCharacteristicReadError::Unavailable))
    };
    let value_error = value.as_ref().err().copied();
    let decision = state.characteristic_read.borrow_mut().value_updated(
        peripheral_id,
        expected_characteristic,
        had_error,
        value_error,
    );
    apply_characteristic_read_decision(state, decision, value.ok());
}

fn handle_characteristic_notification_value(
    state: &CentralState,
    peripheral: &CBPeripheral,
    characteristic: &CBCharacteristic,
    had_error: bool,
) {
    // SAFETY: CoreBluetooth invokes this delegate with its live connected peripheral.
    let native_id: Retained<NSUUID> = unsafe { peripheral.identifier() };
    let Some(peripheral_id) = peripheral_id_from_native(&native_id) else {
        contain_characteristic_notification_panic(state);
        return;
    };
    let expected_characteristic = state.characteristic_notification.borrow().characteristic();
    let Some(expected_characteristic) = expected_characteristic else {
        return;
    };
    if state.connection.borrow().target() != Some(peripheral_id) {
        return;
    }
    // SAFETY: CoreBluetooth provides the selected live characteristic for this callback.
    let native_uuid: Retained<CBUUID> = unsafe { characteristic.UUID() };
    if characteristic_uuid_from_native(&native_uuid) != Some(expected_characteristic) {
        contain_characteristic_notification_panic(state);
        return;
    }
    // SAFETY: The selected peer is live and callback-bound to the manager's main queue.
    let result = if unsafe { peripheral.state() } != CBPeripheralState::Connected {
        let decision = state
            .characteristic_notification
            .borrow_mut()
            .disconnected(peripheral_id);
        apply_characteristic_notification_decision(state, decision);
        return;
    } else if had_error {
        Err(BluetoothCharacteristicNotificationError::NativeFailure)
    } else {
        // SAFETY: CoreBluetooth owns this immutable NSData for the callback duration; the copy is
        // bounded before reading and stored in Rust-owned inline value storage.
        let native_value: Option<Retained<NSData>> = unsafe { characteristic.value() };
        native_value
            .as_deref()
            .map(copy_characteristic_notification_value)
            .unwrap_or(Err(BluetoothCharacteristicNotificationError::Unavailable))
    };
    let service = state
        .characteristic_notification
        .borrow_mut()
        .value_updated(
            peripheral_id,
            expected_characteristic,
            result.as_ref().err().copied(),
        );
    if let Some(service) = service {
        state.characteristic_notification_events.borrow_mut().push(
            BluetoothCharacteristicNotificationEvent::new(
                peripheral_id,
                service,
                expected_characteristic,
                result,
            ),
        );
    }
}

fn copy_characteristic_value(
    value: &NSData,
) -> Result<BluetoothCharacteristicValue, BluetoothCharacteristicReadError> {
    let length = value.length();
    if length > framework_bluetooth::BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES {
        return Err(BluetoothCharacteristicReadError::ValueTooLarge);
    }
    let mut bytes = [0; framework_bluetooth::BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES];
    if length > 0 {
        let buffer = core::ptr::NonNull::new(bytes.as_mut_ptr().cast::<core::ffi::c_void>())
            .ok_or(BluetoothCharacteristicReadError::Unavailable)?;
        // SAFETY: The stack buffer is valid and has at least `length` bytes; `length` was capped.
        unsafe { value.getBytes_length(buffer, length) };
    }
    BluetoothCharacteristicValue::from_bytes(&bytes[..length])
        .ok_or(BluetoothCharacteristicReadError::ValueTooLarge)
}

fn copy_characteristic_notification_value(
    value: &NSData,
) -> Result<BluetoothCharacteristicValue, BluetoothCharacteristicNotificationError> {
    let length = value.length();
    if length > framework_bluetooth::BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES {
        return Err(BluetoothCharacteristicNotificationError::ValueTooLarge);
    }
    let mut bytes = [0; framework_bluetooth::BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES];
    if length > 0 {
        let buffer = core::ptr::NonNull::new(bytes.as_mut_ptr().cast::<core::ffi::c_void>())
            .ok_or(BluetoothCharacteristicNotificationError::Unavailable)?;
        // SAFETY: The stack buffer is valid and has at least `length` bytes; `length` was capped.
        unsafe { value.getBytes_length(buffer, length) };
    }
    BluetoothCharacteristicValue::from_bytes(&bytes[..length])
        .ok_or(BluetoothCharacteristicNotificationError::ValueTooLarge)
}

fn apply_characteristic_read_decision(
    state: &CentralState,
    decision: ReadDecision,
    value: Option<BluetoothCharacteristicValue>,
) {
    let event = match decision {
        ReadDecision::Failed {
            peripheral,
            service,
            characteristic,
            error,
        } => BluetoothCharacteristicReadEvent::Failed {
            peripheral,
            service,
            characteristic,
            error,
        },
        ReadDecision::Completed {
            peripheral,
            service,
            characteristic,
        } => match value {
            Some(value) => BluetoothCharacteristicReadEvent::Completed {
                peripheral,
                service,
                characteristic,
                value,
            },
            None => BluetoothCharacteristicReadEvent::Failed {
                peripheral,
                service,
                characteristic,
                error: BluetoothCharacteristicReadError::Unavailable,
            },
        },
        ReadDecision::Read | ReadDecision::Ignore => return,
    };
    state.characteristic_read_events.borrow_mut().push(event);
    clear_characteristic_read_objects(state);
}

fn clear_characteristic_read_objects(state: &CentralState) {
    state.characteristic_read_characteristic.borrow_mut().take();
    state.characteristic_read_service.borrow_mut().take();
}

fn apply_characteristic_write_decision(state: &CentralState, decision: WriteDecision) {
    let event = match decision {
        WriteDecision::Failed {
            peripheral,
            service,
            characteristic,
            error,
        } => BluetoothCharacteristicWriteEvent::Failed {
            peripheral,
            service,
            characteristic,
            error,
        },
        WriteDecision::Completed {
            peripheral,
            service,
            characteristic,
        } => BluetoothCharacteristicWriteEvent::Completed {
            peripheral,
            service,
            characteristic,
        },
        WriteDecision::Write | WriteDecision::Ignore => return,
    };
    state.characteristic_write_events.borrow_mut().push(event);
    if !state.characteristic_write.borrow().callback_ambiguous() {
        clear_characteristic_write_objects(state);
    }
}

fn apply_characteristic_notification_decision(
    state: &CentralState,
    decision: NotificationDecision,
) {
    match decision {
        NotificationDecision::Event {
            peripheral,
            service,
            characteristic,
            error,
        } => {
            state.characteristic_notification_events.borrow_mut().push(
                BluetoothCharacteristicNotificationEvent::new(
                    peripheral,
                    service,
                    characteristic,
                    Err(error),
                ),
            );
        }
        NotificationDecision::Disabled => clear_characteristic_notification_objects(state),
        NotificationDecision::Ignore
        | NotificationDecision::Enable(_)
        | NotificationDecision::Enabled => return,
    }
    if !state.characteristic_notification.borrow().is_active()
        && !state
            .characteristic_notification
            .borrow()
            .callback_ambiguous()
    {
        clear_characteristic_notification_objects(state);
    }
}

fn clear_characteristic_write_objects(state: &CentralState) {
    state
        .characteristic_write_characteristic
        .borrow_mut()
        .take();
    state.characteristic_write_service.borrow_mut().take();
    state.characteristic_write_data.borrow_mut().take();
}

fn clear_characteristic_notification_objects(state: &CentralState) {
    state
        .characteristic_notification_service
        .borrow_mut()
        .take();
    state
        .characteristic_notification_characteristic
        .borrow_mut()
        .take();
}

fn clear_quarantined_characteristic_write_objects(state: &CentralState) {
    if state.characteristic_write.borrow().callback_ambiguous() {
        clear_characteristic_write_objects(state);
    }
}

fn clear_quarantined_characteristic_notification_objects(state: &CentralState) {
    if state
        .characteristic_notification
        .borrow()
        .callback_ambiguous()
    {
        clear_characteristic_notification_objects(state);
    }
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

fn characteristic_uuid_from_native(uuid: &CBUUID) -> Option<BluetoothCharacteristicUuid> {
    service_uuid_from_native(uuid)
        .map(|uuid| BluetoothCharacteristicUuid::from_uuid_bytes(*uuid.as_uuid_bytes()))
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
