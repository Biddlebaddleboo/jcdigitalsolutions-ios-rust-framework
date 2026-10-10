#!/bin/sh
set -eu

fail() {
  printf 'ios-rust-tools installer: %s\n' "$1" >&2
  exit 1
}

if [ "$#" -ne 2 ] || [ "$1" != "--prefix" ] || [ -z "$2" ] || [ "${2#-}" != "$2" ]; then
  fail 'usage: tools/install-tools.sh --prefix PATH'
fi
prefix=$2
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
workspace_root=$(CDPATH= cd -- "$script_dir/.." && pwd -P)
manifest="$workspace_root/tools/releases/manifest-v1.tsv"
[ -f "$manifest" ] || fail "pinned release manifest is missing: $manifest"

case "$(uname -s):$(uname -m)" in
  Darwin:arm64|Darwin:aarch64) host=aarch64-apple-darwin ;;
  Darwin:x86_64) host=x86_64-apple-darwin ;;
  Linux:x86_64) host=x86_64-unknown-linux-gnu ;;
  *) fail "unsupported host $(uname -s):$(uname -m)" ;;
esac

match_count=$(awk -F '\t' -v host="$host" 'NR > 1 && $1 == host { count++ } END { print count + 0 }' "$manifest")
[ "$match_count" -eq 1 ] || fail "expected one pinned artifact for $host; found $match_count"
pin_line=$(awk -F '\t' -v host="$host" 'NR > 1 && $1 == host { print }' "$manifest")
IFS="$(printf '\t')" read -r pin_host release_id source_sha rustc_version tool_version archive_path archive_sha256 <<EOF
$pin_line
EOF
[ "$pin_host" = "$host" ] || fail 'manifest host does not match this machine'
[ "$release_id" = "ios-rust-tools-$tool_version+$source_sha" ] || fail 'manifest release ID is inconsistent'
case "$source_sha" in *[!0-9a-f]*|'') fail 'invalid source SHA in release manifest' ;; esac
[ "${#source_sha}" -eq 40 ] || [ "${#source_sha}" -eq 64 ] || fail 'source SHA must be a full Git object ID'
case "$archive_sha256" in *[!0-9a-f]*|'') fail 'invalid archive SHA-256 in release manifest' ;; esac
[ "${#archive_sha256}" -eq 64 ] || fail 'archive SHA-256 must have 64 lowercase hex digits'
[ -n "$rustc_version" ] || fail 'manifest compiler version is empty'
[ "$tool_version" = "0.1.0" ] || fail "unsupported pinned tool version $tool_version"
[ "$archive_path" = "tools/releases/ios-rust-tools-$host.tar.gz" ] || fail 'manifest archive path is not canonical'
archive="$workspace_root/$archive_path"
[ -f "$archive" ] || fail "pinned offline archive is missing: $archive"

if command -v shasum >/dev/null 2>&1; then
  (cd "$workspace_root" && printf '%s  %s\n' "$archive_sha256" "$archive_path" | shasum -a 256 -c -) >/dev/null || fail 'archive SHA-256 verification failed'
elif command -v sha256sum >/dev/null 2>&1; then
  (cd "$workspace_root" && printf '%s  %s\n' "$archive_sha256" "$archive_path" | sha256sum -c -) >/dev/null || fail 'archive SHA-256 verification failed'
else
  fail 'shasum or sha256sum is required to verify the pinned archive'
fi

scratch=$(mktemp -d "${TMPDIR:-/tmp}/ios-rust-install.XXXXXX") || fail 'cannot create temporary directory'
trap 'rm -rf "$scratch"' 0
trap 'exit 1' HUP INT TERM
archive_list="$scratch/archive-list.txt"
tar -tzf "$archive" > "$archive_list" || fail 'cannot inspect pinned archive'
entry_count=0
build_seen=0
validate_seen=0
metadata_seen=0
checksums_seen=0
while IFS= read -r entry; do
  entry_count=$((entry_count + 1))
  case "$entry" in
    bin/) : ;;
    bin/ios-rust-build) build_seen=$((build_seen + 1)) ;;
    bin/ios-rust-validate) validate_seen=$((validate_seen + 1)) ;;
    tool-metadata.json) metadata_seen=$((metadata_seen + 1)) ;;
    SHA256SUMS) checksums_seen=$((checksums_seen + 1)) ;;
    *) fail "unexpected archive member: $entry" ;;
  esac
done < "$archive_list"
[ "$entry_count" -eq 5 ] && [ "$build_seen" -eq 1 ] && [ "$validate_seen" -eq 1 ] && [ "$metadata_seen" -eq 1 ] && [ "$checksums_seen" -eq 1 ] || fail 'archive member set is incomplete or duplicated'
mkdir "$scratch/unpacked"
tar -xzf "$archive" -C "$scratch/unpacked" || fail 'cannot extract pinned archive'
[ ! -L "$scratch/unpacked/bin/ios-rust-build" ] && [ -f "$scratch/unpacked/bin/ios-rust-build" ] || fail 'builder is not a regular file'
[ ! -L "$scratch/unpacked/bin/ios-rust-validate" ] && [ -f "$scratch/unpacked/bin/ios-rust-validate" ] || fail 'validator is not a regular file'
if command -v shasum >/dev/null 2>&1; then
  (cd "$scratch/unpacked/bin" && shasum -a 256 -c ../SHA256SUMS) >/dev/null || fail 'internal binary checksum verification failed'
else
  (cd "$scratch/unpacked/bin" && sha256sum -c ../SHA256SUMS) >/dev/null || fail 'internal binary checksum verification failed'
fi

metadata="$scratch/unpacked/tool-metadata.json"
origin="https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/tree/$source_sha"
for field in "\"schema_version\":1" "\"release_id\":\"$release_id\"" "\"source_sha\":\"$source_sha\"" "\"origin\":\"$origin\"" "\"host_triple\":\"$host\"" "\"rustc_version\":\"$rustc_version\""; do
  grep -F "$field" "$metadata" >/dev/null || fail "tool metadata does not match pin: $field"
done
for tool in ios-rust-build ios-rust-validate; do
  version_json=$("$scratch/unpacked/bin/$tool" --version --format json 2>&1) || fail "$tool rejected its version query"
  for field in "\"version\":\"$tool_version\"" "\"release_id\":\"$release_id\"" "\"source_sha\":\"$source_sha\"" "\"rustc_version\":\"$rustc_version\"" "\"host\":\"$host\""; do
    printf '%s\n' "$version_json" | grep -F "$field" >/dev/null || fail "$tool version metadata does not match pin: $field"
  done
done

mkdir -p "$prefix/bin" || fail "cannot create install directory: $prefix/bin"
for tool in ios-rust-build ios-rust-validate; do
  staged="$prefix/bin/.$tool.$$"
  cp "$scratch/unpacked/bin/$tool" "$staged" || fail "cannot stage $tool"
  chmod 755 "$staged" || fail "cannot set executable mode for $tool"
  mv -f "$staged" "$prefix/bin/$tool" || fail "cannot install $tool"
done
printf 'installed ios-rust-build and ios-rust-validate %s for %s at %s/bin\n' "$tool_version" "$host" "$prefix"
