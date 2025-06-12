#!/bin/bash

# ============================================================================
# Individual Crate Compilation Script
# ============================================================================
#
# 🎯 PURPOSE: Compile each binary crate individually to isolate issues
# 🔧 STRATEGY: Sequential compilation with error handling
# 🚀 USAGE: ./build-individual.sh
#
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Binary crates to compile
BINARY_CRATES=(
    "fechatter_core"      # Library first
    "fechatter_protos"    # Protobuf definitions
    "fechatter_server"    # Main server
    "analytics_server"    # Analytics service
    "notify_server"       # Notification service  
    "bot_server"          # Bot service
    "fechatter_gateway"   # Gateway service
    "ai_sdk"              # AI SDK
)

# Initialize counters
SUCCESS_COUNT=0
FAILED_COUNT=0
FAILED_CRATES=()

# Create target directory
mkdir -p target/main/release

log "Starting individual crate compilation..."
echo "=================================================="

# Compile each crate individually
for crate in "${BINARY_CRATES[@]}"; do
    echo ""
    log "🔄 Compiling crate: $crate"
    echo "--------------------------------------------------"
    
    # Check if crate directory exists
    if [ ! -d "$crate" ]; then
        warn "Directory $crate not found, skipping..."
        continue
    fi
    
    # Try to compile the crate
    if cargo build --release -p "$crate" 2>&1; then
        success "✅ $crate compiled successfully"
        ((SUCCESS_COUNT++))
    else
        error "❌ $crate compilation failed"
        ((FAILED_COUNT++))
        FAILED_CRATES+=("$crate")
        
        # Continue with next crate instead of failing
        warn "Continuing with next crate..."
    fi
    
    echo "--------------------------------------------------"
done

echo ""
echo "=================================================="
echo "📊 COMPILATION SUMMARY"
echo "=================================================="
echo "✅ Successful: $SUCCESS_COUNT"
echo "❌ Failed: $FAILED_COUNT"

if [ $FAILED_COUNT -gt 0 ]; then
    echo ""
    echo "💥 Failed crates:"
    for failed_crate in "${FAILED_CRATES[@]}"; do
        echo "   • $failed_crate"
    done
    
    echo ""
    echo "🔧 Next steps for failed crates:"
    echo "1. Check individual error messages above"
    echo "2. Fix compilation errors one by one"
    echo "3. Re-run: cargo build --release -p <crate_name>"
fi

if [ $SUCCESS_COUNT -gt 0 ]; then
    echo ""
    echo "🎯 Successfully compiled binaries:"
    ls -la target/main/release/ 2>/dev/null | grep -E "(fechatter|analytics|notify|bot)" || echo "   No binaries found in target/main/release/"
fi

echo "=================================================="

# Exit with error code if any compilation failed
if [ $FAILED_COUNT -gt 0 ]; then
    exit 1
else
    success "🎉 All crates compiled successfully!"
fi 