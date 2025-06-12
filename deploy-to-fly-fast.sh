#!/bin/bash
# deploy-to-fly-fast.sh - Fast Fly.io deployment with pre-compiled binaries

set -e

echo "🚀 Fechatter Fast Fly.io Deployment"
echo "==================================="
echo ""
echo "This uses pre-compiled binaries to save ~7 minutes per deployment"
echo ""

# Check if binaries exist
if [ ! -d "target/musl/release" ] || [ -z "$(ls -A target/musl/release 2>/dev/null)" ]; then
    echo "⚠️  No pre-compiled musl binaries found!"
    echo ""
    echo "Building musl static binaries (this will take ~7 minutes)..."
    echo "This is a one-time build. Future deployments will be fast!"
    echo ""
    
    # Make build script executable
    chmod +x build-musl.sh
    
    # Build binaries
    ./build-musl.sh
    
    echo ""
    echo "✅ Binaries built successfully!"
    echo ""
fi

# Check environment configuration
if [ ! -f ".env.production" ]; then
    echo "⚠️  Production environment file not found!"
    echo ""
    echo "Creating from template..."
    cp fly/env.production.template .env.production
    echo ""
    echo "📝 Please edit .env.production with your configuration:"
    echo "   - Database settings (Fly Postgres or external)"
    echo "   - Redis settings (Upstash or external)"
    echo "   - External services (NATS, Meilisearch, ClickHouse)"
    echo "   - Security keys (JWT_SECRET)"
    echo ""
    echo "Then run this script again."
    exit 1
fi

# Make all scripts executable
chmod +x fly/deploy-production-fast.sh
chmod +x fly/prepare-configs.sh
chmod +x scripts/fly-start.sh

# Show deployment options
echo "📋 Deployment Options:"
echo ""
echo "1. Fast deployment (recommended)"
echo "   Uses pre-compiled binaries, deploys in ~2 minutes"
echo ""
echo "2. Rebuild and deploy"
echo "   Rebuilds binaries first, then deploys (~9 minutes)"
echo ""

read -p "Choose option (1 or 2): " choice

case $choice in
    1)
        echo ""
        echo "Starting fast deployment..."
        ./fly/deploy-production-fast.sh
        ;;
    2)
        echo ""
        echo "Rebuilding binaries..."
        ./build-musl.sh
        echo ""
        echo "Starting deployment..."
        ./fly/deploy-production-fast.sh
        ;;
    *)
        echo "Invalid choice. Please run again and select 1 or 2."
        exit 1
        ;;
esac

echo ""
echo "✅ Deployment process completed!" 