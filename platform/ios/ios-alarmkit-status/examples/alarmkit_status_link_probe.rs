fn main() {
    let probe: fn() -> Result<
        ios_alarmkit_status::AlarmAuthorizationState,
        ios_alarmkit_status::AlarmKitError,
    > = ios_alarmkit_status::authorization_state;
    core::hint::black_box(probe);
}
