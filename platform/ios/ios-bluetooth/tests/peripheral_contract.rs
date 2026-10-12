use framework_bluetooth::{
    BLUETOOTH_PERIPHERAL_CHARACTERISTIC_MAX_COUNT, BluetoothCharacteristicUuid,
    BluetoothPeripheral, BluetoothPeripheralBackend, BluetoothPeripheralCharacteristic,
    BluetoothPeripheralConfigurationError, BluetoothPeripheralError, BluetoothPeripheralService,
    BluetoothPeripheralState, BluetoothServiceUuid,
};

struct FakePeripheral {
    state: BluetoothPeripheralState,
    service: Option<BluetoothPeripheralService>,
}

impl BluetoothPeripheralBackend for FakePeripheral {
    fn start_advertising(
        &mut self,
        service: BluetoothPeripheralService,
    ) -> Result<(), BluetoothPeripheralError> {
        if self.service.is_some() {
            return Err(BluetoothPeripheralError::AlreadyActive);
        }
        self.service = Some(service);
        self.state = BluetoothPeripheralState::Starting;
        Ok(())
    }

    fn stop_advertising(&mut self) {
        self.service = None;
        self.state = BluetoothPeripheralState::Stopped;
    }

    fn peripheral_state(&self) -> BluetoothPeripheralState {
        self.state
    }

    fn last_peripheral_error(&self) -> Option<BluetoothPeripheralError> {
        None
    }
}

fn characteristic(value: &[u8]) -> BluetoothPeripheralCharacteristic {
    BluetoothPeripheralCharacteristic::new(
        BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16]),
        value,
    )
    .unwrap()
}

#[test]
fn static_service_rejects_empty_duplicate_oversized_and_excess_characteristics() {
    let service_uuid = BluetoothServiceUuid::from_uuid_bytes([0x18; 16]);
    assert_eq!(
        BluetoothPeripheralService::new(service_uuid, &[]),
        Err(BluetoothPeripheralConfigurationError::EmptyCharacteristics)
    );
    assert_eq!(
        BluetoothPeripheralCharacteristic::new(
            BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16]),
            &[0; 513],
        ),
        Err(BluetoothPeripheralConfigurationError::ValueTooLarge)
    );
    let first = characteristic(&[0xA5; 512]);
    let duplicate = characteristic(&[]);
    assert_eq!(
        BluetoothPeripheralService::new(service_uuid, &[first, duplicate]),
        Err(BluetoothPeripheralConfigurationError::DuplicateCharacteristicUuid)
    );
    let excess = [first; BLUETOOTH_PERIPHERAL_CHARACTERISTIC_MAX_COUNT + 1];
    assert_eq!(
        BluetoothPeripheralService::new(service_uuid, &excess),
        Err(BluetoothPeripheralConfigurationError::TooManyCharacteristics)
    );
}

#[test]
fn static_service_accepts_maximum_value_and_facade_starts_stops() {
    let service_uuid = BluetoothServiceUuid::from_uuid_bytes([0x18; 16]);
    let value = [0xA5; 512];
    let service = BluetoothPeripheralService::new(service_uuid, &[characteristic(&value)]).unwrap();
    assert_eq!(service.uuid(), service_uuid);
    assert_eq!(service.characteristics().len(), 1);
    assert_eq!(service.characteristics()[0].value().as_bytes(), &value);
    let multiple = [
        BluetoothPeripheralCharacteristic::new(
            BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16]),
            &[0xA5],
        )
        .unwrap(),
        BluetoothPeripheralCharacteristic::new(
            BluetoothCharacteristicUuid::from_uuid_bytes([0x2B; 16]),
            &[0x5A],
        )
        .unwrap(),
    ];
    let multi_service = BluetoothPeripheralService::new(service_uuid, &multiple).unwrap();
    assert_eq!(multi_service.characteristics().len(), 2);
    let at_capacity: [BluetoothPeripheralCharacteristic;
        BLUETOOTH_PERIPHERAL_CHARACTERISTIC_MAX_COUNT] = core::array::from_fn(|index| {
        BluetoothPeripheralCharacteristic::new(
            BluetoothCharacteristicUuid::from_uuid_bytes([index as u8; 16]),
            &[index as u8],
        )
        .unwrap()
    });
    let full_service = BluetoothPeripheralService::new(service_uuid, &at_capacity).unwrap();
    assert_eq!(
        full_service.characteristics().len(),
        BLUETOOTH_PERIPHERAL_CHARACTERISTIC_MAX_COUNT
    );

    let mut peripheral = BluetoothPeripheral::new(FakePeripheral {
        state: BluetoothPeripheralState::Idle,
        service: None,
    });
    assert_eq!(peripheral.state(), BluetoothPeripheralState::Idle);
    assert_eq!(peripheral.start_advertising(service), Ok(()));
    assert_eq!(peripheral.state(), BluetoothPeripheralState::Starting);
    assert_eq!(
        peripheral.start_advertising(service),
        Err(BluetoothPeripheralError::AlreadyActive)
    );
    peripheral.stop_advertising();
    assert_eq!(peripheral.state(), BluetoothPeripheralState::Stopped);
    assert_eq!(peripheral.last_error(), None);
}
