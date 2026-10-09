# StoreKit 2 purchase-ability status

The opt-in `ios-storekit2-status` feature exposes one synchronous Boolean from B63:

```c
FrameworkStatus framework_ios_storekit2_status_can_make_payments(
    uint8_t *out_can_make_payments);
```

The API and symbol availability floor is iOS 15.0. B63 links StoreKit and the Swift getter as weak
imports; its device and Simulator probes use minos 10.0 and 14.0. Those minos are link settings, not
the API floor. If the weak symbol is absent, B63 returns false without a call. Static compiler/link
evidence supports that fallback; no runtime check below iOS 15.0 was made

On iOS, `FRAMEWORK_STATUS_OK` writes exactly zero or one. True means StoreKit reports that the person
can authorize purchases. False may mean purchase restrictions or the absent weak symbol. This does
not report product availability, account identity, entitlement state, transaction success, or
payment readiness. No product, transaction, purchase, restore, payment UI, account, or network API is
called

On a valid non-iOS call, the function writes zero and returns
`FRAMEWORK_STATUS_UNSUPPORTED`. A null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a
write. A caught panic returns `FRAMEWORK_STATUS_PANIC` with zero output

The output is caller-owned, valid, aligned, and writable for one byte through the full synchronous
call. The API checks nullness only and does not retain the pointer. The caller must prevent
unsynchronized access. The call runs on the caller's thread with no added queue or thread-safety
promise

The header and focused gates are in `bindings/c`; exact scope, source evidence, imports, and minos
limits are in `PLAN_BINDINGS_F32.md`. The link gate builds and inspects C11/C++17 consumers but does
not execute them. It runs no tests or live StoreKit query

The local C11/C++17 host links imported only `libSystem.B.dylib`. Device and Simulator links imported
weak `StoreKit.framework` plus `libSystem.B.dylib`, with the getter symbol marked weak; their minos
were 10.0 and 14.0. These are link-shape results only. The StoreKit 2 API/symbol floor remains
iOS 15.0, and the absent-symbol path was not runtime-tested below that floor
