#!/bin/sh
set -eu

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

require_fixed() {
    file=$1
    needle=$2
    message=$3
    rg -F "$needle" "$file" >/dev/null || fail "$message"
}

require_pattern() {
    file=$1
    pattern=$2
    message=$3
    rg "$pattern" "$file" >/dev/null || fail "$message"
}

if [ "$(uname -s)" != Darwin ]; then
    printf '%s\n' 'requires macOS with Xcode' >&2
    exit 77
fi

for tool in xcrun xcode-select xcodebuild clang awk grep head mktemp rg rm sed sw_vers tr uname wc; do
    command -v "$tool" >/dev/null 2>&1 || fail "missing tool: $tool"
done

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)
probe_parent=${TMPDIR:-/tmp}
probe_dir=$(mktemp -d "$probe_parent/swift-abi-async-runtime.XXXXXX")
cleanup_probe() {
    if [ "${KEEP_SWIFT_ASYNC_RUNTIME_PROBE:-0}" = 1 ]; then
        printf 'preserved temporary audit artifacts: %s\n' "$probe_dir" >&2
    else
        rm -rf "$probe_dir"
    fi
}
trap cleanup_probe EXIT HUP INT TERM

case "$probe_dir/" in
    "$repo_root/"*) fail 'temporary compiler artifacts must stay outside the checkout' ;;
esac

developer_dir=$(xcode-select -p)
toolchain_root="$developer_dir/Toolchains/XcodeDefault.xctoolchain"
iphoneos_sdk=$(xcrun --sdk iphoneos --show-sdk-path)
iphonesimulator_sdk=$(xcrun --sdk iphonesimulator --show-sdk-path)
macos_sdk=$(xcrun --sdk macosx --show-sdk-path)
iphoneos_interface="$iphoneos_sdk/usr/lib/swift/_Concurrency.swiftmodule/arm64e-apple-ios.swiftinterface"
simulator_interface="$iphonesimulator_sdk/usr/lib/swift/_Concurrency.swiftmodule/arm64-apple-ios-simulator.swiftinterface"
concurrency_shim="$iphoneos_sdk/usr/lib/swift/shims/_SwiftConcurrency.h"
shim_modulemap="$iphoneos_sdk/usr/lib/swift/shims/module.modulemap"
simulator_shim="$iphonesimulator_sdk/usr/lib/swift/shims/_SwiftConcurrency.h"
simulator_modulemap="$iphonesimulator_sdk/usr/lib/swift/shims/module.modulemap"
iphoneos_tbd="$iphoneos_sdk/usr/lib/swift/libswift_Concurrency.tbd"
simulator_tbd="$iphonesimulator_sdk/usr/lib/swift/libswift_Concurrency.tbd"

for file in "$iphoneos_interface" "$simulator_interface" "$concurrency_shim" "$shim_modulemap" "$simulator_shim" "$simulator_modulemap" "$iphoneos_tbd" "$simulator_tbd"; do
    [ -f "$file" ] || fail "missing expected installed interface: $file"
done

xcrun swiftc --help | grep -F -- '-cxx-interoperability-mode' >/dev/null || fail 'active swiftc does not document C++ interoperability mode'
xcrun swiftc -frontend -help | grep -F -- '-emit-clang-header-path' >/dev/null || fail 'active Swift frontend does not document C++ header generation'

for interface in "$iphoneos_interface" "$simulator_interface"; do
    require_fixed "$interface" 'public struct Task<' 'installed _Concurrency interface lacks public Task type'
    require_fixed "$interface" 'public init(name: Swift.String? = nil, priority: _Concurrency.TaskPriority? = nil, @_inheritActorContext @_implicitSelfCapture operation: sending @escaping @isolated(any) () async -> Success)' 'installed _Concurrency interface lacks the public Task initializer form'
    require_fixed "$interface" 'Builtin.createTask(' 'installed Task initializer no longer uses its compiler builtin in this toolchain interface'
    require_fixed "$interface" 'withCheckedContinuation' 'installed _Concurrency interface lacks checked continuation declarations'
    require_fixed "$interface" 'withUnsafeContinuation' 'installed _Concurrency interface lacks unsafe continuation declarations'
    require_fixed "$interface" '@_silgen_name("swift_task_getCurrent")' 'installed _Concurrency interface lacks the observed current-task binding'
    require_fixed "$interface" 'public func _taskFutureGet<T>(_ task: Builtin.NativeObject) async -> T' 'installed _Concurrency interface lacks the observed future-wait binding'
    require_fixed "$interface" '@_silgen_name("swift_task_cancel")' 'installed _Concurrency interface lacks the observed task-cancel binding'
done

if rg -n 'swift_task_(create|alloc|dealloc|switch|enqueue|resume)' \
    "$toolchain_root/usr/include" "$iphoneos_sdk/usr/include" "$iphoneos_sdk/usr/lib/swift/shims" \
    "$iphonesimulator_sdk/usr/include" "$iphonesimulator_sdk/usr/lib/swift/shims" \
    -g '*.h' -g '*.modulemap' > "$probe_dir/header-task-symbols.txt"; then
    cat "$probe_dir/header-task-symbols.txt"
    fail 'an installed C header or module map now declares a task runtime symbol; review its contract before proceeding'
