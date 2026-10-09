#ifndef FRAMEWORK_IOS_SPRITEKIT_H
#define FRAMEWORK_IOS_SPRITEKIT_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosSpriteKitNode FrameworkIosSpriteKitNode;

/*
 * This header is opt-in through the framework-c-api Cargo feature `ios-spritekit`.
 * On iOS, it creates one detached SKNode and reads or replaces its parent-local
 * position only. All operations that use a node, including create and destroy,
 * must run on the main thread. Off-main position operations return
 * FRAMEWORK_STATUS_UNAVAILABLE. Off-main destroy returns that status without
 * reading or changing the caller's handle slot; retry destroy on the main thread.
 *
 * Each successful create returns one unique opaque handle. Do not copy it, call
 * another operation on it concurrently, race destroy, or destroy an alias.
 * On iOS main, destroy clears the original slot before drop; a null slot or
 * null handle is a no-op that returns FRAMEWORK_STATUS_OK.
 *
 * Coordinates are finite parent-local values. Non-finite inputs return
 * FRAMEWORK_STATUS_INVALID_ARGUMENT. Create initializes out_node to NULL before
 * work. Get initializes each non-null output to 0.0 before work; both outputs
 * are required and are written only on FRAMEWORK_STATUS_OK. On non-iOS targets,
 * valid operations return FRAMEWORK_STATUS_UNSUPPORTED and create leaves the
 * output handle NULL. The non-iOS destroy stub does not inspect or change its
 * slot. No SKNode pointer or Objective-C type crosses this header.
 */
FrameworkStatus framework_ios_spritekit_node_create(
    double x,
    double y,
    FrameworkIosSpriteKitNode **out_node);

FrameworkStatus framework_ios_spritekit_node_get_position(
    const FrameworkIosSpriteKitNode *node,
    double *out_x,
    double *out_y);

FrameworkStatus framework_ios_spritekit_node_set_position(
    FrameworkIosSpriteKitNode *node,
    double x,
    double y);

FrameworkStatus framework_ios_spritekit_node_destroy(
    FrameworkIosSpriteKitNode **node);

#ifdef __cplusplus
}
#endif

#endif
