# Shared validation specifications

`specs/validation-v1.json` is the repo-local contract consumed by the PATH-installed `ios-rust-validate` executable. Its `schema_version` changes independently from the tool release version; schema 1 is described by `specs/schema-v1.json`.

Use `ios-rust-validate --workspace-root PATH --spec PATH --list` to enumerate pilots, `--explain ID` to inspect inputs, gates, targets, SDKs, OS floors, source assertions, adapter requirements and evidence limits, and `--capability ID`, `--all`, or `--changed EXPLICIT_BASE` to execute validation. `--format json` returns schema-versioned JSON on stdout. A command-level diagnostic is emitted as structured JSON on stderr and exits nonzero; a validation run with failed gates keeps its results as one valid JSON document on stdout and emits the summary diagnostic on stderr.

Python 3 adapters are optional unless their spec says `required: true`. The validator executes the declared interpreter and script directly, clears inherited environment variables except `PATH` and its temporary-directory settings, passes bounded versioned JSON on stdin, caps stdout and stderr, enforces a deadline and temporary-space budget, and terminates the child on timeout or cancellation. Adapter source is trusted code, not sandboxed. PASS, FAIL, SKIPPED and ERROR-BLOCKED are distinct; an unavailable or invalid required adapter is never reported as passed.

`fixtures/` holds golden command snapshots and test-only declarative specs. `adapters/` holds optional test-specific Python 3 checks. Neither location adds a runtime dependency to shipping framework libraries.

`fixtures/example-validation-v1.json` is a test-only pilot definition showing that a new validation configuration can declare its own targets and adapters without changing engine code or adding capability implementation. Its passing and deliberately invalid adapter scripts exercise the protocol; do not use this fixture as a production pilot.
