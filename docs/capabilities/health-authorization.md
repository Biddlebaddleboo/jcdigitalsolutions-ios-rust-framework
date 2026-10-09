# Health data authorization

`framework-health-authorization` is a portable `#![no_std]` contract for checking whether a HealthKit data store is available and making an explicit, type-scoped read/share authorization request. It does not query, read, write, or retain health samples.

## Request model

`HealthAuthorizationRequest` borrows separate slices for `read_types` and `share_types`. A `HealthDataType` pairs an Apple HealthKit type-family tag with its UTF-8 identifier; identifiers are borrowed and must be non-empty and contain no NUL character. The workout type is a fixed value created with `HealthDataType::workout()`. Empty requests are invalid. The iOS backend validates identifiers with the corresponding public HealthKit type factory. An identifier can be valid UTF-8 and still be unknown or unavailable on the running OS.

The initial family set is quantity, category, characteristic, correlation, and workout. Characteristics may be requested for read only; HealthKit's share set accepts sample types. Document types, series types, scored assessments, clinical-record types, per-object authorization, and HealthKit query/write APIs are not part of this contract.

## Availability and completion semantics

`HealthAuthorizationBackend::is_available` reports only the platform's HealthKit data-store availability Boolean. A false result can reflect device/OS support or a restricted environment; this contract does not classify the cause. Availability does not indicate that the app has a HealthKit entitlement or any authorization.

The caller must invoke `request_authorization` explicitly. Immediate `Ok(())` means only that the backend accepted the request for native processing. The completion callback's `Ok(())` means only that native request processing completed without a reported error. It is not a grant result. The backend may invoke the callback inline or later on its own execution context and must invoke it at most once. There is no completion deadline, no guarantee of delivery if the process exits or the platform does not complete the request, and no portable cancellation operation.

HealthKit intentionally hides read denial: an app cannot determine whether a person granted read access for a type. The API therefore has no read-authorization query or per-type grant state. It does not call or expose `HKHealthStore.authorizationStatus(for:)`, which reports sharing status, and it does not expose or map the authorization request callback `Bool` to a read or share grant. A successful completion cannot be treated as evidence of access.

## Ownership and dependencies

The contract owns no native object, heap allocation, string, or callback state. Its request borrows caller-owned slices for the duration of the call. Backend selection is static through `HealthAuthorizationBackend`; there is no registry, boxed backend, global initialization, or executor requirement. Native error mapping preserves a portable `ErrorKind` and optional nonzero platform code through `HealthAuthorizationError::Backend`.

## Explicit limits

- No health sample read/write/query, saved-data access, history, observer query, background delivery, workout session, clinical record, per-object authorization, authorization-status query, or entitlement discovery
- No promise that HealthKit shows a prompt; the prompt may be skipped after prior choices
- No read-grant or share-grant result in the request callback
- No inference from an empty result set, a callback `Bool`, or a sharing-status API
- No cross-platform normalization for health databases that do not exist on another platform