fi

require_fixed "$concurrency_shim" 'typedef struct _SwiftContext {' 'installed concurrency shim no longer has the expected context stub'
require_fixed "$concurrency_shim" 'struct _SwiftContext *parentContext;' 'installed concurrency shim no longer has the expected parent-context field'
require_fixed "$shim_modulemap" 'module _SwiftConcurrencyShims {' 'installed shim module map lacks the concurrency shim module'
require_fixed "$simulator_shim" 'struct _SwiftContext *parentContext;' 'installed simulator concurrency shim no longer has the expected parent-context field'
require_fixed "$simulator_modulemap" 'module _SwiftConcurrencyShims {' 'installed simulator shim module map lacks the concurrency shim module'

for symbol in _swift_task_create _swift_task_create_common _swift_task_alloc _swift_task_dealloc _swift_task_switch _swift_task_enqueueGlobal _swift_task_cancel; do
    require_fixed "$iphoneos_tbd" "$symbol" "iPhoneOS export stub lacks expected observed symbol $symbol"
    require_fixed "$simulator_tbd" "$symbol" "iOS Simulator export stub lacks expected observed symbol $symbol"
done

host_target_info=$(xcrun swiftc -print-target-info)
host_swift_target=$(printf '%s\n' "$host_target_info" | awk -F '"' '/"triple"/ { print $4; exit }')
case "$host_swift_target" in
    *-apple-macosx*) ;;
    *) fail "unsupported Swift host target: $host_swift_target" ;;
esac

device_target=arm64-apple-ios17.0
simulator_target=arm64-apple-ios17.0-simulator
cat > "$probe_dir/Oracle.swift" <<'SWIFT'
public enum OracleFailure: Error { case failed }
public func asyncIdentity(_ value: Int32) async -> Int32 { value }
public func asyncThrowsIdentity(_ value: Int32) async throws -> Int32 { if value < 0 { throw OracleFailure.failed }; return value }
public func asyncCaller(_ value: Int32) async -> Int32 { await asyncIdentity(value) }
public func asyncThrowsCaller(_ value: Int32) async -> Int32 { do { return try await asyncThrowsIdentity(value) } catch { return -1 } }
public func launchTask(_ value: Int32) -> Task<Int32, Never> { Task { await asyncIdentity(value) } }
public func launchThrowingTask(_ value: Int32) -> Task<Int32, any Error> { Task { try await asyncThrowsIdentity(value) } }
SWIFT

xcrun swiftc -parse-as-library -module-name SwiftAsyncRuntimeEntryAudit -target "$device_target" -sdk "$iphoneos_sdk" -Onone -emit-silgen -o "$probe_dir/device.sil" "$probe_dir/Oracle.swift"
xcrun swiftc -parse-as-library -module-name SwiftAsyncRuntimeEntryAudit -target "$device_target" -sdk "$iphoneos_sdk" -Onone -emit-ir -o "$probe_dir/device.ll" "$probe_dir/Oracle.swift"
xcrun swiftc -parse-as-library -module-name SwiftAsyncRuntimeEntryAudit -target "$simulator_target" -sdk "$iphonesimulator_sdk" -Onone -emit-ir -o "$probe_dir/simulator.ll" "$probe_dir/Oracle.swift"

for ir in "$probe_dir/device.ll" "$probe_dir/simulator.ll"; do
    require_fixed "$ir" '@swift_task_create(' 'compiler-generated Task initializer call no longer lowers through observed swift_task_create in this probe'
    require_fixed "$ir" '@swift_task_alloc(' 'compiler-generated async caller no longer allocates an async context in this probe'
    require_fixed "$ir" '@swift_task_dealloc(' 'compiler-generated async caller no longer tears down an async context in this probe'
    require_fixed "$ir" '@swift_task_switch(' 'compiler-generated async caller no longer emits an async resume/switch path in this probe'
    require_fixed "$ir" 'swiftself null' 'compiler-generated throwing completion no longer includes the observed success/error distinction'
    require_pattern "$ir" 'swiftself %[0-9]+' 'compiler-generated throwing completion no longer passes an observed error object in swiftself'
done
require_fixed "$probe_dir/device.sil" 'hop_to_executor' 'compiler-generated Swift caller SIL no longer records an executor hop'
require_pattern "$probe_dir/device.ll" 'store ptr .*TQ0_' 'compiler-generated async caller no longer stores a compiler resume function in its context'

