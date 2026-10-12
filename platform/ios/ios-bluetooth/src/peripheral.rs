use core::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use crate::peripheral_policy::{ManagerAvailability, PeripheralAction, PeripheralPolicy};
use framework_bluetooth::{
    BluetoothPeripheralBackend, BluetoothPeripheralError, BluetoothPeripheralService,
    BluetoothPeripheralState,
};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_bluetooth::{
    CBAdvertisementDataServiceUUIDsKey, CBAttributePermissions, CBCharacteristicProperties,
    CBManagerState, CBMutableCharacteristic, CBMutableService, CBPeripheralManager,
    CBPeripheralManagerDelegate, CBService, CBUUID,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSObject, NSObjectProtocol};

struct PeripheralState {
    policy: RefCell<PeripheralPolicy>,
    service: RefCell<Option<BluetoothPeripheralService>>,
    native_service: RefCell<Option<Retained<CBMutableService>>>,
}

impl PeripheralState {
    const fn new() -> Self {
        Self {
            policy: RefCell::new(PeripheralPolicy::new()),
            service: RefCell::new(None),
            native_service: RefCell::new(None),
        }
    }
}

struct DelegateIvars {
    state: Rc<PeripheralState>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateIvars]
    struct PeripheralDelegate;

    // SAFETY: NSObjectProtocol adds no methods. The manager, retained delegate, and all state are
    // created, called, and released on the supplied main thread.
    unsafe impl NSObjectProtocol for PeripheralDelegate {}

    // SAFETY: CoreBluetooth callbacks use the main queue because the manager has a nil dispatch
    // queue. Every callback contains Rust panics before returning across the Objective-C ABI.
    #[allow(non_snake_case)]
    unsafe impl CBPeripheralManagerDelegate for PeripheralDelegate {
        #[unsafe(method(peripheralManagerDidUpdateState:))]
        unsafe fn peripheralManagerDidUpdateState(&self, manager: &CBPeripheralManager) {
            let state = &self.ivars().state;
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let availability = manager_availability(unsafe { manager.state() });
                let action = state.policy.borrow_mut().manager_update(availability);
                apply_action(state, manager, action);
            }));
            if outcome.is_err() {
                contain_panic(state, manager);
            }
        }

        #[unsafe(method(peripheralManager:didAddService:error:))]
        unsafe fn peripheralManager_didAddService_error(
            &self,
            manager: &CBPeripheralManager,
            service: &CBService,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let state = &self.ivars().state;
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                if !is_current_service(state, service) {
                    return;
                }
                let failed = error.is_some();
                let action = state.policy.borrow_mut().did_add_service(failed);
                apply_action(state, manager, action);
                if failed {
                    clear_service(state);
                }
            }));
            if outcome.is_err() {
                contain_panic(state, manager);
            }
        }

        #[unsafe(method(peripheralManagerDidStartAdvertising:error:))]
        unsafe fn peripheralManagerDidStartAdvertising_error(
            &self,
            manager: &CBPeripheralManager,
            error: Option<&objc2_foundation::NSError>,
        ) {
            let state = &self.ivars().state;
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let action = state
                    .policy
                    .borrow_mut()
                    .did_start_advertising(error.is_some());
                apply_action(state, manager, action);
            }));
            if outcome.is_err() {
                contain_panic(state, manager);
            }
        }
    }
);

impl PeripheralDelegate {
    fn new(marker: MainThreadMarker, state: Rc<PeripheralState>) -> Retained<Self> {
        let allocated: Allocated<Self> = marker.alloc::<Self>();
        let allocated = allocated.set_ivars(DelegateIvars { state });
        // SAFETY: The class derives directly from NSObject and all ivars are initialized before
        // the NSObject designated initializer runs.
        unsafe { msg_send![super(allocated), init] }
    }
}

/// Caller-owned main-thread CoreBluetooth peripheral backend for one bounded static GATT service.
///
/// Creating this backend does not initialize CoreBluetooth. The first explicit
/// [`start_advertising`](BluetoothPeripheralBackend::start_advertising) call creates a
/// `CBPeripheralManager` and may prompt for Bluetooth access. The manager delivers callbacks on
/// the main queue; construct, call, and drop this backend on the supplied main thread. Its single
/// primary service advertises its public UUID and contains one through eight static read-only
/// characteristics, each copied and bounded to 512 bytes. CoreBluetooth's static characteristic
/// values service reads without a dynamic request callback. Stop and drop remove only this
/// backend's service when the manager is powered on. The host owns Bluetooth privacy strings and
/// the `bluetooth-peripheral` background mode; background advertising or GATT access is not
/// guaranteed by this foreground adapter.
pub struct IosBluetoothPeripheralBackend {
    state: Rc<PeripheralState>,
    manager: Option<Retained<CBPeripheralManager>>,
    delegate: Option<Retained<PeripheralDelegate>>,
    _marker: MainThreadMarker,
}

