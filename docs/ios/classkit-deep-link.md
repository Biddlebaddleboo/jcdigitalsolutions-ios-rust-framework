# ClassKit deep-link marker

`ios-system-services::is_classkit_deep_link(&NSUserActivity) -> bool` reads only the
caller-owned activity's `isClassKitDeepLink` marker on iOS 11.3 or later. It does not access
assignment content, `CLSDataStore`, context identifiers, or user identity.

The getter has no documented permission, usage-description, or entitlement requirement. A host
that shares assignment data with Schoolwork has separate ClassKit adoption and
`com.apple.developer.ClassKit-environment` requirements; this marker query does not make an app
ClassKit-enabled or prove assignment access.

The activity is borrowed and not retained. The API does not document an independent thread-safety
guarantee; use the activity within the host application's activity lifecycle and threading rules.
No live activity or Schoolwork flow is verified by compile/link checks.

See the [D81 implementation record](../../PLAN_CAPABILITIES_CLASSKIT.md) and
[G73 validation record](../../PLAN_VALIDATION_IOS_CLASSKIT.md).