host_dir="$probe_dir/host"
mkdir "$host_dir"
xcrun swiftc -parse-as-library -module-name SwiftAsyncRuntimeEntryAudit -cxx-interoperability-mode=default -emit-clang-header-path "$host_dir/SwiftAsyncRuntimeEntryAudit.h" -target "$host_swift_target" -emit-library -o "$host_dir/libSwiftAsyncRuntimeEntryAudit.dylib" "$probe_dir/Oracle.swift" > "$host_dir/compiler.stdout" 2> "$host_dir/compiler.stderr"
require_fixed "$host_dir/SwiftAsyncRuntimeEntryAudit.h" "Unavailable in C++: Swift global function 'launchTask(_:)'" 'generated C++ header no longer records Task-returning wrapper as unavailable'
require_fixed "$host_dir/SwiftAsyncRuntimeEntryAudit.h" "Unavailable in C++: Swift global function 'launchThrowingTask(_:)'" 'generated C++ header no longer records throwing Task-returning wrapper as unavailable'
require_fixed "$host_dir/SwiftAsyncRuntimeEntryAudit.h" "Unavailable in C++: Swift global function 'asyncIdentity(_:)'" 'generated C++ header no longer records async function as unavailable'
require_fixed "$host_dir/SwiftAsyncRuntimeEntryAudit.h" "Unavailable in C++: Swift global function 'asyncThrowsIdentity(_:)'" 'generated C++ header no longer records throwing async function as unavailable'
if rg 'SWIFT_(INLINE_THUNK|EXTERN).*launch(Task|ThrowingTask)|SWIFT_(INLINE_THUNK|EXTERN).*async(Identity|ThrowsIdentity)' "$host_dir/SwiftAsyncRuntimeEntryAudit.h" >/dev/null; then
    fail 'generated C++ header unexpectedly exposes a task-entry or async function; extend the bounded call-path audit'
fi

printf 'macOS: %s (%s)\n' "$(sw_vers -productVersion)" "$(sw_vers -buildVersion)"
printf 'Xcode: %s\n' "$(xcodebuild -version | tr '\n' ' ')"
printf 'Swift: %s\n' "$(xcrun swiftc --version | head -n 1)"
printf 'Clang: %s\n' "$(xcrun clang --version | head -n 1)"
printf 'Selected developer directory: %s\n' "$developer_dir"
printf 'macOS SDK: %s (%s); host target: %s\n' "$macos_sdk" "$(xcrun --sdk macosx --show-sdk-version)" "$host_swift_target"
printf 'iPhoneOS SDK: %s (%s); compile target: %s\n' "$iphoneos_sdk" "$(xcrun --sdk iphoneos --show-sdk-version)" "$device_target"
printf 'iPhoneSimulator SDK: %s (%s); compile target: %s\n' "$iphonesimulator_sdk" "$(xcrun --sdk iphonesimulator --show-sdk-version)" "$simulator_target"
printf 'Installed Swift task API interface: %s\n' "$iphoneos_interface"
rg -n '^@frozen public struct Task<|public init\(name: Swift.String\? = nil, priority: _Concurrency.TaskPriority\? = nil|Builtin\.createTask\(|withCheckedContinuation|withUnsafeContinuation' "$iphoneos_interface" | head -n 28
printf 'Underscored Swift-interface bindings to task runtime symbols:\n'
rg -n -A 1 '@_silgen_name\("swift_task_(getCurrent|future_wait|future_wait_throwing|cancel)"\)' "$iphoneos_interface"
printf 'Compiler shim and module map: %s %s\n' "$concurrency_shim" "$shim_modulemap"
rg -n 'module _SwiftConcurrencyShims|_SwiftContext|parentContext' "$shim_modulemap" "$concurrency_shim"
printf 'Public C headers/module maps: no swift_task_create/alloc/dealloc/switch/enqueue/resume declaration found under toolchain include or iPhoneOS/iPhoneSimulator SDK include/shims\n'
printf 'iPhoneOS and simulator export stubs: %s %s\n' "$iphoneos_tbd" "$simulator_tbd"
rg -n 'targets:|swift-abi-version:|_swift_task_(alloc|cancel|create|create_common|dealloc|enqueueGlobal|switch)' "$iphoneos_tbd" | head -n 20
rg -n 'targets:|swift-abi-version:' "$simulator_tbd" | head -n 5
printf 'Compiler-generated device IR task/context/resume observations:\n'
rg -n 'define swifttailcc void .*asyncCaller|define swifttailcc void .*asyncThrowsCaller|@swift_task_(create|alloc|dealloc|switch)|store ptr .*TQ0_|swiftself null|swiftself %' "$probe_dir/device.ll" | head -n 55
printf 'Compiler-generated device SIL executor observations:\n'
rg -n 'launchTask|Task<>\.init|hop_to_executor' "$probe_dir/device.sil" | head -n 28
printf 'Generated C++ header task-entry result:\n'
rg -n "Unavailable in C\+\+: Swift global function '(launchTask|launchThrowingTask|asyncIdentity|asyncThrowsIdentity)" "$host_dir/SwiftAsyncRuntimeEntryAudit.h"
printf 'Generated Swift compiler diagnostics: stdout=%s bytes stderr=%s bytes\n' "$(wc -c < "$host_dir/compiler.stdout" | tr -d ' ')" "$(wc -c < "$host_dir/compiler.stderr" | tr -d ' ')"
printf '%s\n' 'Audit result: public Swift Task APIs exist, but no supported public C/C++ task-entry contract was exposed by the inspected interfaces; runtime task symbols were observed only, not called'
