#!/bin/bash

# ============================================================================
# Production Cross-compilation Build Script for x86_64 Linux
# ============================================================================
#
# 🎯 PURPOSE: Compile binary crates for x86_64-unknown-linux-gnu architecture
# 🔧 STRATEGY: Use Cross tool with production optimizations
# 🚀 USAGE: ./build-cross.sh [--clean] [--profile <profile>]
#
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Target architecture and default profile
TARGET="x86_64-unknown-linux-gnu"
BUILD_PROFILE="release"
CLEAN_BUILD=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --profile)
            BUILD_PROFILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--clean] [--profile <profile>]"
            exit 1
            ;;
    esac
done

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Binary crates to compile in dependency order
BINARY_CRATES=(
    "fechatter_core"      # Core library - compile first
    "fechatter_protos"    # Protocol definitions
    "fechatter_server"    # Main server binary
    "analytics_server"    # Analytics service binary
    "notify_server"       # Notification service binary
    "bot_server"          # Bot service binary
    "fechatter_gateway"   # Gateway service binary
    "ai_sdk"              # AI SDK binary
)

# Initialize counters
SUCCESS_COUNT=0
FAILED_COUNT=0
FAILED_CRATES=()
START_TIME=$(date +%s)

# Create target directories
mkdir -p target/$TARGET/$BUILD_PROFILE
mkdir -p target/main/$BUILD_PROFILE

log "Starting cross-compilation for target: $TARGET (profile: $BUILD_PROFILE)"
echo "=================================================="

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    error "Cross tool is not installed."
    log "Installing cross tool..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Clean build if requested
if [ "$CLEAN_BUILD" = true ]; then
    log "🧹 Cleaning previous build artifacts..."
    cargo clean
    rm -rf target/$TARGET/$BUILD_PROFILE/*
    rm -rf target/main/$BUILD_PROFILE/*
fi

# Pre-flight checks
log "🔍 Pre-flight checks..."
if [ ! -f "Cargo.toml" ]; then
    error "Cargo.toml not found. Please run from project root."
    exit 1
fi

# Compile each crate individually for better error isolation
for crate in "${BINARY_CRATES[@]}"; do
    echo ""
    log "🔄 Cross-compiling crate: $crate for $TARGET"
    echo "--------------------------------------------------"
    
    # Check if crate directory exists
    if [ ! -d "$crate" ]; then
        warn "Directory $crate not found, skipping..."
        continue
    fi
    
    # Check if crate has a binary target
    if [ ! -f "$crate/Cargo.toml" ]; then
        warn "No Cargo.toml found in $crate, skipping..."
        continue
    fi
    
    # Try to cross-compile the crate with better error output
    if cross build --$BUILD_PROFILE --target $TARGET -p "$crate" --color=always 2>&1; then
        success "✅ $crate cross-compiled successfully"
        ((SUCCESS_COUNT++))
    else
        error "❌ $crate cross-compilation failed"
        ((FAILED_COUNT++))
        FAILED_CRATES+=("$crate")
        
        # Continue with next crate for resilient builds
        warn "Continuing with next crate..."
    fi
    
    echo "--------------------------------------------------"
done

echo ""
echo "=================================================="
echo "📊 CROSS-COMPILATION SUMMARY"
echo "=================================================="

# Calculate build time
END_TIME=$(date +%s)
BUILD_DURATION=$((END_TIME - START_TIME))

echo "🎯 Target: $TARGET"
echo "📝 Profile: $BUILD_PROFILE"
echo "⏱️  Build Time: ${BUILD_DURATION}s"
echo "✅ Successful: $SUCCESS_COUNT"
echo "❌ Failed: $FAILED_COUNT"

if [ $FAILED_COUNT -gt 0 ]; then
    echo ""
    echo "💥 Failed crates:"
    for failed_crate in "${FAILED_CRATES[@]}"; do
        echo "   • $failed_crate"
    done
    
    echo ""
    echo "🔧 Troubleshooting steps:"
    echo "1. Check individual error messages above"
    echo "2. Verify dependencies and features in Cargo.toml"
    echo "3. Re-run with: cross build --$BUILD_PROFILE --target $TARGET -p <crate_name>"
    echo "4. Try clean build: $0 --clean --profile $BUILD_PROFILE"
fi

if [ $SUCCESS_COUNT -gt 0 ]; then
    echo ""
    log "📦 Processing compiled binaries..."
    
    # List compiled binaries
    echo "🎯 Successfully compiled binaries in target/$TARGET/$BUILD_PROFILE/:"
    ls -la target/$TARGET/$BUILD_PROFILE/ 2>/dev/null | grep -E "^-rwx.*" | grep -v "\.d$" || echo "   No executable binaries found"
    
    # Copy binaries to main release directory for Docker builds
    log "📂 Copying binaries to target/main/$BUILD_PROFILE/ for Docker compatibility..."
    
    # Ensure target directory exists
    mkdir -p target/main/$BUILD_PROFILE
    
    # Copy all executable binaries (not just specific ones)
    for binary in target/$TARGET/$BUILD_PROFILE/*; do
        if [ -f "$binary" ] && [ -x "$binary" ] && [[ ! "$binary" =~ \.d$ ]]; then
            cp "$binary" target/main/$BUILD_PROFILE/
            log "Copied: $(basename "$binary")"
        fi
    done
    
    echo ""
    echo "📂 Binaries available in target/main/$BUILD_PROFILE/:"
    ls -la target/main/$BUILD_PROFILE/ | grep -E "^-rwx.*" | grep -v "\.d$" || echo "   No executable binaries found"
    
    # Generate binary manifest for Docker builds
    log "📋 Generating binary manifest..."
    echo "# Binary Manifest - Generated $(date)" > target/main/$BUILD_PROFILE/manifest.txt
    echo "# Target: $TARGET" >> target/main/$BUILD_PROFILE/manifest.txt
    echo "# Profile: $BUILD_PROFILE" >> target/main/$BUILD_PROFILE/manifest.txt
    echo "# Build Duration: ${BUILD_DURATION}s" >> target/main/$BUILD_PROFILE/manifest.txt
    echo "" >> target/main/$BUILD_PROFILE/manifest.txt
    ls -la target/main/$BUILD_PROFILE/ | grep -E "^-rwx.*" | grep -v "\.d$" >> target/main/$BUILD_PROFILE/manifest.txt
fi

echo "=================================================="

# Exit with appropriate code
if [ $FAILED_COUNT -gt 0 ]; then
    error "Build completed with $FAILED_COUNT failed crates"
    exit 1
else
    success "🎉 All crates cross-compiled successfully!"
    echo ""
    echo "🚀 Next steps for Docker deployment:"
    echo "1. Build Docker images: docker compose -f docker-compose.local.yml build"
    echo "2. Start infrastructure: docker compose -f docker-compose.local.yml --profile infrastructure up -d"
    echo "3. Start services: docker compose -f docker-compose.local.yml --profile core up -d"
    echo "4. Check logs: docker compose -f docker-compose.local.yml logs -f"
fi 