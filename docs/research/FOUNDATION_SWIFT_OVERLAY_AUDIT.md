# Foundation Swift Overlay Elimination Audit

Research date: 2026-10-07

This pass focuses on common Foundation value types that appear in Swift signatures but already bridge to Objective-C/CoreFoundation types.

The practical question is:

> When a Swift-only Apple API mentions a Foundation Swift value type, does Rust truly need to construct/manipulate that Swift value directly, or can it reuse an Objective-C/CoreFoundation representation?

## Executive finding

A substantial set of common Foundation Swift value types are explicitly documented as bridging to Objective-C classes.

Examples confirmed:

| Swift value type | Objective-C/native counterpart | Research conclusion |
|---|---|---|
| `Data` | `NSData` / `CFData` | Prefer native byte buffer/NSData bridging before custom Swift Data layout work |
| `Date` | `NSDate` / `CFDate` | Prefer Objective-C/CoreFoundation representation |
| `URL` | `NSURL` / `CFURL` | Prefer native NSURL/CFURL when API bridging permits |
| `UUID` | `NSUUID` | Prefer NSUUID or raw 16-byte Rust UUID representation |
| `IndexPath` | `NSIndexPath` | Prefer NSIndexPath for UIKit/native boundary |
| `CharacterSet` | `NSCharacterSet` / `CFCharacterSet` | Prefer native bridged representation |
| `Decimal` | `NSDecimalNumber` / NSDecimal | Prefer native/Foundation decimal path when Apple API needs Foundation decimal |
| `URLRequest` | `NSURLRequest` / `NSMutableURLRequest` | Prefer Objective-C request objects for native URL loading |
| `Set<T>` for bridgeable/object T | `NSSet` | Evaluate direct bridge instead of implementing Swift Set layout |
| Foundation errors | `NSError` / `CFError` | Preserve NSError/native error where bridge exists |

This does not mean every Swift ABI call will accept the Objective-C object pointer directly. It means the **language bridge already exists in Foundation**, and the ABI research should first determine whether the compiler/runtime performs that bridge at the call boundary before implementing Swift-native storage manually.

# Data / NSData

Apple explicitly documents:

- `Data` is the Swift overlay value type;
- it bridges to `NSData` and `NSMutableData`;
- `NSData` is toll-free bridged to `CFData`.

Reference:
https://developer.apple.com/documentation/foundation/nsdata?language=objc

## Framework consequence

Rust already has ideal byte-buffer representations:
- `&[u8]`;
- `Vec<u8>`;
- memory-mapped/native buffers.

For Objective-C APIs, use `NSData` only where Apple requires it.

For Swift-only APIs taking `Data`:
1. inspect whether the function ABI expects the bridged Swift struct directly;
2. inspect compiler lowering from NSData -> Data where relevant;
3. avoid copying when Foundation can reference existing bytes safely;
4. never convert Rust bytes -> NSData -> Rust Data model -> serialized bytes merely for convenience.

The existence of the bridge suggests that generic "Swift Data support" may be much smaller than implementing the Data representation itself.

# Date / NSDate

Apple documents that Swift `Date` bridges to `NSDate`; `NSDate` is toll-free bridged with `CFDate`.

References:
https://developer.apple.com/documentation/foundation/nsdate
https://developer.apple.com/documentation/foundation/date

## Framework consequence

Internally, Rust applications should generally use a compact Rust time representation appropriate to the application.

At Apple boundaries:
- use NSDate/CFDate for Objective-C/C APIs;
- only construct Swift Date directly when a genuinely Swift-only API physically requires its value ABI.

Do not adopt Foundation Date throughout Rust application state merely because Apple APIs use it.

# URL / NSURL

Apple documents that Swift `URL` bridges to `NSURL`; NSURL is toll-free bridged with CFURL.

Reference:
https://developer.apple.com/documentation/foundation/nsurl?language=objc

## Framework consequence

