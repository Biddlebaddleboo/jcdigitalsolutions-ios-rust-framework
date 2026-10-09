use ios_native_build_support::{Architecture, DeploymentTarget, IosNativeBuildConfig, IosTarget};

const CONFIG: IosNativeBuildConfig = IosNativeBuildConfig {
    capability: "ActivityKit",
    library: "activitykit_status",
    sources: &["native/activitykit_status.c"],
    frameworks: &["ActivityKit"],
    minimum_os: DeploymentTarget {
        major: 16,
        minor: 1,
    },
    supported_architectures: &[Architecture::Arm64],
    targets: &[IosTarget::DeviceArm64, IosTarget::SimulatorArm64],
    compiler_options: &["-std=c11", "-Wall", "-Wextra", "-Werror"],
    compile_description: "ActivityKit C swiftcall bridge compile",
    archive_description: "ActivityKit static archive creation",
};

fn main() {
    ios_native_build_support::build(&CONFIG);
}
