#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = ios_runtime::main_thread::MainThread::current() else {
        return;
    };
    std::hint::black_box(ios_vpn::request_personal_vpn_status(main_thread));
    let Some(configuration_main_thread) = ios_runtime::main_thread::MainThread::current() else {
        return;
    };
    std::hint::black_box(ios_vpn::request_personal_vpn_configuration(
        configuration_main_thread,
    ));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
