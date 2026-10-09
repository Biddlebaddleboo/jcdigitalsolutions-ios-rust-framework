#[cfg(target_os = "ios")]
use framework_spritekit::{SpriteNodePosition, SpriteNodePositionBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_spritekit::IosSpriteNode;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let position = SpriteNodePosition::new(3.0, -2.0).expect("finite sample position");
    let mut node = IosSpriteNode::new(main_thread, position).expect("SKNode init");
    let _snapshot = node.position().expect("position getter");
    let _ = node.set_position(position);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
