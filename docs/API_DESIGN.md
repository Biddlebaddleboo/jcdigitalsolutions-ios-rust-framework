# API Design Rules

## Goal

Make native iOS programming from Rust substantially less cumbersome than raw objc2 without constructing a second iOS framework on top of UIKit.

## Default rule: no wrapper

If the raw objc2 API is already ergonomic, safe enough, and low-friction, expose/use it directly.

Add a helper only when it removes repeated meaningful friction.

## Good reasons to add an abstraction

- repeated Objective-C lifetime boilerplate;
- repeated target/action or delegate glue;
- Block lifecycle boilerplate;
- main-thread invariant encoding;
- error-conversion boilerplate;
- common native string/data handling;
- a genuinely portable application concept;
- a measured optimization.

## Bad reasons

- hiding Apple naming for cosmetic reasons;
- creating a framework object for every UIKit object;
- forcing every API through a lowest-common-denominator portability layer;
- making an uncommon native feature wait for a wrapper;
- replacing static dispatch with trait-object dispatch without need;
- adding builders that allocate purely for style.

## Cost transparency

A convenience API should document when it:

- allocates;
- retains/releases;
- copies;
- transcodes;
- boxes;
- locks;
- hops threads;
- stores callback state.

Prefer APIs where simple operations inline down to the corresponding objc2/native operation.

## Native escape

Wrappers must expose the underlying native object through a safe borrowed accessor when possible.

Provide raw unsafe handles only when a higher-level native accessor cannot express the required operation.

## Ergonomics target

Common code should feel closer to idiomatic native application development than low-level Objective-C ABI work, but source brevity is secondary to keeping runtime cost obvious and small.

## No duplicated UI semantics

Do not recreate UIKit properties, layout, accessibility, navigation, scrolling, or event semantics in Rust unless there is a concrete reason. Wrap recurring boilerplate; do not replace the platform.
