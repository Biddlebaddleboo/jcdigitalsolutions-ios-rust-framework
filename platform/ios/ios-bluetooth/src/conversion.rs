use framework_bluetooth::{BluetoothAuthorization, BluetoothPeripheralId, BluetoothServiceUuid};

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

/// Formats UUID octets as canonical uppercase ASCII for the public NSUUID parser.
pub(crate) fn uuid_ascii_from_id(value: BluetoothPeripheralId) -> [u8; 36] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = [0; 36];
    let mut cursor = 0;
    for (index, byte) in value.as_uuid_bytes().iter().copied().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            output[cursor] = b'-';
            cursor += 1;
        }
        output[cursor] = HEX[usize::from(byte >> 4)];
        output[cursor + 1] = HEX[usize::from(byte & 0x0f)];
        cursor += 2;
    }
    output
}

/// Converts CoreBluetooth RSSI to fixed-width dBm, mapping its reserved 127 value to unavailable.
pub(crate) fn rssi_dbm_from_native(value: isize) -> Option<i32> {
    if value == 127 {
        None
    } else {
        i32::try_from(value).ok()
    }
}

/// Normalizes a CBUUID string form into canonical 128-bit Bluetooth UUID octets.
pub(crate) fn service_uuid_from_uuid_ascii(value: &[u8]) -> Option<BluetoothServiceUuid> {
    const BASE: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34,
        0xfb,
    ];
    match value.len() {
        4 => {
            let mut bytes = BASE;
            parse_hex_octets(value, &mut bytes[2..4])?;
            Some(BluetoothServiceUuid::from_uuid_bytes(bytes))
        }
        8 => {
            let mut bytes = BASE;
            parse_hex_octets(value, &mut bytes[..4])?;
            Some(BluetoothServiceUuid::from_uuid_bytes(bytes))
        }
        36 => {
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
                let nibble = hex_nibble(character)?;
                if let Some(high) = high_nibble.take() {
                    bytes[output] = (high << 4) | nibble;
                    output += 1;
                } else {
                    high_nibble = Some(nibble);
                }
            }
            if output == bytes.len() && high_nibble.is_none() {
                Some(BluetoothServiceUuid::from_uuid_bytes(bytes))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_hex_octets(value: &[u8], output: &mut [u8]) -> Option<()> {
    if value.len() != output.len() * 2 {
        return None;
    }
    for (pair, byte) in value.chunks_exact(2).zip(output) {
        *byte = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(())
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
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
    fn peer_uuid_octets_format_for_native_lookup_without_byte_order_changes() {
        let id = BluetoothPeripheralId::from_uuid_bytes([
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF,
        ]);
        let ascii = uuid_ascii_from_id(id);
        assert_eq!(&ascii, b"00112233-4455-6677-8899-AABBCCDDEEFF");
        assert_eq!(peripheral_id_from_uuid_ascii(&ascii), Some(id));
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

    #[test]
    fn service_uuids_expand_short_forms_and_preserve_canonical_octet_order() {
        let base = [
            0x00, 0x00, 0x18, 0x0d, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b,
            0x34, 0xfb,
        ];
        assert_eq!(
            service_uuid_from_uuid_ascii(b"180D")
                .unwrap()
                .as_uuid_bytes(),
            &base
        );
        assert_eq!(
            service_uuid_from_uuid_ascii(b"0000180d")
                .unwrap()
                .as_uuid_bytes(),
            &base
        );
        let custom = [
            0x68, 0x75, 0x3a, 0x44, 0x4d, 0x6f, 0x12, 0x26, 0x9c, 0x60, 0x00, 0x50, 0xe4, 0xc0,
            0x00, 0x67,
        ];
        assert_eq!(
            service_uuid_from_uuid_ascii(b"68753A44-4D6F-1226-9C60-0050E4C00067")
                .unwrap()
                .as_uuid_bytes(),
            &custom
        );
    }

    #[test]
    fn service_uuid_parser_rejects_invalid_width_hex_and_hyphen_layout() {
        assert!(service_uuid_from_uuid_ascii(b"180").is_none());
        assert!(service_uuid_from_uuid_ascii(b"GGGG").is_none());
        assert!(service_uuid_from_uuid_ascii(b"68753A44-4D6F_1226-9C60-0050E4C00067").is_none());
        assert!(service_uuid_from_uuid_ascii(b"180d").is_some());
    }
}