For ordinary:
- networking;
- filesystem URLs;
- deep links;
- UIApplication URL opening;

prefer NSURL/CFURL through objc2/native APIs.

Only the residual Swift ABI layer needs to solve Swift URL when a Swift-only method physically accepts the value type and no bridge thunk can reuse NSURL cheaply.

This matters for:
- AdAttributionKit reengagement URL;
- LockedCameraCapture session URLs;
- Swift-only framework configuration values.

# UUID / NSUUID

Apple explicitly documents that Swift `UUID` bridges to `NSUUID`.

Reference:
https://developer.apple.com/documentation/foundation/nsuuid

## Framework consequence

Rust can keep UUIDs as:
- 16 bytes;
- a normal Rust UUID type;
- NSUUID only at Objective-C boundaries.

For Swift-only methods taking UUID:
- first determine whether compiler bridge machinery can cheaply consume NSUUID;
- otherwise implement the minimal fixed Swift UUID value representation only if ABI-stable/publicly documented enough.

Do not use NSString roundtrips.

# IndexPath / NSIndexPath

Apple explicitly documents that Swift `IndexPath` bridges to `NSIndexPath`.

Reference:
https://developer.apple.com/documentation/foundation/nsindexpath

This is especially relevant to UIKit, where NSIndexPath is already part of the Objective-C API.

## Decision

Use NSIndexPath/objc2 for UIKit.

There is no reason for the core framework UI layer to implement Swift IndexPath ABI.

# CharacterSet / NSCharacterSet

Apple documents that Swift `CharacterSet` bridges to `NSCharacterSet`, which is also toll-free bridged to `CFCharacterSet`.

References:
https://developer.apple.com/documentation/foundation/nscharacterset
https://developer.apple.com/documentation/foundation/characterset

## Decision

Use Objective-C/CoreFoundation at Apple boundaries.

Application-side character classification can remain Rust-native where appropriate.

# Decimal / NSDecimalNumber

Apple documents that Swift `Decimal` bridges to `NSDecimalNumber`.

Reference:
https://developer.apple.com/documentation/foundation/nsdecimalnumber

## Decision

Money/business logic should not automatically use binary floating point merely to avoid Foundation.

Use:
- integer minor units;
- an appropriate Rust decimal type;
- or NSDecimalNumber when Apple APIs specifically require Foundation decimal behavior.

Implement Swift Decimal ABI only for a concrete Swift-only API that requires it.

# URLRequest / NSURLRequest

Apple explicitly states that Swift `URLRequest` bridges to `NSURLRequest` and `NSMutableURLRequest`.

Reference:
https://developer.apple.com/documentation/foundation/nsurlrequest?language=objc

## Consequence

The framework's networking API should not depend on Swift URLRequest.

Use native NSURLRequest/NSMutableURLRequest and NSURLSession.

This reinforces the previous conclusion that Foundation's Swift async/networking overlay is optional syntax, not required capability.

# Set / NSSet

Swift documentation explicitly documents bridging between `Set` and `NSSet` for bridgeable element types.

It notes:
- Swift Set -> NSSet bridge can be O(1) in time/space before element access;
- element bridging may be lazy;
- immutable NSSet -> Set can share storage with copy-on-write bookkeeping.

Reference:
https://developer.apple.com/documentation/swift/set

## StoreKit implication

StoreKit purchase options use Swift `Set<Product.PurchaseOption>`.

This does **not automatically mean NSSet solves the call**, because `Product.PurchaseOption` is itself a Swift value type and may not be Foundation-bridgeable.

But for APIs with sets of:
- strings;
- NSNumber-compatible values;
- Objective-C classes;

the bridge may avoid bespoke Swift Set manipulation.

Therefore Set support should remain concrete-type-driven.

# Dictionary / NSDictionary

Foundation exposes NSDictionary/NSMutableDictionary as its Objective-C map types.

