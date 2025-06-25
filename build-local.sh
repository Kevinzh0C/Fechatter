#!/bin/bash

# ============================================================================
# Local Cross-compilation Build Script
# ============================================================================
#
# 🎯 PURPOSE: Build all workspace binaries using Docker for x86_64 Linux
# 🔧 STRATEGY: Docker-based compilation with local binary extraction
# 🚀 USAGE: ./build-local.sh
#
# ============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
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

# Check if Docker is running
if ! docker info &> /dev/null; then
    error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Create target directory structure
log "Creating target directory structure..."
mkdir -p target/main/release

# Build the Docker image
log "Building Docker compilation image..."
if docker build -f Dockerfile.build -t fechatter-builder . --no-cache; then
    success "Docker build image created successfully"
else
    error "Failed to build Docker image"
    exit 1
fi

# Create temporary directory for binary extraction
TEMP_DIR=$(mktemp -d)
log "Using temporary directory: $TEMP_DIR"

# Run compilation and extract binaries
log "Running compilation inside Docker container..."
if docker run --rm -v "$TEMP_DIR:/output" fechatter-builder; then
    success "Compilation completed successfully"
else
    error "Compilation failed"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Copy binaries to target directory
log "Copying binaries to target/main/release/..."
if [ "$(ls -A $TEMP_DIR)" ]; then
    cp "$TEMP_DIR"/* target/main/release/
    success "Binaries copied successfully"
    
    # List compiled binaries
    log "Compiled binaries:"
    ls -la target/main/release/
else
    error "No binaries found in output directory"
    rm -rf "$TEMP_DIR"
    exit 1
fi

# Clean up
rm -rf "$TEMP_DIR"

# Make binaries executable
chmod +x target/main/release/*

success "Cross-compilation build completed!"
log "All binaries are ready in target/main/release/"

# Show build summary
echo ""
echo "==============================================="
echo "✅ BUILD SUMMARY"
echo "==============================================="
echo "📁 Target: x86_64-unknown-linux-gnu"
echo "📍 Location: target/main/release/"
echo "🗂️  Binaries:"
for binary in target/main/release/*; do
    if [ -f "$binary" ] && [ -x "$binary" ]; then
        echo "   • $(basename "$binary")"
    fi
done
echo "==============================================="
echo ""
echo "Next steps:"
echo "1. Build Docker images: docker-compose -f docker-compose.local.yml build"
echo "2. Start services: docker-compose -f docker-compose.local.yml up -d" 