impl IosBluetoothPeripheralBackend {
    /// Creates an idle peripheral backend on the supplied main-thread context.
    pub fn new(marker: MainThreadMarker) -> Self {
        Self {
            state: Rc::new(PeripheralState::new()),
            manager: None,
            delegate: None,
            _marker: marker,
        }
    }

    fn create_manager(&mut self) {
        let delegate = PeripheralDelegate::new(self._marker, Rc::clone(&self.state));
        let delegate_object = ProtocolObject::from_ref(&*delegate);
        // SAFETY: CoreBluetooth's delegate property is weak, so this backend retains the delegate.
        // A nil queue documents main-queue callback delivery, matching its main-thread marker.
        let manager = unsafe {
            CBPeripheralManager::initWithDelegate_queue(
                CBPeripheralManager::alloc(),
                Some(delegate_object),
                None,
            )
        };
        self.delegate = Some(delegate);
        self.manager = Some(manager);
    }
}

impl BluetoothPeripheralBackend for IosBluetoothPeripheralBackend {
    fn start_advertising(
        &mut self,
        service: BluetoothPeripheralService,
    ) -> Result<(), BluetoothPeripheralError> {
        if self.state.policy.borrow().service_added()
            && self.state.service.borrow().as_ref() != Some(&service)
        {
            return Err(BluetoothPeripheralError::AlreadyActive);
        }
        self.state.policy.borrow_mut().start()?;
        *self.state.service.borrow_mut() = Some(service);
        if self.manager.is_none() {
            self.create_manager();
        }
        let manager = self.manager.as_deref().expect("manager just initialized");
        // SAFETY: The manager is owned by this main-thread backend.
        let availability = manager_availability(unsafe { manager.state() });
        let action = self.state.policy.borrow_mut().manager_update(availability);
        apply_action(&self.state, manager, action);
        Ok(())
    }

    fn stop_advertising(&mut self) {
        let cleanup = self.state.policy.borrow_mut().stop();
        if let Some(manager) = self.manager.as_deref() {
            // SAFETY: Stop and service removal execute on the manager's main-queue thread. The
            // action only removes the retained service that this backend created.
            unsafe {
                if cleanup.stop_advertising && manager.isAdvertising() {
                    manager.stopAdvertising();
                }
                if cleanup.remove_service
                    && manager.state() == CBManagerState::PoweredOn
                    && let Some(service) = self.state.native_service.borrow().as_deref()
                {
                    manager.removeService(service);
                    self.state.policy.borrow_mut().service_removed();
                    clear_service(&self.state);
                }
            }
            if unsafe { manager.state() } == CBManagerState::PoweredOff
                && !self.state.policy.borrow().has_pending_callback()
            {
                self.state.policy.borrow_mut().service_removed();
                clear_service(&self.state);
            }
        }
    }

    fn peripheral_state(&self) -> BluetoothPeripheralState {
        self.state.policy.borrow().state()
    }

    fn last_peripheral_error(&self) -> Option<BluetoothPeripheralError> {
        self.state.policy.borrow().error()
    }
}

impl Drop for IosBluetoothPeripheralBackend {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.as_deref() {
            // SAFETY: Drop runs on the stored main thread. Stop only this manager's advertisement,
            // remove only this backend's service, then clear its weak delegate before state release.
            unsafe {
                if manager.isAdvertising() {
                    manager.stopAdvertising();
                }
                if manager.state() == CBManagerState::PoweredOn
                    && let Some(service) = self.state.native_service.borrow().as_deref()
                {
                    manager.removeService(service);
                }
                manager.setDelegate(None);
            }
        }
    }
}

fn manager_availability(state: CBManagerState) -> ManagerAvailability {
    match state {
        CBManagerState::PoweredOn => ManagerAvailability::PoweredOn,
        CBManagerState::PoweredOff => ManagerAvailability::PoweredOff,
        CBManagerState::Unauthorized => ManagerAvailability::Unauthorized,
        CBManagerState::Unsupported => ManagerAvailability::Unsupported,
        _ => ManagerAvailability::Other,
    }
}

fn is_current_service(state: &PeripheralState, service: &CBService) -> bool {
    let current = state.native_service.borrow();
    let Some(current) = current.as_deref() else {
        return false;
    };
    let current: &CBService = current;
    core::ptr::eq(current, service)
}

