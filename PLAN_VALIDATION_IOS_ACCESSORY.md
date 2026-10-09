# PLAN_VALIDATION_IOS_ACCESSORY.md — Workstream G38: ExternalAccessory Gates

## Objective

Gate D39/B44's current connected-list presence snapshot without running accessory communication.

## Required gates

- Package formatting, portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator compile plus strict Clippy
- Source guard for the three allowed native calls and the absence of picker, event, session, protocol, stream, or transfer APIs
- Release device and Simulator link/import audit for `ExternalAccessory`, Foundation, libobjc, and libSystem; inspect Objective-C symbols and exclude Swift runtime symbols
- Repository docs, zero-Swift-source, and whitespace checks

The probe is linked, not run. No live list result, connected hardware, protocol eligibility, or communication is claimed.
