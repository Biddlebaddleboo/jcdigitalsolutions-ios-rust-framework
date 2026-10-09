# B293: Opaque iOS File Data-Protection-Class Code

## Scope

B293 adds `IosFiles::entry_data_protection_class_code(&self, path: AppPath<'_>) -> Result<IosFileDataProtectionClassCode, FileError>` for one app-sandbox regular file or directory. It returns the raw `u32` reported by descriptor-bound `fgetattrlist(ATTR_CMN_DATA_PROTECT_FLAGS)`

`IosFileDataProtectionClassCode::raw_value()` exists only to preserve or display the observed number. The API does not name protection levels, map the number to Foundation values, or support cross-object or cross-OS-version comparison. The number is not an object identity or a cross-call stable-object snapshot

## API and source evidence

The installed iPhoneOS 26.5 SDK `sys/attr.h` defines public `ATTR_CMN_DATA_PROTECT_FLAGS` as `0x40000000`. Locked `libc` 0.2.190 binds that constant. Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) documents a `u_int32_t` field that contains the file or directory's data-protection class. It publishes no mapping from numeric values to named protection levels

The method sets `attrlist.commonattr` to `ATTR_CMN_DATA_PROTECT_FLAGS`, leaves `forkattr` zero, and uses `fgetattrlist` without `FSOPT_ATTR_CMN_EXTENDED`. The fixed 8-byte buffer contains the leading 4-byte length and returned `u32`. XNU skips unsupported attributes by default; a length-only response maps to `Unsupported`, malformed lengths map to `InvalidInput`, and `EINVAL` or `ENOTSUP` maps to `Unsupported`

The SDK declares `fgetattrlist` at iOS 3.0 and gives no separate availability annotation for this attribute. The existing `ios-files` package floor remains iOS 10.0; B293 adds no deployment-floor change, framework, or dependency. This operation uses the File Timestamp required-reason API category; the host must declare an applicable approved reason in its `PrivacyInfo.xcprivacy` for actual use

## Contract limits

- The method validates `AppPath`, uses the existing descriptor-relative no-follow parent traversal, opens the final component with `O_NOFOLLOW | O_NONBLOCK`, and accepts only a regular file or directory. A final symlink or other entry kind returns `InvalidInput`
- `fgetattrlist` reads the attribute from the selected open descriptor; the code is for that object as opened by this call. No identity is returned, and no identity, persistence, or same-object relation across later calls is promised
- Callers may preserve or display the raw `u32` only. Do not infer a named protection level, read/write availability, lock state, encryption state, system/hardware key behavior, or security guarantee. Do not compare values across objects or OS versions
- Opening can fail before the metadata query, including due to ordinary access policy or current file state. An open failure does not report a protection class
- The query reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no portable `FileBackend` behavior. It does not set protection metadata
- Existing descriptor-relative no-follow traversal does not prevent a concurrent native rename from moving an already-open parent directory outside its original root. See the B1 confinement limits in [`PLAN_IOS_APP_DATA.md`](PLAN_IOS_APP_DATA.md)

## Relation to B168

B168's no-go remains in force for a named protection-level API: `F_GETPROTECTIONCLASS` and Foundation's path/URL values do not provide a verified descriptor-bound numeric mapping. B293 is narrower and separate: it exposes only the documented opaque `u32` from `ATTR_CMN_DATA_PROTECT_FLAGS`, without naming or mapping levels

## Validation

No tests, consumers, probes, or live sandbox queries are part of this slice. Scoped non-test gates: locked offline iOS device and Simulator `cargo check`, strict Clippy, rustdoc, `cargo fmt --check`, docs-check, and `git diff --check`
