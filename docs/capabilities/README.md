# Capability support

[`capability-status.json`](capability-status.json) is the canonical machine-readable and
human-readable support manifest. It has 113 capability rows across 15 families. Each row names a
crate/module, portability class, iOS implementation class, verified platform metadata, parity and
performance status, and native-escape status.

| iOS class | Meaning |
| --- | --- |
| `R` | Portable Rust implementation selected after correctness and performance evidence |
| `M` | Rust semantics/state over a minimal public Apple primitive |
| `B` | Apple/system-owned backend reached through Rust |
| `A` | Apple implementation preferred for performance or hardware reasons |
| `C` | Compiler/build/discovery contract |
| `X` | No supported implementation in this workstream; reason is in the row |

Current counts: four rows have partial `B` support for the UIKit app example; 109 rows are `X` for
iOS runtime support. No row claims a complete capability backend. The four D1 portable contracts
cover application lifecycle, sandbox files/directories, preferences, and foreground HTTP values.
The D2 secure-storage contract covers opaque bytes and explicit protection requirements, but does
not provide a Keychain backend. The UIKit slice remains partial: a Rust-owned app delegate, one
window, basic views, and one target/action callback only. The manifest counts seven portable
contracts as implemented and one as partial; none is a complete iOS capability backend.

| Family | Rows | iOS class count |
| --- | ---: | --- |
| Core app/UI | 9 | `B`: 4 partial; `X`: 5 |
| Files/data/preferences | 7 | `X`: 7 |
| Security/auth | 7 | `X`: 7 |
| Networking/web | 7 | `X`: 7 |
| Notifications/background | 6 | `X`: 6 |
| Sensors/connectivity | 9 | `X`: 9 |
| Camera/audio/media | 8 | `X`: 8 |
| Graphics/GPU | 9 | `X`: 9 |
| ML/vision/language | 5 | `X`: 5 |
| Personal data/system stores | 8 | `X`: 8 |
| Cloud/accounts/communication | 10 | `X`: 10 |
| Commerce/services | 7 | `X`: 7 |
| Maps/AR/spatial | 5 | `X`: 5 |
| Extension/entitlement capabilities | 12 | `X`: 12 |
| Compiler/build-host capabilities | 4 | `X`: 4 |

For unverified platform metadata the manifest uses `null`, not an inferred empty requirement. A
verified empty list means the row's source documents no required item for that field. The B UI
facts come from [`docs/ios/runtime.md`](../ios/runtime.md) and the `ios-minimal` example; no iOS
minimum version is stated there, so the manifest leaves it unknown. No entitlement, permission, or
framework requirement is inferred for an `X` row. A `true` native-escape value on a partial `B` row
means the example uses direct UIKit handles; it does not mean a reusable capability-level `ios`
escape facade exists.

See [the D1 API guide](app-data.md) for the four portable crate contracts and
[the secure-storage guide](secure-storage.md) for D2's opaque-byte contract. These guides detail
ownership, copy, atomicity, async/cancellation, errors, and runtime limits. Later D capability
groups need separate named subplans and executors; D1 and D2 do not claim Workstream D complete.
