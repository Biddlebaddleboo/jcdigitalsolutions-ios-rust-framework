# Minimal C consumer

This program checks the packed ABI version and links the owned-buffer destructor through its null no-op case. This slice has no owned-buffer creator, so it does not fabricate or destroy an owned object.

From the repository root, run `sh bindings/c/check.sh`. The script builds the static library, compiles the header as C11 and C++, verifies C layouts, audits the two declared C symbols against `abi-manifest.json`, then links and runs this program.
