use std::env;
use std::io::Result;

/// Entry point for build script.
/// Detects cross-compilation and either skips or compiles protobuf files accordingly.
fn main() -> Result<()> {
    // More reliable cross-compilation detection
    let is_cross_compiling = is_using_cross();

    println!("cargo:warning=Cross-compilation detected: {}", is_cross_compiling);

    if is_cross_compiling {
        println!("cargo:warning=Skipping protoc compilation for cross-build");

        // Instruct cargo not to recompile proto files
        println!("cargo:rerun-if-changed=never");

        return Ok(());
    }

    // Local build: compile protobuf files
    compile_protos()
}

/// Detects if the build is running in a cross-compilation environment.
/// Returns true if cross-compiling, false otherwise.
fn is_using_cross() -> bool {
    // Check for cross-compilation specific environment variables
    if env::var("CROSS_COMPILATION").is_ok() {
        return true;
    }

    if env::var("CROSS_DOCKER_IN_DOCKER").is_ok() {
        return true;
    }

    // Check if cargo path contains "cross"
    if let Ok(cargo) = env::var("CARGO") {
        if cargo.contains("cross") {
            return true;
        }
    }

    // Check if running inside a Docker container
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }

    // Check for other cross-compilation markers
    env::var("CROSS").is_ok()
        || env::var("CROSS_RUNNER").is_ok()
        || (env::var("CARGO_TARGET_DIR").map_or(false, |dir| dir.contains("target"))
            && env::var("USER").map_or(false, |user| user == "root"))
}

/// Compiles protobuf files using tonic_build for local builds.
fn compile_protos() -> Result<()> {
    println!("cargo:warning=Compiling protobuf files locally");

    let proto_files = [
        "fechatter/v1/core.proto",
        "fechatter/v1/analytics.proto",
        "fechatter/v1/auth.proto",
        "fechatter/v1/bot.proto",
        "fechatter/v1/chat.proto",
        "fechatter/v1/files.proto",
        "fechatter/v1/notifications.proto",
    ];

    tonic_build::configure()
        .compile_protos(&proto_files, &["."])?;

    for proto in &proto_files {
        println!("cargo:rerun-if-changed={}", proto);
    }

    Ok(())
}
