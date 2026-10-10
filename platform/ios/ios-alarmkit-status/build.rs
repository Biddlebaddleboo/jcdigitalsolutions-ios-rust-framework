use ios_native_build_support::{Architecture, DeploymentTarget, IosNativeBuildConfig, IosTarget};

const CONFIG: IosNativeBuildConfig = IosNativeBuildConfig {
    capability: "AlarmKit",
    library: "alarmkit_status",
    sources: &["native/alarmkit_status.c"],
    headers: &["../../../interop/swift-abi-core/include/swift_abi_runtime.h"],
    frameworks: &["AlarmKit"],
    minimum_os: DeploymentTarget {
        major: 26,
        minor: 0,
    },
    supported_architectures: &[Architecture::Arm64],
    targets: &[IosTarget::DeviceArm64, IosTarget::SimulatorArm64],
    compiler_options: &["-std=c11", "-Wall", "-Wextra", "-Werror"],
    compile_description: "AlarmKit C swiftcall and enum-witness bridge compile",
    archive_description: "AlarmKit static archive creation",
};

fn main() {
    ios_native_build_support::build(&CONFIG);
}
