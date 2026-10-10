# Native build specifications

Each native pilot keeps a repo-local `build-spec.json` beside its `Cargo.toml`. The file declares schema 1 inputs for `ios-rust-build`; see `specs/schema-v1.json`. The installed builder accepts an explicit workspace root, spec path, target triple, target OS, Cargo output directory and package manifest directory. It rejects specs and source/header paths that escape the workspace root, unknown fields, unsupported targets and unmet feature guards.

Cargo invokes a small Rust `build.rs` bridge. The bridge calls `ios-rust-build` through argv, verifies tool, protocol and host versions, validates the structured result, then emits only allow-listed Cargo link and rebuild directives. Missing tools or incompatible results fail the build. Non-iOS targets are explicit no-ops and do not inherit Apple link flags.
