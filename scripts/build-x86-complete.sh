#!/bin/bash
# build-x86-complete.sh - Complete x86_64 cross-compilation and packaging workflow
# This script performs the entire process from protobuf setup to Docker packaging

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
TARGET="x86_64-unknown-linux-musl"
BUILD_DIR="target/main/$TARGET/release"
PACKAGE_DIR="docker/binaries/x86_64"
BUILD_LOG="build-complete-x86.log"

# Service definitions
SERVICES=("fechatter_server" "analytics_server" "notify_server" "fechatter_gateway")
BOT_BINARIES=("bot" "indexer")

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Function to log with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$BUILD_LOG"
}

# Function to check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    local all_good=true
    
    # Check Rust toolchain
    if command -v rustc &> /dev/null; then
        local rust_version=$(rustc --version)
        print_status "✅ Rust: $rust_version"
        log_message "Rust version: $rust_version"
    else
        print_error "❌ Rust not found"
        all_good=false
    fi
    
    # Check target
    if rustup target list --installed | grep -q "$TARGET"; then
        print_status "✅ Target $TARGET is installed"
    else
        print_warning "⚠️  Installing target $TARGET"
        rustup target add "$TARGET"
        print_status "✅ Target $TARGET installed"
    fi
    
    # Check musl tools
    if command -v x86_64-linux-musl-gcc &> /dev/null; then
        print_status "✅ musl cross-compilation tools available"
    else
        print_warning "⚠️  musl tools not found, but continuing..."
    fi
    
    # Check protoc
    if command -v protoc &> /dev/null; then
        local protoc_version=$(protoc --version)
        print_status "✅ Protocol Buffers: $protoc_version"
        log_message "protoc version: $protoc_version"
    else
        print_error "❌ protoc not found"
        all_good=false
    fi
    
    if [[ "$all_good" == false ]]; then
        print_error "Prerequisites check failed"
        exit 1
    fi
    
    print_status "All prerequisites satisfied"
}

# Function to build protobuf library
build_protobuf() {
    print_header "Building Protocol Buffers Library"
    print_step "Compiling fechatter_protos for $TARGET"
    
    log_message "Starting protobuf compilation"
    
    if cargo build --release --target "$TARGET" -p fechatter_protos; then
        print_status "✅ fechatter_protos compiled successfully"
        log_message "✅ fechatter_protos: SUCCESS"
        return 0
    else
        print_error "❌ fechatter_protos compilation failed"
        log_message "❌ fechatter_protos: FAILED"
        return 1
    fi
}

# Function to build core library
build_core() {
    print_header "Building Core Library"
    print_step "Compiling fechatter_core for $TARGET"
    
    log_message "Starting core library compilation"
    
    if cargo build --release --target "$TARGET" -p fechatter_core; then
        print_status "✅ fechatter_core compiled successfully"
        log_message "✅ fechatter_core: SUCCESS"
        return 0
    else
        print_error "❌ fechatter_core compilation failed"
        log_message "❌ fechatter_core: FAILED"
        return 1
    fi
}

# Function to build individual service
build_service() {
    local service_name="$1"
    print_step "Building $service_name"
    
    log_message "Starting $service_name compilation"
    
    if cargo build --release --target "$TARGET" -p "$service_name"; then
        print_status "✅ $service_name compiled successfully"
        log_message "✅ $service_name: SUCCESS"
        return 0
    else
        print_error "❌ $service_name compilation failed"
        log_message "❌ $service_name: FAILED"
        return 1
    fi
}

