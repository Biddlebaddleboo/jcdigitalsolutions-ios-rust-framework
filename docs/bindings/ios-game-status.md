# Game Center local-player status

The opt-in `ios-game-status` feature exposes one synchronous B55 snapshot:

```c
FrameworkStatus framework_ios_game_status_is_local_player_authenticated(
    uint8_t *out_authenticated);
```

On iOS, `FRAMEWORK_STATUS_OK` writes exactly zero or one from
`GKLocalPlayer.localPlayer.isAuthenticated`. A false result is a momentary value and can include an
offline temporary player or an uninitialized Game Center service; it does not prove that no account
exists. This query does not initialize authentication, install an auth handler, present UI, observe
status changes, or return player identity/data

The output is caller-owned, valid, aligned, and writable for one byte through the full synchronous
call. The API checks nullness only and does not retain the pointer. The caller must prevent
unsynchronized access. A null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A
valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` with zero output. A future unrecognized
portable status returns `FRAMEWORK_STATUS_UNAVAILABLE` with zero output. A caught panic returns
`FRAMEWORK_STATUS_PANIC` with zero output

The SDK API floor is iOS 4.1. F33's device and Simulator link probes use minos 10.0 and 14.0; those
are link settings, not the API floor. A configured Game Center app still needs the signed
`com.apple.developer.game-center` entitlement. This query does not validate that entitlement or
service readiness. No usage-description key is needed for the read

The local C/C++ target links imported Foundation, GameKit, `libSystem.B.dylib`, and
`libobjc.A.dylib`; host links imported only `libSystem.B.dylib`. The linked binaries were inspected,
not executed. This does not verify signed entitlement setup or live Game Center behavior

The header and gates are in `bindings/c`; the exact source, imports, and evidence limits are in
`PLAN_BINDINGS_F33.md`. The gates build and inspect C11/C++17 consumers but never execute them. They
run no tests or live Game Center query
