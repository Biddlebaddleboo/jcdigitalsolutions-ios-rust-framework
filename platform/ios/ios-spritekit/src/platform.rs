use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use framework_spritekit::{SpriteNodePosition, SpriteNodePositionBackend};
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Allocated, Retained};
use objc2::{MainThreadMarker, MainThreadOnly, extern_class, extern_methods};
use objc2_core_foundation::{CGFloat, CGPoint};
use objc2_ui_kit::UIResponder;

/// A main-thread-owned, retained SpriteKit node whose only facade operation is its position.
///
/// The node is detached when created. The wrapper creates no scene, view, renderer, child, or
/// visual content. The handle is neither `Send` nor `Sync` because SpriteKit declares `SKNode` as
/// main-actor isolated and Objective-C release must remain on the owning thread.
pub struct IosSpriteNode {
    native: Retained<NativeSkNode>,
    _main_thread: MainThreadMarker,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl IosSpriteNode {
    /// Creates one detached `SKNode` at a finite parent-local position.
    pub fn new(main_thread: MainThread, position: SpriteNodePosition) -> Result<Self> {
        let marker = main_thread.into_objc2();
        let point = native_point(position)?;
        let native = unsafe { NativeSkNode::init(NativeSkNode::alloc(marker)) };
        // SAFETY: The public SKNode header declares `position` as CGPoint and `setPosition:` as
        // its matching setter. The typed CGPoint is valid, and the receiver is alive and main-
        // thread-bound for this call.
        unsafe { native.set_position(point) };
        Ok(Self {
            native,
            _main_thread: marker,
            _not_send_sync: PhantomData,
        })
    }

    /// Borrows the native node for caller-owned SpriteKit integration.
    ///
    /// The borrow cannot outlive this wrapper. The caller remains responsible for any scene or
    /// parent hierarchy and must use the node on the main thread.
    pub fn native_node(&self) -> &NativeSkNode {
        &self.native
    }
}

impl SpriteNodePositionBackend for IosSpriteNode {
    fn position(&self) -> Result<SpriteNodePosition> {
        let _marker = current_main_thread()?;
        // SAFETY: The public SKNode header declares `position` as a CGPoint getter. The receiver
        // remains retained by `self`, and this method verifies main-thread context.
        let point = unsafe { self.native.position() };
        SpriteNodePosition::new(point.x, point.y)
    }

    fn set_position(&mut self, position: SpriteNodePosition) -> Result<()> {
        let _marker = current_main_thread()?;
        let point = native_point(position)?;
        // SAFETY: The public SKNode header declares `setPosition:` with one CGPoint argument.
        // `native_point` ensures finite coordinates are representable by this target's CGFloat,
        // and the retained receiver is live.
        unsafe {
            self.native.set_position(point);
        }
        Ok(())
    }
}

fn native_point(position: SpriteNodePosition) -> Result<CGPoint> {
    let x = position.x() as CGFloat;
    let y = position.y() as CGFloat;
    if !x.is_finite() || !y.is_finite() {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(CGPoint::new(x, y))
}

fn current_main_thread() -> Result<MainThreadMarker> {
    MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Platform))
}

extern_class!(
    /// Minimal typed declaration of the public iOS SpriteKit `SKNode` class.
    #[unsafe(super(UIResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "SKNode"]
    pub struct NativeSkNode;
);

#[allow(non_snake_case)]
impl NativeSkNode {
    extern_methods!(
        // SAFETY: The iOS 26.5 public `SKNode.h` declares `-init` as its designated initializer.
        #[unsafe(method(init))]
        #[unsafe(method_family = init)]
        pub(super) unsafe fn init(this: Allocated<Self>) -> Retained<Self>;

        // SAFETY: These typed declarations match the public Objective-C `CGPoint position`
        // property and its `setPosition:` setter in the iOS 26.5 `SKNode.h`.
        #[unsafe(method(position))]
        pub(super) unsafe fn position(&self) -> CGPoint;

        #[unsafe(method(setPosition:))]
        pub(super) unsafe fn set_position(&self, position: CGPoint);
    );
}
