# Objective-C Interoperability

## Default stack

Use `objc2` for Objective-C runtime integration and generated Apple-framework bindings.

Use `block2` for Apple Blocks.

The goal is to participate directly in the same Objective-C object model used by native UIKit code rather than introducing a bridge runtime.

## Why objc2 is the default

It provides:

- typed Objective-C messaging;
- ownership-aware object wrappers;
- Objective-C class/protocol implementation from Rust;
- selector support;
- main-thread-aware APIs;
- Apple framework bindings;
- Block integration through the companion ecosystem.

When optimized, an ordinary call should preserve essentially the same Objective-C message-dispatch topology as native Objective-C code.

## Rust-defined Objective-C classes

Use `define_class!` for narrow bridge objects where Apple requires an Objective-C object.

Likely examples:

- application delegate;
- button/control target;
- table data source;
- table delegate;
- text-field delegate;
- notification delegate;
- navigation delegate.

Do not subclass merely to make Rust state accessible.

## Target/action

Common events should avoid universal heap-boxed closures.

A likely efficient design is:

- native Objective-C target object;
- compact callback ID and/or function pointer;
- Rust registry/state locator;
- stale-generation checking where required.

Measure the dispatch path against Objective-C and Swift baselines.

## Delegates

Delegate objects should hold only the minimum context needed to locate Rust state.

Respect UIKit weak/strong delegate ownership conventions.

## Blocks

Use the cheapest valid Block representation.

Distinguish:
- no-capture/global;
- borrowed/nonescaping;
- escaping/retained.

Do not retain callback environments unnecessarily.

## NSString/NSData

Treat conversion as an explicit design decision, not an automatic convenience.

Keep borrowed/native forms when they avoid unnecessary allocation or transcoding.

## Going below objc2

Allowed only when a binding limitation or measured overhead justifies it.

A manual Objective-C ABI path must include ABI tests or disassembly review and must preserve exactly the ownership semantics required by the Apple API.
