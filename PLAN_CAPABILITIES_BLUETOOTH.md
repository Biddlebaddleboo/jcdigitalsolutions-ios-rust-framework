# PLAN_CAPABILITIES_BLUETOOTH.md — Workstream D27: Portable Bluetooth Authorization

## Objective

Add a portable status contract for Bluetooth authorization and bounded central discovery. Do not add or claim connection, advertisement, peripheral, or radio-control behavior.

D29 adds the central-discovery extension in [PLAN_IOS_BLUETOOTH_DISCOVERY.md](PLAN_IOS_BLUETOOTH_DISCOVERY.md).

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 portable API conventions are integrated

## Write scope

- `crates/framework-bluetooth/**`
- `docs/capabilities/bluetooth.md`

Do not edit the root workspace or lockfile, canonical capability data, aggregate plans or indexes, CI, iOS backend crates, Swift ABI, C bindings, or unrelated capability families.

## Required contract

- Provide fixed-width portable `BluetoothAuthorization` values for Unknown, NotDetermined, Restricted, Denied, and Allowed
- Preserve those states as distinct values; map unknown native states to Unknown
- Provide a no-std, allocation-free, statically selected backend contract and thin caller-owned facade for a non-prompting authorization query
- Provide fixed-width peer-ID/RSSI values and a statically selected central backend contract with explicit unfiltered scan start/stop, poll of later callback results, scan lifecycle state, and dropped-event count
- State that authorization is only a status snapshot and does not report radio power, platform/role support, scan results, or connection success
- Require no global registry, dynamic dispatch, executor, manager lifecycle, or platform type in the portable API
- Include no permission request, connecting, advertising, peripheral, beacon, or Bluetooth radio-control operation. An explicit iOS scan start may prompt through CoreBluetooth.

## Validation and handoff

- Add deterministic tests for fixed-width status semantics and fake-backend query behavior
- Run `cargo fmt --all -- --check`, `cargo test -p framework-bluetooth`, `cargo check -p framework-bluetooth --no-default-features`, and `git diff --check`
- Audit the portable API for `std`, allocation, Apple types, dynamic dispatch, and hidden initialization
- For the scan contract, state that start returns before asynchronous readiness/results, and that the iOS backend bounds and counts dropped discoveries
- See [PLAN_IOS_BLUETOOTH_DISCOVERY.md](PLAN_IOS_BLUETOOTH_DISCOVERY.md) for native lifetime and scan limits
- Report exact checks, API limits, changed files, deviations, and unresolved assumptions
