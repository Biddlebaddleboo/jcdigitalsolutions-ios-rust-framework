use ios_native_build_support::{Architecture, DeploymentTarget, IosNativeBuildConfig, IosTarget};

const CONFIG: IosNativeBuildConfig = IosNativeBuildConfig {
    capability: "photogrammetry",
    library: "photogrammetry_status",
    sources: &["native/photogrammetry_status.c"],
    headers: &["../../../interop/swift-abi-core/include/swift_abi_runtime.h"],
    frameworks: &["RealityFoundation"],
    minimum_os: DeploymentTarget {
        major: 17,
        minor: 0,
    },
    supported_architectures: &[Architecture::Arm64, Architecture::X86_64],
    targets: &[
        IosTarget::DeviceArm64,
        IosTarget::SimulatorArm64,
        IosTarget::SimulatorX86_64,
    ],
    compiler_options: &["-std=c11", "-Wall", "-Wextra", "-Werror"],
    compile_description: "RealityFoundation C swiftcall thunk compile",
    archive_description: "RealityFoundation static archive creation",
};

fn main() {
    ios_native_build_support::build(&CONFIG);
}
