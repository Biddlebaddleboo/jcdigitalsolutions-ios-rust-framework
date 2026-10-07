#!/bin/sh
set -eu

example_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$example_dir/../.." && pwd)
project="$example_dir/ios-minimal.xcodeproj"
scheme_file="$project/xcshareddata/xcschemes/ios-minimal.xcscheme"
archive_root="$repo_dir/target/ios-minimal/archive"
archive_path="$archive_root/ios-minimal.xcarchive"
derived_data="$repo_dir/target/ios-minimal/archive-derived-data"
archive_log="$archive_root/xcodebuild-archive.log"
list_log="$archive_root/xcodebuild-list.log"

if [ "$#" -ne 0 ]; then
    echo "usage: $0" >&2
    exit 2
fi

for tool in cargo xcodebuild xcrun plutil otool codesign; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "archive smoke unavailable: required command '$tool' was not found" >&2
        exit 1
    fi
done
if [ ! -x /usr/libexec/PlistBuddy ]; then
    echo "archive smoke unavailable: /usr/libexec/PlistBuddy was not found" >&2
    exit 1
fi

if [ ! -f "$project/project.pbxproj" ] || [ ! -f "$scheme_file" ]; then
    echo "archive smoke unavailable: Xcode project or shared ios-minimal scheme is missing" >&2
    exit 1
fi

xcode_version=$(xcodebuild -version) || {
    echo "archive smoke unavailable: xcodebuild -version failed" >&2
    exit 1
}
sdk_path=$(xcrun --sdk iphoneos --show-sdk-path) || {
    echo "archive smoke unavailable: xcrun could not find the iPhoneOS SDK" >&2
    exit 1
}
sdk_version=$(xcrun --sdk iphoneos --show-sdk-version) || {
    echo "archive smoke unavailable: xcrun could not report the iPhoneOS SDK version" >&2
    exit 1
}
if [ ! -d "$sdk_path" ]; then
    echo "archive smoke unavailable: iPhoneOS SDK path does not exist: $sdk_path" >&2
    exit 1
fi

mkdir -p "$archive_root"
if ! xcodebuild -list -project "$project" >"$list_log" 2>&1; then
    echo "archive smoke unavailable: xcodebuild could not load the ios-minimal project/scheme; see $list_log" >&2
    cat "$list_log" >&2
    exit 1
fi
if ! awk '/Schemes:/{in_schemes=1; next} in_schemes && $1 == "ios-minimal"{found=1} END{exit !found}' "$list_log"; then
    echo "archive smoke unavailable: ios-minimal scheme is not listed by xcodebuild; see $list_log" >&2
    cat "$list_log" >&2
    exit 1
fi

printf '%s\n' "$xcode_version"
printf 'iPhoneOS SDK %s at %s\n' "$sdk_version" "$sdk_path"
echo "Building the Release device app with examples/ios-minimal/build.sh device"
sh "$example_dir/build.sh" device

rm -rf "$archive_path" "$derived_data"
echo "Running xcodebuild archive (CODE_SIGNING_ALLOWED=NO, CODE_SIGNING_REQUIRED=NO)"
if xcodebuild \
    -project "$project" \
    -scheme ios-minimal \
    -configuration Release \
    -destination 'generic/platform=iOS' \
    -derivedDataPath "$derived_data" \
    -archivePath "$archive_path" \
    CODE_SIGNING_ALLOWED=NO \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGN_IDENTITY= \
    archive >"$archive_log" 2>&1; then
    tail -n 30 "$archive_log"
else
    status=$?
    echo "xcodebuild archive failed with status $status; full log: $archive_log" >&2
    tail -n 100 "$archive_log" >&2
    exit "$status"
fi

archive_app="$archive_path/Products/Applications/ios-minimal.app"
app_executable="$archive_app/ios-minimal"
app_plist="$archive_app/Info.plist"
for expected in "$archive_path/Info.plist" "$app_executable" "$app_plist"; do
    if [ ! -f "$expected" ]; then
        echo "archive verification failed: expected file is missing: $expected" >&2
        exit 1
    fi
done

plutil -lint "$archive_path/Info.plist" "$app_plist"
bundle_executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$app_plist")
if [ "$bundle_executable" != "ios-minimal" ]; then
    echo "archive verification failed: CFBundleExecutable is '$bundle_executable', expected 'ios-minimal'" >&2
    exit 1
fi

imports="$archive_root/archive-imports.txt"
if ! otool -L "$app_executable" >"$imports"; then
    echo "archive verification failed: otool could not inspect $app_executable" >&2
    exit 1
fi
cat "$imports"
if grep -Eiq 'libswift|libpython|Python\.framework' "$imports"; then
    echo "archive verification failed: Swift or Python runtime import detected; see $imports" >&2
    exit 1
fi

swift_source=$(find "$archive_path" -type f -name '*.swift' -print -quit)
if [ -n "$swift_source" ]; then
    echo "archive verification failed: Swift source found in archive: $swift_source" >&2
    exit 1
fi

codesign_report="$archive_root/codesign-display.txt"
if codesign -dv --verbose=2 "$archive_app" >"$codesign_report" 2>&1; then
    echo "archive verification failed: CODE_SIGNING_ALLOWED=NO produced a code signature; see $codesign_report" >&2
    cat "$codesign_report" >&2
    exit 1
elif ! grep -Fq "code object is not signed at all" "$codesign_report"; then
    echo "archive verification failed: could not establish unsigned status; see $codesign_report" >&2
    cat "$codesign_report" >&2
    exit 1
fi
if [ -d "$archive_app/_CodeSignature" ] || [ -f "$archive_app/embedded.mobileprovision" ]; then
    echo "archive verification failed: CODE_SIGNING_ALLOWED=NO produced signature or provisioning metadata" >&2
    exit 1
fi

echo "Archive smoke passed: $archive_path"
echo "Signing state: unsigned (codesign reports no signature; CODE_SIGNING_ALLOWED=NO; no _CodeSignature or embedded.mobileprovision)"
echo "Archive app: $archive_app"
echo "Import report: $imports"
