# B365: iOS Volume Used-Capacity Snapshot

## API

`IosFiles::volume_used_capacity_bytes(AppDirectory) -> Result<u64, FileError>` reports the volume's filesystem-reported used bytes from the retained semantic-root descriptor. It adds no portable `FileBackend` operation

## Query and buffer contract

The first `fgetattrlist` request asks only for `ATTR_VOL_INFO | ATTR_VOL_ATTRIBUTES`. The buffer has a `u32` returned-length header followed by `vol_attributes_attr_t`; its `validattr` field begins the payload, and the first two `attribute_set_t` fields are `commonattr` then `volattr`. The parser validates the exact buffer and returned lengths, then requires `validattr.volattr & ATTR_VOL_SPACEUSED != 0`

The second request asks for `ATTR_VOL_INFO | ATTR_VOL_SPACEUSED` on the same open root descriptor. XNU defines the value as `off_t`. Its attribute-buffer rule aligns each value to four bytes, including 64-bit values, so the `off_t` follows the four-byte length header at byte offset four. The buffer is exactly `size_of::<u32>() + size_of::<libc::off_t>()`; the parser checks both buffer and returned lengths and reads the possibly eight-byte-aligned `off_t` without an alignment assumption. An omitted value maps to `Unsupported`; a negative value or malformed length maps to `InvalidInput`

The two requests use the same descriptor but are separate, non-atomic queries. The value is point-in-time and can change between either query or a B137/B146 query

## Semantics and limits

- XNU defines `ATTR_VOL_SPACEUSED` as total used bytes on the volume. On space-sharing volumes, it may differ from volume size minus free space; callers must not derive it from B137 or B146
- This is volume-wide use, not this app/container's use, an app quota, physical-device use, a reservation, a path's allocation, or a guarantee that a later write will succeed
- The query reads no file content, accepts no arbitrary URL, starts no security scope, and changes no file operation
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. The host app or SDK that uses the method must declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use
- No permission prompt, Info.plist key, entitlement, framework, dependency, or deployment-floor increase is added

## SDK and binding evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_SPACEUSED` as `0x00800000`, `ATTR_VOL_INFO` as `0x80000000`, and `ATTR_VOL_ATTRIBUTES` as `0x40000000` in `sys/attr.h`
- The SDK defines `vol_attributes_attr_t` as `validattr` followed by `nativeattr`; `attribute_set_t` lists `commonattr` then `volattr`, both `attrgroup_t` (`u_int32_t`) fields
- The SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability and gives no separate availability annotation for these attribute bits; the `ios-files` floor remains iOS 10.0
- Locked `libc` 0.2.190 binds the attribute constants, `vol_attributes_attr_t`, `attribute_set_t`, `off_t`, and `fgetattrlist`
- Apple's XNU manual defines the buffer length header, four-byte alignment for every attribute including 64-bit types, the `off_t` result, and the space-sharing caveat

## Validation

- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files` (host)
- Passed strict Clippy with `-- -D warnings` for host, iOS device, and arm64 Simulator
- Passed rustdoc for host, iOS device, and arm64 Simulator
- Passed `cargo +1.94.1 fmt -p ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and `git diff --check`
- No tests, runtime capacity query, consumer, live filesystem call, or probe was run

## Sources

- [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2)
- Apple required-reason [File Timestamp API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype)