fn make_service(service: BluetoothPeripheralService) -> Retained<CBMutableService> {
    let service_uuid_data = NSData::with_bytes(service.uuid().as_uuid_bytes());
    // SAFETY: UUIDWithData copies the exact 16-byte canonical Bluetooth UUID.
    let service_uuid = unsafe { CBUUID::UUIDWithData(&service_uuid_data) };
    let native_service = unsafe {
        CBMutableService::initWithType_primary(CBMutableService::alloc(), &service_uuid, true)
    };
    let mut characteristics = Vec::with_capacity(service.characteristics().len());
    for characteristic in service.characteristics() {
        let uuid_data = NSData::with_bytes(characteristic.uuid().as_uuid_bytes());
        // SAFETY: UUIDWithData copies the exact 16-byte canonical Bluetooth UUID.
        let uuid = unsafe { CBUUID::UUIDWithData(&uuid_data) };
        let value = NSData::with_bytes(characteristic.value().as_bytes());
        let native_characteristic = unsafe {
            CBMutableCharacteristic::initWithType_properties_value_permissions(
                CBMutableCharacteristic::alloc(),
                &uuid,
                CBCharacteristicProperties::Read,
                Some(&value),
                CBAttributePermissions::Readable,
            )
        };
        let native_characteristic = native_characteristic.into_super();
        characteristics.push(native_characteristic);
    }
    let native_characteristics = NSArray::from_retained_slice(&characteristics);
    // SAFETY: The retained array contains exactly the characteristics created for this service.
    unsafe { native_service.setCharacteristics(Some(&native_characteristics)) };
    native_service
}

fn apply_action(state: &PeripheralState, manager: &CBPeripheralManager, action: PeripheralAction) {
    match action {
        PeripheralAction::None => {}
        PeripheralAction::AddService => {
            if state.native_service.borrow().is_none() {
                let Some(service) = *state.service.borrow() else {
                    state.policy.borrow_mut().callback_panicked();
                    return;
                };
                *state.native_service.borrow_mut() = Some(make_service(service));
            }
            if let Some(service) = state.native_service.borrow().as_deref() {
                // SAFETY: The service is retained by this backend until its completion callback or
                // teardown, and CoreBluetooth is powered on when this action is issued.
                unsafe { manager.addService(service) };
            }
        }
        PeripheralAction::StartAdvertising => {
            let Some(service) = *state.service.borrow() else {
                state.policy.borrow_mut().callback_panicked();
                return;
            };
            let service_uuid_data = NSData::with_bytes(service.uuid().as_uuid_bytes());
            // SAFETY: UUIDWithData copies the exact 16-byte canonical Bluetooth UUID.
            let service_uuid = unsafe { CBUUID::UUIDWithData(&service_uuid_data) };
            let service_uuids = NSArray::from_slice(&[&*service_uuid]);
            let object: &AnyObject = &service_uuids;
            // SAFETY: CoreBluetooth declares this process-lifetime constant key.
            let advertisement_key = unsafe { CBAdvertisementDataServiceUUIDsKey };
            let advertisement = NSDictionary::from_slices(&[advertisement_key], &[object]);
            // SAFETY: The dictionary uses the documented service-UUID key and NSArray<CBUUID>
            // value supported by CoreBluetooth advertising data.
            unsafe { manager.startAdvertising(Some(&advertisement)) };
        }
        PeripheralAction::RemoveService => remove_owned_service(state, manager),
        PeripheralAction::ForgetNativeService => {
            state.native_service.borrow_mut().take();
        }
    }
}

fn remove_owned_service(state: &PeripheralState, manager: &CBPeripheralManager) {
    let native_state = unsafe { manager.state() };
    if native_state == CBManagerState::PoweredOn {
        if let Some(service) = state.native_service.borrow().as_deref() {
            // SAFETY: This is the exact retained CBMutableService created by this backend.
            unsafe { manager.removeService(service) };
        }
        state.policy.borrow_mut().service_removed();
        clear_service(state);
    } else if native_state == CBManagerState::PoweredOff {
        state.policy.borrow_mut().service_removed();
        clear_service(state);
    }
}

fn clear_service(state: &PeripheralState) {
    state.native_service.borrow_mut().take();
    state.service.borrow_mut().take();
}

fn contain_panic(state: &PeripheralState, manager: &CBPeripheralManager) {
    state.policy.borrow_mut().callback_panicked();
    // SAFETY: The callback is on the main queue; stopAdvertising and removal use this manager's
    // native state. A powered-off manager has already cleared the local database.
    unsafe {
        if manager.isAdvertising() {
            manager.stopAdvertising();
        }
    }
    remove_owned_service(state, manager);
}
