#!/bin/bash
# build-musl.sh - Build musl static binaries for Fly.io deployment

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Check prerequisites
print_step "Checking prerequisites..."

# Check if musl target is installed
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
    print_info "Installing musl target..."
    rustup target add x86_64-unknown-linux-musl
fi

# Create output directory
OUTPUT_DIR="target/musl/release"
mkdir -p "$OUTPUT_DIR"

# Build configuration
PROFILE="${BUILD_PROFILE:-release}"
TARGET="x86_64-unknown-linux-musl"

print_step "Building musl static binaries..."
print_info "Profile: $PROFILE"
print_info "Target: $TARGET"

# Export required environment variables
export SQLX_OFFLINE=true
export RUST_LOG=info

# Build each service
SERVICES=(
    "fechatter_server"
    "fechatter_gateway"
    "notify_server"
    "analytics_server"
    "bot_server"
)

for service in "${SERVICES[@]}"; do
    print_info "Building $service..."
    
    if cargo build --release --target "$TARGET" --bin "$service"; then
        print_info "✅ $service built successfully"
        
        # Copy to output directory
        cp "target/$TARGET/release/$service" "$OUTPUT_DIR/"
        
        # Strip binary to reduce size
        strip "$OUTPUT_DIR/$service"
        
        # Show binary info
        file_size=$(ls -lh "$OUTPUT_DIR/$service" | awk '{print $5}')
        print_info "  Size: $file_size"
    else
        print_error "❌ Failed to build $service"
        exit 1
    fi
done

print_step "Build summary:"
print_info "Binaries location: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"

print_info ""
print_info "✅ All musl static binaries built successfully!"
print_info "Ready for Fly.io deployment" 