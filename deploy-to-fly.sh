#!/bin/bash
# deploy-to-fly.sh - Quick deployment script for Fly.io

set -e

echo "🚀 Fechatter Fly.io Deployment"
echo "=============================="
echo ""

# Check if production environment file exists
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

# Make scripts executable
chmod +x fly/deploy-production.sh
chmod +x fly/prepare-configs.sh
chmod +x scripts/fly-start.sh

# Run the deployment
echo "Starting deployment..."
echo ""
./fly/deploy-production.sh

echo ""
echo "✅ Deployment script completed!"
echo ""
echo "📚 Next steps:"
echo "   - Check application status: flyctl status -a fechatter-prod"
echo "   - View logs: flyctl logs -a fechatter-prod"
echo "   - Access dashboard: https://fly.io/apps/fechatter-prod" 