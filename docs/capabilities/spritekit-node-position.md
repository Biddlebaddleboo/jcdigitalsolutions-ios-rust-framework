# SpriteKit node position

`framework-spritekit` defines `SpriteNodePosition`, a no-std finite point, and a static `SpriteNodePositionBackend` contract for one `SKNode.position` in its parent's coordinate system. The host scene defines the coordinate unit; the value does not imply pixels, view coordinates, transforms, or display.

The contract validates that both coordinates are finite. A backend may further reject a value outside its native scalar range. It does not define a scene, renderer, node tree, visual content, animation, physics, hit testing, or cross-platform SpriteKit behavior.
