# Pinned PATH tool packages

`manifest-v1.tsv` is the sole offline release pin for `tools/install-tools.sh`. Each supported host has one checked-in archive and a full archive SHA-256. The archive metadata records the immutable source commit, release ID, host triple and Rust compiler version; each executable reports the same source and host metadata through `--version --format json`.

To provision a supported host, run `tools/install-tools.sh --prefix PATH` from any directory and add `PATH/bin` to `PATH`. The installer makes no network requests, rejects unknown archive members, verifies the archive and executable checksums, checks the embedded metadata and version interface, then installs the two binaries. Missing host pins or artifacts fail closed.

The canonical build recipe is `.github/workflows/shared-tools.yml`, using Rust 1.94.1 and native GitHub-hosted runners for Linux x86_64 and macOS arm64. It checks out the exact source SHA, verifies the runner triple, runs tool tests and strict Clippy, builds release binaries, records compiler/provenance metadata and uploads a checksum-bearing artifact. To add or refresh a pin, download the artifact for the exact recorded source commit, place its archive at the path in the manifest, calculate its SHA-256, then update the manifest in the same reviewed commit. Never substitute a moving URL or an unverified local binary.

Tool source remains in the framework Git history until R1/R2 parity, CI, supported-host, source-free smoke and Apple artifact gates pass. Rebuild from the source commit and toolchain in the manifest/workflow; the source commit remains the immutable maintenance reference after engine source is removed from an ordinary checkout.