Reference:
https://developer.apple.com/documentation/foundation/nsdictionary?language=objc

Many modern Objective-C APIs already use NSDictionary directly.

For Swift-only APIs using `Dictionary`, inspect element bridgeability before adding native Swift dictionary support.

Do not implement a general Swift Dictionary ABI until a Tier-1 API requires it.

# Error / NSError

NSError remains the public Objective-C error object and toll-free bridges with CFError.

Reference:
https://developer.apple.com/documentation/foundation/nserror

Swift Error is broader than NSError, so not every Swift error is an NSError.

## Framework policy

When a Swift-only API throws:
1. inspect whether the concrete Apple error type bridges to NSError;
2. preserve NSError directly if it does;
3. only build general Swift existential Error handling for APIs whose errors are genuinely non-bridged Swift values.

This could reduce error ABI complexity for many Apple frameworks.

# NSString and Swift String remain special

Unlike the Foundation overlay structs above, Swift `String` is a core Swift standard-library type with Foundation bridging to NSString.

It remains a central Layer-1 ABI primitive because Tier-1 Swift APIs use String directly.

However, the framework should still exploit NSString bridging whenever the compiler ABI allows it.

Do not duplicate/transcode strings unnecessarily.

# Architectural consequence: distinguish three kinds of Swift values

## Category A — pure Swift standard-library values

Examples:
- String;
- Array;
- Set;
- Dictionary;
- Optional;
- Result-like enums.

These may require genuine Swift ABI machinery.

## Category B — Foundation Swift overlay values with Objective-C/CoreFoundation bridge

Examples:
- Data;
- Date;
- URL;
- UUID;
- IndexPath;
- CharacterSet;
- Decimal;
- URLRequest.

For these, research bridging first.

## Category C — Apple-framework-specific Swift values

Examples:
- StoreKit Product;
- AdAttributionKit AppImpression;
- TranslationSession.Response;
- ActivityKit ActivityContent.

These generally require Swift metadata/value-witness handling unless public accessors expose an easier route.

This categorization should materially shrink the Layer-1 primitive implementation.

# Revised minimum primitive priority

After this audit:

## Must solve early

- Swift calling convention;
- type metadata/value witnesses;
- Swift String;
- Array for concrete required element types;
- optionals/enums;
- generic metadata;
- async/throws.

## Investigate bridge before implementing native Swift storage

- Data;
- Date;
- URL;
- UUID;
- IndexPath;
- CharacterSet;
- Decimal;
- URLRequest;
- Dictionary/Set where elements bridge.

## Do not implement without a concrete consumer

- arbitrary Foundation overlay value type;
- arbitrary Swift collection;
- arbitrary Error existential.

# Performance impact

This bridge-aware strategy can improve both complexity and runtime cost.

Example:

Bad generic approach:

```text
Rust URL
 -> allocate Swift URL representation manually
 -> pass Swift value
 -> framework internally reaches NSURL/CFURL/native URL machinery
```

Potentially better:

```text
Rust
 -> NSURL/CFURL/native representation
 -> use Foundation's established bridge only where Swift call requires it
```

Likewise for Data:
- keep bytes in Rust/native buffer;
- create an NSData/CFData view/owner only when necessary;
- rely on Apple's bridge semantics rather than adding a framework-owned second buffer.

The exact zero-copy properties must be proven per bridge and ownership mode; do not assume all bridges are free.

# Research requirement for the implementation phase

For each Foundation-bridged type used in a Tier-1 Swift call:

1. compile a minimal reference Swift call;
2. pass both native Swift value and bridged Objective-C equivalent where source allows;
3. inspect SIL/LLVM/assembly;
4. determine whether bridge is:
   - representation-compatible;
   - O(1) wrapper/storage sharing;
   - allocating;
   - copying;
   - lazily bridging elements;
5. choose the lowest-cost supported path.

Document the measured result in PERFORMANCE.md before making generic performance claims.
