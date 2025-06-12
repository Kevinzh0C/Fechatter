#!/bin/bash

# Production build script for Fechatter
# Builds all services locally with optimizations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BUILD_MODE="${BUILD_MODE:-release}"
TARGET_DIR="target/build"
LOG_FILE="build.log"

# Cross-compilation configuration
CROSS_COMPILE="${CROSS_COMPILE:-auto}"
TARGET_ARCH="${TARGET_ARCH:-auto}"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check system requirements
check_requirements() {
    print_status "Checking build requirements..."
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust first."
        exit 1
    fi
    
    # Check required system packages
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if ! command -v brew &> /dev/null; then
            print_warning "Homebrew not found. Some dependencies might be missing."
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if ! dpkg -l | grep -q libssl-dev; then
            print_warning "libssl-dev might be missing. Install with: apt-get install libssl-dev"
        fi
    fi
    
    print_success "Requirements check completed"
}

# Function to setup build environment
setup_environment() {
    print_status "Setting up build environment..."
    
    # Create target directory
    mkdir -p "$TARGET_DIR"
    
    # Setup cross-compilation if requested
    if [[ "$CROSS_COMPILE" != "false" ]]; then
        setup_cross_compilation
    fi
    
    # Set environment variables for optimization
    if [[ -n "${RUST_TARGET:-}" ]]; then
        # Cross-compilation optimizations
        export RUSTFLAGS="-C opt-level=3 -C strip=symbols -C target-feature=+crt-static"
        print_status "Using cross-compilation optimizations for $RUST_TARGET"
    else
        # Native compilation optimizations
        export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C strip=symbols"
        print_status "Using native compilation optimizations"
    fi
    
    export CARGO_PROFILE_RELEASE_LTO=true
    export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
    export CARGO_PROFILE_RELEASE_PANIC="abort"
    
    # For static OpenSSL linking in musl builds
    if [[ "${RUST_TARGET:-}" == *"musl" ]]; then
        export OPENSSL_STATIC=1
        export PKG_CONFIG_ALLOW_CROSS=1
        print_status "Configured for static musl linking"
    fi
    
    print_success "Environment configured"
}

# Function to clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."
    cargo clean
    rm -rf "$TARGET_DIR"
    mkdir -p "$TARGET_DIR"
    print_success "Clean completed"
}

# Function to build specific binary
build_binary() {
    local binary_name="$1"
    local package_name="$2"
    
    print_status "Building $binary_name..."
    
    # Build command with optional target
    local build_cmd="cargo build --release --bin $binary_name"
    if [[ -n "${RUST_TARGET:-}" ]]; then
        build_cmd="$build_cmd --target $RUST_TARGET"
        print_status "Cross-compiling for target: $RUST_TARGET"
    fi
    
    if $build_cmd 2>&1 | tee -a "$LOG_FILE"; then
        # Determine the correct binary path
        local binary_path
        if [[ -n "${RUST_TARGET:-}" ]]; then
            binary_path="target/main/$RUST_TARGET/release/$binary_name"
        else
            binary_path="target/main/release/$binary_name"
        fi
        
        if [[ -f "$binary_path" ]]; then
            cp "$binary_path" "$TARGET_DIR/"
            
            # Get binary size and check if it's statically linked
            local size=$(ls -lh "$TARGET_DIR/$binary_name" | awk '{print $5}')
            local link_info=""
            
            # Check linking type (Linux only)
            if command -v file &> /dev/null; then
                local file_info=$(file "$TARGET_DIR/$binary_name")
                if [[ "$file_info" == *"statically linked"* ]]; then
                    link_info=" (statically linked)"
                elif [[ "$file_info" == *"dynamically linked"* ]]; then
                    link_info=" (dynamically linked)"
                fi
            fi
            
            print_success "$binary_name built successfully (size: $size$link_info)"
        else
            print_error "Binary not found at $binary_path"
            return 1
        fi
    else
        print_error "Failed to build $binary_name"
        return 1
    fi
}

# Function to build all services
build_all_services() {
    print_status "Building all Fechatter services..."
    
    # List of services to build
    local services=(
        "fechatter_server"
        "fechatter_gateway"
        "analytics_server"
        "bot"
        "notify_server"
    )
    
    # Build each service
    for service in "${services[@]}"; do
        if ! build_binary "$service" "$service"; then
            print_error "Build failed for $service"
            exit 1
        fi
    done
    
    print_success "All services built successfully"
}

# Function to verify builds
verify_builds() {
    print_status "Verifying built binaries..."
    
    local services=(
        "fechatter_server"
        "fechatter_gateway"
        "analytics_server"
        "bot"
        "notify_server"
    )
    
    for service in "${services[@]}"; do
        if [[ -f "$TARGET_DIR/$service" ]]; then
            # Check if binary is executable
            if [[ -x "$TARGET_DIR/$service" ]]; then
                print_success "$service: OK"
            else
                print_error "$service: Not executable"
                exit 1
            fi
        else
            print_error "$service: Binary not found"
            exit 1
        fi
    done
    
    print_success "All binaries verified"
}

