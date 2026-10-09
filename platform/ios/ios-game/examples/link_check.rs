#[cfg(target_os = "ios")]
fn main() {
    use framework_game::LocalPlayer;
    use ios_game::IosGameCenterBackend;

    let player = LocalPlayer::new(IosGameCenterBackend::new());
    std::hint::black_box(player.authentication_status());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
