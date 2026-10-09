use framework_bluetooth::{BluetoothAuthorization, BluetoothPeripheralId};

/// Maps `CBManagerAuthorization` raw values to portable Bluetooth authorization states.
pub(crate) const fn authorization_from_native(value: isize) -> BluetoothAuthorization {
    match value {
        0 => BluetoothAuthorization::NotDetermined,
        1 => BluetoothAuthorization::Restricted,
        2 => BluetoothAuthorization::Denied,
        3 => BluetoothAuthorization::Allowed,
        _ => BluetoothAuthorization::Unknown,
    }
}

/// Parses NSUUID's canonical 36-byte ASCII representation into UUID octet order.
pub(crate) fn peripheral_id_from_uuid_ascii(value: &[u8; 36]) -> Option<BluetoothPeripheralId> {
    let mut bytes = [0; 16];
    let mut output = 0;
    let mut high_nibble = None;
    for (index, character) in value.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if character != b'-' {
                return None;
            }
            continue;
        }
        let nibble = match character {
            b'0'..=b'9' => character - b'0',
            b'a'..=b'f' => character - b'a' + 10,
            b'A'..=b'F' => character - b'A' + 10,
            _ => return None,
        };
        if let Some(high) = high_nibble.take() {
            bytes[output] = (high << 4) | nibble;
            output += 1;
        } else {
            high_nibble = Some(nibble);
        }
    }
    if output != bytes.len() || high_nibble.is_some() {
        return None;
    }
    Some(BluetoothPeripheralId::from_uuid_bytes(bytes))
}

/// Converts CoreBluetooth RSSI to fixed-width dBm, mapping its reserved 127 value to unavailable.
pub(crate) fn rssi_dbm_from_native(value: isize) -> Option<i32> {
    if value == 127 {
        None
    } else {
        i32::try_from(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_authorization_values_keep_each_state_distinct() {
        assert_eq!(
            authorization_from_native(0),
            BluetoothAuthorization::NotDetermined
        );
        assert_eq!(
            authorization_from_native(1),
            BluetoothAuthorization::Restricted
        );
        assert_eq!(authorization_from_native(2), BluetoothAuthorization::Denied);
        assert_eq!(
            authorization_from_native(3),
            BluetoothAuthorization::Allowed
        );
        assert_eq!(
            authorization_from_native(4),
            BluetoothAuthorization::Unknown
        );
    }

    #[test]
    fn peer_uuid_ascii_maps_to_canonical_octet_order() {
        assert_eq!(
            peripheral_id_from_uuid_ascii(b"00112233-4455-6677-8899-AABBCCDDEEFF")
                .unwrap()
                .as_uuid_bytes(),
            &[
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
                0xEE, 0xFF,
            ]
        );
        assert!(peripheral_id_from_uuid_ascii(b"00112233-4455-6677-8899-AABBCCDDEEFZ").is_none());
        assert!(peripheral_id_from_uuid_ascii(b"00112233-4455-6677-8899_AABBCCDDEEFF").is_none());
    }

    #[test]
    fn corebluetooth_rssi_sentinel_and_fixed_width_range_are_preserved() {
        assert_eq!(rssi_dbm_from_native(-127), Some(-127));
        assert_eq!(rssi_dbm_from_native(-54), Some(-54));
        assert_eq!(rssi_dbm_from_native(127), None);
        assert_eq!(
            rssi_dbm_from_native(isize::MAX),
            i32::try_from(isize::MAX).ok()
        );
    }
}
