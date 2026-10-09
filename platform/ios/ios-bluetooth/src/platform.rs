use crate::conversion::authorization_from_native;
use framework_bluetooth::{BluetoothAuthorization, BluetoothAuthorizationBackend};
use objc2_core_bluetooth::CBManager;

/// Stateless iOS CoreBluetooth authorization-status backend.
///
/// This backend requires iOS 13.1 or newer when [`authorization_status`](Self::authorization_status)
/// is called. It queries `CBManager.authorization` and does not allocate a manager object.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosBluetoothBackend;

impl IosBluetoothBackend {
    /// Creates a stateless backend without querying authorization or initializing CoreBluetooth.
    pub const fn new() -> Self {
        Self
    }
}

impl BluetoothAuthorizationBackend for IosBluetoothBackend {
    fn authorization_status(&self) -> BluetoothAuthorization {
        // SAFETY: The public `CBManager.authorization` class property is available from iOS 13.1.
        // This backend documents iOS 13.1 as its minimum runtime API floor and creates no manager.
        let status = unsafe { CBManager::authorization_class() };
        authorization_from_native(status.0)
    }
}
