#!/bin/sh
set -eu

package=platform/ios/ios-app-lifecycle
source="$package/src/platform.rs"
queue="$package/src/queue.rs"
mapping="$package/src/mapping.rs"

for symbol in \
    UIApplicationDidBecomeActiveNotification \
    UIApplicationWillResignActiveNotification \
    UIApplicationWillEnterForegroundNotification \
    UIApplicationDidEnterBackgroundNotification \
    UIApplicationWillTerminateNotification \
    UIApplicationOpenSettingsURLString \
    URLWithString \
    openURL_options_completionHandler \
    open_application_settings \
    addObserverForName_object_queue_usingBlock \
    NSOperationQueue::mainQueue \
    ios_runtime::ffi::catch_unwind \
    main_thread.into_objc2 \
    ObserverOwner \
    removeObserver
do
    if ! rg -Fq "$symbol" "$source"; then
        printf 'missing required lifecycle surface: %s\n' "$symbol" >&2
        exit 1
    fi
done

for symbol in \
    EVENT_QUEUE_CAPACITY:\ usize\ =\ 16 \
    LifecycleEvent::TerminationRequested
do
    if ! rg -Fq "$symbol" "$queue"; then
        printf 'missing required lifecycle queue behavior: %s\n' "$symbol" >&2
        exit 1
    fi
done

for symbol in ErrorKind::Platform PlatformErrorCode::new\(value\); do
    if ! rg -Fq "$symbol" "$mapping"; then
        printf 'missing native state error mapping: %s\n' "$symbol" >&2
        exit 1
    fi
done

if find "$package" -type f -name '*.swift' -print -quit | rg -q .; then
    printf 'Swift source is forbidden in the iOS framework package\n' >&2
    exit 1
fi
