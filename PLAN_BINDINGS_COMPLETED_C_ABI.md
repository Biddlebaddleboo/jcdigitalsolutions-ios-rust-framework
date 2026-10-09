# PLAN_BINDINGS_COMPLETED_C_ABI.md — closed F17–F22 slices

This is a bounded completion ledger, **not** a claim of end-to-end device runtime validation or full Apple framework support. Six implementation workstream plans are retired; original exact contracts, acceptance checks and failure history remain in Git at `ec1d9d34af1808c970f1f8fe86418d720f654a99`. Current sources under `bindings/c/**`, `docs/VALIDATION.md`, and `docs/capabilities/capability-status.json` remain canonical.

- **F17** (`PLAN_BINDINGS_F17.md`): Vision text-recognition revision membership; public iOS backend; 13.0 device/14.0 simulator; static pointer and import gates. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F17.md`.
- **F18** (`PLAN_BINDINGS_F18.md`): ProximityReader device-model support snapshot; iOS 15.4; no reader/payment or session. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F18.md`.
- **F19** (`PLAN_BINDINGS_F19.md`): CommonCrypto one-shot SHA-256, checked 32-bit input length; iOS 10/14 target gates. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F19.md`.
- **F20** (`PLAN_BINDINGS_F20.md`): ModelIO extension importability Boolean; no asset loading; fixed scalar C ABI. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F20.md`.
- **F21** (`PLAN_BINDINGS_F21.md`): Sign in with Apple prior-user credential-state asynchronous query; distinguish NSError presence; no sign-in or cancellation promise. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F21.md`.
- **F22** (`PLAN_BINDINGS_F22.md`): Accelerate equal-length f32 vector addition, checked borrowed ranges and vDSP_Length; no speed/parity claim. Original plan: `ec1d9d34af1808c970f1f8fe86418d720f654a99:PLAN_BINDINGS_F22.md`.

Each slice reports an integrated root C ABI feature and its focused `bindings/c/check-ios-*.sh` gate. Records establish compilation, static-link checks, import/selector allowlists, C11/C++17 consumer links and no-unwanted-feature checks; **the binaries were not run**. Preserve pointer/length/alignment preconditions, initialization of outputs, no pointer retention, exact platform availability, error mapping, async completion ownership and feature isolation when extending them. F21's async operation is an Objective-C callback bridge, **not** proof of a general Swift async task-entry API.

## Remaining active work

Keep `PLAN_BINDINGS.md` and every other F-series plan active pending their own proof. In particular, do not retire F23 or F24 on the basis of another branch's readiness, nor assume that new calls in F25–F33 have finished integration. R2 validation may consolidate mechanical scripts but must preserve the distinct negative ABI assertions and all static-versus-runtime evidence limitations.
