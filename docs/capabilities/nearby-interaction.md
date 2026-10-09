# Nearby Interaction capability snapshot

`framework-nearby` provides a `no_std` value and a static backend trait for one scalar capability: whether a backend reports support for precise distance measurement. The portable crate has no third-party dependency and exposes no Apple type.

The snapshot is deliberately narrower than a Nearby Interaction service. `true` does not establish user permission, compatible peers, valid session configuration, session readiness, successful operation, measurement quality, or background execution. `false` reports only this precise-distance field; it does not classify every other Nearby Interaction feature. The value is a query-time snapshot, not cached authorization or runtime state.

The backend contract permits a non-prompting scalar query only. It excludes creating or running an `NISession`, requesting permission, exchanging discovery tokens, peer discovery, and ranging.

The iOS implementation is described in the [iOS Nearby Interaction guide](../ios/nearby-interaction.md). The current manifest row is `041-sensors-connectivity-nearby-interaction`; the integrator updates it after review of this slice.
