# PLAN_CAPABILITIES_SENSORKIT.md — Workstream D68: Row 042 Feasibility Gate

## Status

D68 found a public, non-prompting, per-sensor authorization query in the installed iOS SDK, but SensorKit requires a research-study entitlement that Apple grants only after study approval. Apple's current documentation also deprecates the installed `SRSensorReader` API in favor of a beta `SRReader<Sensor>` API that is absent from the inspected SDK. Do not treat the query as general sensor support. Keep row 042 at `X`; no implementation or matrix change is part of D68.

## Objective

Determine whether SensorKit has a narrow public authorization snapshot that can be used as ordinary app capability support, without a prompt, sample access, or special research eligibility.

## Installed SDK and binding evidence

- Inspected Xcode 26.6 build 17F113 and the iPhoneOS 26.5 SDK. `SensorKit.framework/Headers/SRAuthorization.h` declares `SRAuthorizationStatus` from iOS 14.0 with values `NotDetermined = 0`, `Authorized = 1`, and `Denied = 2`. The header defines `Denied` as a user denial or disabled collection in Settings.
- `SensorKit.framework/Headers/SRSensorReader.h` declares `SRSensorReader` from iOS 14.0, `initWithSensor:`, and the read-only `authorizationStatus` property. A reader is bound to one `SRSensor`; the property reports authorization for that reader's sensor. Reading this property does not call the separate `requestAuthorizationForSensors:completion:` method and does not itself present the authorization prompt.
- The installed headers have no `API_DEPRECATED` annotation on `SRSensorReader`. Current Apple documentation marks `SRSensorReader` deprecated and directs developers to `SRReader<Sensor>`, which Apple labels beta. The installed 26.5 SDK has no `SRReader` declaration in its SensorKit headers or Swift module interface.
- `objc2-sensor-kit` 0.3.2 is available on docs.rs and exposes generated `SRSensorReader::initWithSensor`, `SRSensorReader::authorizationStatus`, `SRAuthorizationStatus`, and sensor constants. It is not present in the current workspace, `Cargo.lock`, or local Cargo source cache, so no local feature or compile audit was run.

## Entitlement and privacy boundary

- Apple says the OS requires `com.apple.developer.sensorkit.reader.allow` in the signed app to use SensorKit. This entitlement is an array of permitted sensor identifiers; the OS closes the app if its code signature lacks the entitlement.
- Apple grants the SensorKit entitlement only for a preapproved research study. The developer must submit a research proposal, receive Apple approval, and use the approved entitlement in a provisioning profile. This is not an ordinary user permission or a generally available app capability.
- The reader's `authorizationStatus` is a user-agreement status for one sensor. `.Authorized` does not prove that the signed app has the required entitlement, is approved to distribute, can read every sensor, or has data available. The entitlement and per-sensor user choice are separate gates.
- `NSSensorKitUsageDetail` is a dictionary of per-sensor explanations used when the system prompts for access. The header describes this metadata on the authorization-request path; D68 does not claim it is required for a status-only read.
- Do not request authorization, access samples, start or stop recording, fetch device data, or probe the API at runtime as part of D68.

## Feasibility result and next evidence

The status property is technically narrow and non-prompting, but its use depends on a special Apple-approved research entitlement. That fails the requested ordinary, general-purpose SensorKit slice. Do not add a crate that invites host apps to use the API without the entitlement, and do not claim that a successful authorization value equals entitlement approval.

Before any later implementation, establish all of the following:

1. A product scope limited to an Apple-approved research study and an explicit list of sensor identifiers in `com.apple.developer.sensorkit.reader.allow`.
2. A supported, non-beta API for the chosen SDK baseline. Either obtain a stable supported replacement for `SRSensorReader` or explicitly accept the deprecated iOS 14 API with a documented maintenance window; do not use beta `SRReader<Sensor>` in a shipping V1 contract.
3. A local `objc2-sensor-kit` dependency/feature audit and device/Simulator compile evidence for only the selected sensor reader and status values.
4. A per-sensor contract that preserves unknown native status values and does not infer entitlement approval, sensor availability, sample access, or data presence from the authorization state.

## Deferred work

- No portable `framework-motion` change, SensorKit data API, Rust backend, entitlement integration, `NSSensorKitUsageDetail` host configuration, or B/G gate is authorized by D68.
- Row 042 remains `X`; existing Core Motion support in row 038 is separate and does not depend on SensorKit.
- No canonical status JSON, Cargo/workspace/lockfile, CI, aggregate plan, or shared index edit is part of D68.

## Apple and binding references

- [SRSensorReader](https://developer.apple.com/documentation/sensorkit/srsensorreader)
- [SRSensorReader.authorizationStatus](https://developer.apple.com/documentation/sensorkit/srsensorreader/authorizationstatus)
- [SRAuthorizationStatus](https://developer.apple.com/documentation/sensorkit/srauthorizationstatus)
- [SensorKit reader entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.sensorkit.reader.allow)
- [Configuring your project for sensor reading](https://developer.apple.com/documentation/sensorkit/configuring-your-project-for-sensor-reading)
- [NSSensorKitUsageDetail](https://developer.apple.com/documentation/bundleresources/information-property-list/nssensorkitusagedetail)
- [SRReader](https://developer.apple.com/documentation/sensorkit/srreader)
- [`SRSensorReader` in objc2-sensor-kit 0.3.2](https://docs.rs/objc2-sensor-kit/0.3.2/objc2_sensor_kit/struct.SRSensorReader.html)
