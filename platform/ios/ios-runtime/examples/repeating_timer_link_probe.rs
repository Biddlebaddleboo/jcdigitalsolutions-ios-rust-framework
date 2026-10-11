#[cfg(target_os = "ios")]
fn main() {
    use core::time::Duration;
    use dispatch2::DispatchQueue;
    use ios_runtime::repeating_timer::RepeatingTimer;

    let timer = RepeatingTimer::schedule_on(
        DispatchQueue::main(),
        Duration::from_secs(60),
        Duration::from_millis(250),
        |context| {
            let _ = context.cancel();
        },
    )
    .expect("valid timer schedule");
    assert!(timer.cancel());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
