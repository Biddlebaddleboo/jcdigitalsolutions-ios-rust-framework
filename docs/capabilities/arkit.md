# ARKit world-tracking support value

`framework-maps::WorldTrackingSupport` is an owned, portable boolean result for a platform
support query. It carries no ARKit object, camera frame, session, permission state, or tracking
result. The capability is partial: the portable value describes one support snapshot only.

The iOS backend is documented in the [iOS ARKit guide](../ios/arkit.md). This value does not
claim that world tracking is active or that a later session can track a particular environment.
