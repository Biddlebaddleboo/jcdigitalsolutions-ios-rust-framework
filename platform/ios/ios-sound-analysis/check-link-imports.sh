#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s\n' "$tool is required for the SoundAnalysis link/import check" >&2
        exit 1
    fi
done

cat > target/ios-sound-analysis-link-imports-expected.txt <<'IMPORTS'
Foundation
SoundAnalysis
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-sound-analysis --example ios_sound_analysis_link_probe --target "$target"
    binary="target/$target/release/examples/ios_sound_analysis_link_probe"
    imports="target/ios-sound-analysis-link-imports-$target.txt"
    libraries="target/ios-sound-analysis-link-libraries-$target.txt"
    symbols="target/ios-sound-analysis-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-sound-analysis-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|AVAudio|SNAudio(Stream|File)Analyzer|SHSession|startAnalysis' "$symbols"; then
        printf '%s\n' "unexpected Swift runtime, audio-input, analyzer, or ShazamKit symbol import in $symbols" >&2
        exit 1
    fi
done
