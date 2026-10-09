# iOS CFData byte copies

`ios-data` maps `framework_data::ByteView` and `framework_data::OwnedBytes` to immutable Core Foundation `CFData` by explicit O(n) copies

`IosData::copy_from` checks the byte count against `CFIndex`, calls the fallible `CFData::new` wrapper for `CFDataCreate`, and maps a null native result to `IosDataError::NativeAllocation`. It does not call `CFData::from_bytes` or a no-copy API

`IosData::copy_to_owned` checks the native length, calls `try_reserve_exact` before it writes any bytes, then uses `CFDataGetBytes` to copy into Rust-owned storage. It maps an unrepresentable count to `LengthOverflow` and a reserve error to `RustAllocation`. Empty data returns an empty `OwnedBytes` without a native byte-buffer access

`IosData::as_cf_data` lends `&CFData` only for the owner lifetime. It exposes no raw pointer or ownership transfer. `CFData` and `NSData` are toll-free bridged by Apple, so Apple APIs that accept `CFDataRef` or its `NSData` counterpart may use this borrowed handle; this crate does not provide an `NSString` or Foundation wrapper

Both directions copy bytes. This crate does not expose `CFMutableData`, `CFDataCreateWithBytesNoCopy`, UTF-8 conversion, file or Keychain operations, or a zero-copy claim. It has no permission, entitlement, or `Info.plist` requirement

`sh platform/ios/ios-data/check-link-imports.sh` builds device and Simulator link probes and inspects imports without running either probe. Link evidence does not prove live app use, runtime parity, allocation behavior under memory pressure, or performance

## Validation status

The exact local toolchain, SDK, import set, checks, and evidence limits are recorded in `PLAN_IOS_DATA.md`