# Function to show build summary
show_summary() {
    print_status "Build Summary:"
    echo "=================="
    
    local total_size=0
    for binary in "$TARGET_DIR"/*; do
        if [[ -f "$binary" ]]; then
            local size=$(stat -f%z "$binary" 2>/dev/null || stat -c%s "$binary")
            local size_mb=$((size / 1024 / 1024))
            total_size=$((total_size + size))
            
            printf "%-20s %5d MB\n" "$(basename "$binary")" "$size_mb"
        fi
    done
    
    echo "=================="
    printf "%-20s %5d MB\n" "Total Size:" "$((total_size / 1024 / 1024))"
    echo ""
    
    print_success "Build completed successfully!"
    print_status "Binaries available in: $TARGET_DIR"
}

# Function to detect target architecture
detect_target_arch() {
    local host_arch
    host_arch=$(uname -m)
    
    case "$host_arch" in
        "x86_64"|"amd64")
            echo "x86_64"
            ;;
        "aarch64"|"arm64")
            echo "aarch64"
            ;;
        *)
            print_warning "Unknown architecture: $host_arch, defaulting to x86_64"
            echo "x86_64"
            ;;
    esac
}

# Function to setup cross-compilation target
setup_cross_compilation() {
    print_status "Setting up cross-compilation environment..."
    
    # Determine target architecture
    if [[ "$TARGET_ARCH" == "auto" ]]; then
        if [[ "$CROSS_COMPILE" == "auto" ]]; then
            # Auto-detect for container deployment
            TARGET_ARCH="x86_64"  # Default to x86_64 for Linux containers
            print_status "Auto-detected target: Linux x86_64 (for container deployment)"
        else
            TARGET_ARCH=$(detect_target_arch)
            print_status "Auto-detected target: $TARGET_ARCH"
        fi
    fi
    
    # Get the Rust target triple
    local rust_target=""
    case "$TARGET_ARCH" in
        "x86_64")
            rust_target="x86_64-unknown-linux-musl"
            ;;
        "aarch64"|"arm64")
            rust_target="aarch64-unknown-linux-musl"
            ;;
        *)
            print_error "Unsupported target architecture: $TARGET_ARCH"
            print_status "Supported targets: x86_64, aarch64, arm64"
            exit 1
            ;;
    esac
    
    export RUST_TARGET="$rust_target"
    print_status "Using Rust target: $RUST_TARGET"
    
    # Install target if not already installed
    if ! rustup target list --installed | grep -q "$rust_target"; then
        print_status "Installing Rust target: $rust_target"
        rustup target add "$rust_target"
    fi
    
    # Setup musl tools for static linking
    setup_musl_tools
}

# Function to setup musl cross-compilation tools
setup_musl_tools() {
    print_status "Setting up musl tools..."
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - try different approaches
        if command -v brew &> /dev/null; then
            # Try to find musl tools in Homebrew
            if brew list messense/macos-cross-toolchains/x86_64-unknown-linux-musl &> /dev/null 2>&1; then
                print_status "Using messense musl toolchain"
                export CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc"
                export CXX_x86_64_unknown_linux_musl="x86_64-linux-musl-g++"
                export AR_x86_64_unknown_linux_musl="x86_64-linux-musl-ar"
                export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="x86_64-linux-musl-gcc"
            elif command -v x86_64-linux-musl-gcc &> /dev/null; then
                print_status "Found existing musl toolchain"
                export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="x86_64-linux-musl-gcc"
            else
                print_warning "No musl toolchain found. Attempting brew installation..."
                # Try the messense tap which provides musl cross toolchains
                if ! brew tap messense/macos-cross-toolchains &> /dev/null; then
                    print_warning "Could not add messense tap"
                fi
                
                if brew install messense/macos-cross-toolchains/x86_64-unknown-linux-musl &> /dev/null; then
                    print_success "Installed musl toolchain successfully"
                    export CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc"
                    export CXX_x86_64_unknown_linux_musl="x86_64-linux-musl-g++"
                    export AR_x86_64_unknown_linux_musl="x86_64-linux-musl-ar"
                    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="x86_64-linux-musl-gcc"
                else
                    print_warning "Could not install musl toolchain. Will try Rust-only cross compilation."
                    # For simple projects, Rust can often cross-compile without external linkers
                    print_status "Using Rust built-in cross-compilation (may have limitations)"
                fi
            fi
            
            # Setup for aarch64 if needed
            case "$TARGET_ARCH" in
                "aarch64")
                    if command -v aarch64-linux-musl-gcc &> /dev/null; then
                        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-musl-gcc"
                    else
                        print_warning "aarch64 musl toolchain not found, using Rust built-in"
                    fi
                    ;;
            esac
        else
            print_warning "Homebrew not found. Using Rust built-in cross-compilation."
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux - install musl-tools
        if command -v apt-get &> /dev/null; then
            if ! dpkg -l | grep -q musl-tools; then
                print_status "Installing musl-tools..."
                sudo apt-get update
                sudo apt-get install -y musl-tools musl-dev
            fi
        elif command -v yum &> /dev/null; then
            if ! rpm -q musl-gcc &> /dev/null; then
                print_status "Installing musl-gcc..."
                sudo yum install -y musl-gcc musl-libc-static
            fi
        fi
        
        # Setup environment for native Linux
        case "$TARGET_ARCH" in
            "x86_64")
                export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="musl-gcc"
                ;;
            "aarch64")
                if command -v aarch64-linux-gnu-gcc &> /dev/null; then
                    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-gnu-gcc"
                fi
                ;;
        esac
    fi
    
    print_success "Musl tools configured"
}

# Main function
main() {
    echo "Fechatter Build Script"
    echo "====================="
    echo "Build Mode: $BUILD_MODE"
    echo "Target Dir: $TARGET_DIR"
    echo ""
    
    # Initialize log file
    echo "Build started at $(date)" > "$LOG_FILE"
    
    # Execute build pipeline
    check_requirements
    setup_environment
    
    if [[ "${CLEAN_BUILD:-false}" == "true" ]]; then
        clean_build
    fi
    
    build_all_services
    verify_builds
    show_summary
    
    print_status "Build log saved to: $LOG_FILE"
}

# Handle command line arguments
case "${1:-build}" in
    "clean")
        clean_build
        ;;
    "check")
        check_requirements
        ;;
    "build")
        main
        ;;
    "multi-arch")
        # Build for multiple architectures
        print_status "Building for multiple architectures..."
        export CROSS_COMPILE="true"
        
        # Build x86_64 first
        export TARGET_ARCH="x86_64"
        main
        
        # Move x86_64 binaries to arch-specific directory
        mkdir -p "$TARGET_DIR/x86_64"
        mv "$TARGET_DIR"/* "$TARGET_DIR/x86_64/" 2>/dev/null || true
        
        print_status "Building ARM64 version..."
        # Build aarch64
        export TARGET_ARCH="aarch64"  
        main
        
        # Move aarch64 binaries to arch-specific directory
        mkdir -p "$TARGET_DIR/aarch64"
        mv "$TARGET_DIR"/* "$TARGET_DIR/aarch64/" 2>/dev/null || true
        # Move x86_64 back
        mv "$TARGET_DIR/x86_64"/* "$TARGET_DIR/" 2>/dev/null || true
        
        print_success "Multi-architecture build completed!"
        print_status "x86_64 binaries: $TARGET_DIR/"
        print_status "ARM64 binaries: $TARGET_DIR/aarch64/"
        ;;
    "linux")
        # Build for Linux containers (x86_64)
        export CROSS_COMPILE="true"
        export TARGET_ARCH="x86_64"
        main
        ;;
    "linux-arm64")
        # Build for Linux containers (aarch64/ARM64)  
        export CROSS_COMPILE="true"
        export TARGET_ARCH="arm64"
        main
        ;;
    "cross")
        # Cross-compile with specified target
        if [[ -n "${2:-}" ]]; then
            export CROSS_COMPILE="true" 
            export TARGET_ARCH="$2"
            main
        else
            print_error "Please specify target architecture: x86_64, aarch64, or arm64"
            exit 1
        fi
        ;;
    "help"|"--help"|"-h")
        cat << EOF
Fechatter Build Script - Production-ready cross-compilation support

Usage: $0 [COMMAND] [OPTIONS]

Commands:
  build         Build for current platform (default)
  multi-arch    Build for multiple architectures (x86_64 + ARM64)
  linux         Build for Linux x86_64 containers (musl static)
  linux-arm64   Build for Linux ARM64 containers (musl static)  
  cross TARGET  Cross-compile for specified target architecture
  clean         Clean previous builds
  check         Check build requirements
  help          Show this help message

Cross-compile Targets:
  x86_64        Linux x86_64 (Intel/AMD 64-bit)
  aarch64       Linux ARM64 (Apple Silicon, ARM servers)
  arm64         Alias for aarch64

Environment Variables:
  BUILD_MODE    Build mode: release (default) or debug
  CROSS_COMPILE Enable cross-compilation: true, false, or auto
  TARGET_ARCH   Target architecture: x86_64, aarch64, arm64, or auto
  CLEAN_BUILD   Clean before build: true or false

Examples:
  $0 build                    # Native build for current platform
  $0 multi-arch               # Build for multiple architectures
  $0 linux                    # Build for Linux x86_64 containers
  $0 linux-arm64             # Build for Linux ARM64 containers
  $0 cross x86_64            # Cross-compile for Linux x86_64
  TARGET_ARCH=aarch64 $0 build # Build for ARM64 via environment
  CLEAN_BUILD=true $0 linux   # Clean build for Linux containers

Container Deployment:
  For Docker containers, use 'linux' or 'linux-arm64' commands to build
  statically-linked binaries that work with Alpine Linux base images.
  
  For multi-platform deployment, use 'multi-arch' to build both architectures,
  then create multi-platform Docker images with:
    docker buildx build --platform linux/amd64,linux/arm64 -t myapp .

Requirements:
  - Rust toolchain with rustup
  - Cross-compilation targets (installed automatically)
  - musl-cross tools (macOS) or musl-tools (Linux)
EOF
        exit 0
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac 