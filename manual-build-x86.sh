#!/bin/bash

# ============================================================================
# Manual x86_64 Cross-compilation Guide
# ============================================================================
#
# 🎯 PURPOSE: Step-by-step manual compilation of each binary crate
# 🔧 STRATEGY: Individual crate compilation with error isolation
# 🚀 USAGE: Follow the steps below or run specific sections
#
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Target architecture
TARGET="x86_64-unknown-linux-gnu"

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo "============================================================================"
echo "🎯 Manual x86_64 Cross-compilation Guide"
echo "============================================================================"
echo ""
echo "Target: $TARGET"
echo "Mode: Manual step-by-step compilation"
echo ""

# Create target directories
mkdir -p target/$TARGET/release
mkdir -p target/main/release

echo "📋 Binary Crates to Compile:"
echo "1. fechatter_server (Main chat server)"
echo "2. analytics_server (Analytics service)"  
echo "3. notify_server (Notification service)"
echo "4. fechatter_gateway (API gateway)"
echo "5. bot (Bot service from bot_server)"
echo "6. indexer (Indexer service from bot_server)"
echo ""

echo "🔧 Manual Compilation Commands:"
echo "============================================================================"

echo ""
echo "# Step 1: Core Library (dependency for others)"
echo "cross build --release --target $TARGET -p fechatter_core"

echo ""
echo "# Step 2: Protocol Buffers (may need protoc)"
echo "cross build --release --target $TARGET -p fechatter_protos"

echo ""
echo "# Step 3: Main Server"
echo "cross build --release --target $TARGET -p fechatter_server"

echo ""
echo "# Step 4: Analytics Server"
echo "cross build --release --target $TARGET -p analytics_server"

echo ""
echo "# Step 5: Notification Server"
echo "cross build --release --target $TARGET -p notify_server"

echo ""
echo "# Step 6: Gateway"
echo "cross build --release --target $TARGET -p fechatter_gateway"

echo ""
echo "# Step 7: Bot Service (two binaries)"
echo "cross build --release --target $TARGET --bin bot"
echo "cross build --release --target $TARGET --bin indexer"

echo ""
echo "============================================================================"
echo "📦 After compilation, copy binaries:"
echo "============================================================================"

echo ""
echo "# Copy all compiled binaries to Docker build location"
echo "cp target/$TARGET/release/fechatter_server target/main/release/"
echo "cp target/$TARGET/release/analytics_server target/main/release/"
echo "cp target/$TARGET/release/notify_server target/main/release/"
echo "cp target/$TARGET/release/fechatter_gateway target/main/release/"
echo "cp target/$TARGET/release/bot target/main/release/bot_server"
echo "cp target/$TARGET/release/indexer target/main/release/"

echo ""
echo "============================================================================"
echo "🐳 Docker Build Commands:"
echo "============================================================================"

echo ""
echo "# Build individual service images"
echo "docker compose -f docker-compose.local.yml build fechatter-server"
echo "docker compose -f docker-compose.local.yml build analytics-server"
echo "docker compose -f docker-compose.local.yml build notify-server"
echo "docker compose -f docker-compose.local.yml build fechatter-gateway"

echo ""
echo "# Or build all at once"
echo "docker compose -f docker-compose.local.yml build"

echo ""
echo "============================================================================"
echo "🚀 Deployment Commands:"
echo "============================================================================"

echo ""
echo "# Start infrastructure"
echo "docker compose -f docker-compose.local.yml --profile infrastructure up -d"

echo ""
echo "# Start core services"
echo "docker compose -f docker-compose.local.yml --profile core up -d"

echo ""
echo "# Check service health"
echo "curl http://localhost:6688/health  # Fechatter Server"
echo "curl http://localhost:6690/health  # Analytics Server"
echo "curl http://localhost:6687/health  # Notify Server"

echo ""
echo "============================================================================"
echo "❗ Common Issues and Solutions:"
echo "============================================================================"

echo ""
echo "🔹 protoc errors in fechatter_protos:"
echo "   Solution: brew install protobuf (on macOS) or apt install protobuf-compiler"

echo ""
echo "🔹 OpenSSL linking errors:"
echo "   Solution: Already handled with 'vendored' feature in Cargo.toml"

echo ""
echo "🔹 Cross tool not found:"
echo "   Solution: cargo install cross --git https://github.com/cross-rs/cross"

echo ""
echo "🔹 Binary not found in Docker:"
echo "   Solution: Check if binary was copied to target/main/release/"

echo ""
echo "============================================================================"
echo "🎯 Quick Start - Copy and run these commands:"
echo "============================================================================"

cat << 'EOF'

# 1. Compile core dependencies first
cross build --release --target x86_64-unknown-linux-gnu -p fechatter_core

# 2. Try protobuf (may fail if protoc issues)
cross build --release --target x86_64-unknown-linux-gnu -p fechatter_protos || echo "Skip if protoc errors"

# 3. Compile main services
cross build --release --target x86_64-unknown-linux-gnu -p fechatter_server
cross build --release --target x86_64-unknown-linux-gnu -p analytics_server  
cross build --release --target x86_64-unknown-linux-gnu -p notify_server
cross build --release --target x86_64-unknown-linux-gnu -p fechatter_gateway

# 4. Compile bot services
cross build --release --target x86_64-unknown-linux-gnu --bin bot
cross build --release --target x86_64-unknown-linux-gnu --bin indexer

# 5. Copy binaries for Docker
mkdir -p target/main/release
cp target/x86_64-unknown-linux-gnu/release/fechatter_server target/main/release/
cp target/x86_64-unknown-linux-gnu/release/analytics_server target/main/release/
cp target/x86_64-unknown-linux-gnu/release/notify_server target/main/release/
cp target/x86_64-unknown-linux-gnu/release/fechatter_gateway target/main/release/
cp target/x86_64-unknown-linux-gnu/release/bot target/main/release/bot_server
cp target/x86_64-unknown-linux-gnu/release/indexer target/main/release/

# 6. Build Docker images  
docker compose -f docker-compose.local.yml build

# 7. Deploy
docker compose -f docker-compose.local.yml --profile infrastructure up -d
docker compose -f docker-compose.local.yml --profile core up -d

EOF

echo ""
echo "============================================================================" 