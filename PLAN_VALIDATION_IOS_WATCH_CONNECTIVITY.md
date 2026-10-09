# PLAN_VALIDATION_IOS_WATCH_CONNECTIVITY.md — Workstream G37: Watch Connectivity Gates

## Objective

Gate D38/B43's platform support Boolean without session lifecycle or watch communication.

## Required gates

- portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator check, strict Clippy, and iOS rustdoc
- source guard allowing only `WCSession::isSupported()` in the adapter
- Release import audit for WatchConnectivity, Foundation, libobjc, and libSystem
- docs, zero-Swift-source, and diff checks

The binary probes are not executed. No paired-device, activation, reachability, delivery, or runtime claim is made.
