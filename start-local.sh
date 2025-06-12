#!/bin/bash

# ============================================================================
# Fechatter Local Development - One-Click Startup Script
# ============================================================================
#
# 🎯 PURPOSE: Single command to start complete Fechatter ecosystem
# 🚀 USAGE: ./start-local.sh
#
# ============================================================================

set -e

echo "🚀 Starting Fechatter Local Development Environment..."
echo ""

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Check if cross-compiled binaries exist
if [ ! -f "target/main/release/fechatter_server" ]; then
    echo "🔧 Cross-compiled binaries not found. Building now..."
    echo "   This may take a few minutes on first run..."
    ./build-cross.sh --profile release
fi

echo "🐳 Starting all Fechatter services with Docker Compose..."
echo ""

# Start complete ecosystem
docker compose -f docker-compose.local.yml --profile gateway up -d

echo ""
echo "✅ Fechatter ecosystem started successfully!"
echo ""
echo "📋 Service Status Check:"
echo "   Waiting for services to become healthy..."

# Wait a moment for services to start
sleep 5

# Check service status
docker compose -f docker-compose.local.yml ps

echo ""
echo "🌐 Available Services:"
echo "   ├─ 🌐 Gateway (API Entry):  http://localhost:8080"
echo "   ├─ 🎯 Main Server:          http://localhost:6688"  
echo "   ├─ 📊 Analytics:            http://localhost:6690"
echo "   ├─ 📣 Notifications:        http://localhost:6687"
echo "   ├─ 🤖 Bot Service:          http://localhost:6686"
echo "   ├─ 🗄️  Database:             postgresql://fechatter:fechatter_password@localhost:5432/fechatter"
echo "   ├─ 📦 Redis:                redis://:fechatter_redis_pass@localhost:6379"
echo "   ├─ 🔍 Search:               http://localhost:7700"
echo "   └─ 📈 ClickHouse:           http://localhost:8123"
echo ""
echo "🎉 Ready to use! Visit http://localhost:8080 to access the API gateway."
echo ""
echo "🛑 To stop all services: docker compose -f docker-compose.local.yml down"
echo "📊 To view logs:         docker compose -f docker-compose.local.yml logs -f [service-name]"
echo "" 