# Function to build bot binaries
build_bot_binaries() {
    print_header "Building Bot Server Binaries"
    
    local success_count=0
    
    for binary_name in "${BOT_BINARIES[@]}"; do
        print_step "Building bot binary: $binary_name"
        log_message "Starting bot $binary_name compilation"
        
        if cargo build --release --target "$TARGET" -p bot_server --bin "$binary_name"; then
            print_status "✅ Bot $binary_name compiled successfully"
            log_message "✅ bot_server.$binary_name: SUCCESS"
            success_count=$((success_count + 1))
        else
            print_error "❌ Bot $binary_name compilation failed"
            log_message "❌ bot_server.$binary_name: FAILED"
        fi
    done
    
    if [[ $success_count -eq ${#BOT_BINARIES[@]} ]]; then
        print_status "All bot binaries compiled successfully"
        return 0
    else
        print_error "Some bot binaries failed to compile"
        return 1
    fi
}

# Function to build all services
build_all_services() {
    print_header "Building All Services"
    
    local total_services=$((${#SERVICES[@]} + 1))  # +1 for bot_server
    local successful_builds=0
    
    # Build main services
    for service in "${SERVICES[@]}"; do
        if build_service "$service"; then
            successful_builds=$((successful_builds + 1))
        fi
    done
    
    # Build bot binaries
    if build_bot_binaries; then
        successful_builds=$((successful_builds + 1))
    fi
    
    print_header "Build Summary"
    print_status "Services processed: $total_services"
    print_status "Successful builds: $successful_builds"
    
    if [[ $successful_builds -eq $total_services ]]; then
        print_status "🎉 All services compiled successfully!"
        return 0
    else
        print_error "⚠️  Some services failed to compile"
        return 1
    fi
}

# Function to collect and package binaries
package_binaries() {
    print_header "Packaging Binaries"
    
    if [[ -x "./scripts/collect-x86-binaries.sh" ]]; then
        print_step "Running binary collection script"
        if ./scripts/collect-x86-binaries.sh all; then
            print_status "✅ Binaries packaged successfully"
            log_message "✅ Binary packaging: SUCCESS"
            return 0
        else
            print_error "❌ Binary packaging failed"
            log_message "❌ Binary packaging: FAILED"
            return 1
        fi
    else
        print_error "❌ Binary collection script not found"
        return 1
    fi
}

# Function to verify build results
verify_build() {
    print_header "Verifying Build Results"
    
    local expected_binaries=()
    for service in "${SERVICES[@]}"; do
        expected_binaries+=("$service")
    done
    for binary in "${BOT_BINARIES[@]}"; do
        expected_binaries+=("$binary")
    done
    
    local found_binaries=0
    
    for binary_name in "${expected_binaries[@]}"; do
        local binary_path="$BUILD_DIR/$binary_name"
        if [[ -f "$binary_path" ]]; then
            local size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null || echo "unknown")
            print_status "✅ $binary_name (${size} bytes)"
            found_binaries=$((found_binaries + 1))
        else
            print_error "❌ $binary_name not found"
        fi
    done
    
    print_status "Found $found_binaries/${#expected_binaries[@]} expected binaries"
    
    if [[ $found_binaries -eq ${#expected_binaries[@]} ]]; then
        print_status "🎉 All binaries verified!"
        return 0
    else
        print_error "⚠️  Some binaries are missing"
        return 1
    fi
}

# Function to show final summary
show_summary() {
    print_header "Build Summary Report"
    
    echo "📊 Build Statistics:"
    echo "  • Target: $TARGET"
    echo "  • Build date: $(date)"
    echo "  • Build log: $BUILD_LOG"
    
    if [[ -f "$PACKAGE_DIR/MANIFEST.txt" ]]; then
        echo "  • Package manifest: $PACKAGE_DIR/MANIFEST.txt"
        echo ""
        echo "📦 Packaged binaries:"
        grep -v "^#" "$PACKAGE_DIR/MANIFEST.txt" | while read -r line; do
            echo "  • $line"
        done
    fi
    
    echo ""
    print_status "🚀 Build completed successfully!"
    print_status "Ready for Docker packaging and deployment"
}

# Function to clean previous builds
clean_build() {
    print_header "Cleaning Previous Builds"
    
    if [[ -d "$BUILD_DIR" ]]; then
        print_step "Cleaning target directory"
        cargo clean --target "$TARGET"
        print_status "Target directory cleaned"
    fi
    
    if [[ -d "$PACKAGE_DIR" ]]; then
        print_step "Cleaning package directory"
        rm -rf "$PACKAGE_DIR"
        print_status "Package directory cleaned"
    fi
    
    if [[ -f "$BUILD_LOG" ]]; then
        rm -f "$BUILD_LOG"
    fi
    
    print_status "Clean completed"
}

# Main execution function
main() {
    local command="${1:-build}"
    
    # Initialize build log
    echo "# Fechatter x86_64 Complete Build Log" > "$BUILD_LOG"
    echo "# Started: $(date)" >> "$BUILD_LOG"
    echo "" >> "$BUILD_LOG"
    
    case "$command" in
        "build")
            print_header "Fechatter x86_64 Cross-Compilation Build"
            check_prerequisites
            build_protobuf
            build_core
            build_all_services
            package_binaries
            verify_build
            show_summary
            ;;
        "clean")
            clean_build
            ;;
        "verify")
            verify_build
            ;;
        "package")
            package_binaries
            ;;
        *)
            echo "Usage: $0 {build|clean|verify|package}"
            echo ""
            echo "Commands:"
            echo "  build   - Complete cross-compilation build (default)"
            echo "  clean   - Clean previous builds"
            echo "  verify  - Verify build results"
            echo "  package - Package binaries only"
            exit 1
            ;;
    esac
}

# Error handling
trap 'print_error "Build failed at line $LINENO"; log_message "Build failed at line $LINENO"' ERR

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi 