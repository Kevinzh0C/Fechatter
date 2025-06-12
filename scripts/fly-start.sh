#!/bin/bash
# Production startup script for Fly.io deployment

set -e

# Colors for logging
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'  
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1" >&2; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1" >&2; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1" >&2; }

# Environment validation
if [ -z "$DATABASE_URL" ]; then
    log_error "DATABASE_URL not set"
    exit 1
fi

log_step "🚀 Starting Fechatter Production Deployment"

# Wait for database to be ready
log_step "⏳ Waiting for database connection..."
for i in {1..30}; do
    if pg_isready -d "$DATABASE_URL" > /dev/null 2>&1; then
        log_info "✅ Database ready"
        break
    fi
    if [ $i -eq 30 ]; then
        log_error "❌ Database connection timeout"
        exit 1
    fi
    sleep 2
done

# Run database migrations  
log_step "🔄 Running database migrations..."
cd /app
if [ -d "migrations" ]; then
    # Use sqlx migrate if available, otherwise use psql
    if command -v sqlx >/dev/null 2>&1; then
        sqlx migrate run --source migrations --database-url "$DATABASE_URL"
    else
        # Fallback to running migration files manually
        for migration in migrations/*.sql; do
            if [ -f "$migration" ]; then
                log_info "Running migration: $(basename "$migration")"
                psql "$DATABASE_URL" -f "$migration" || log_warn "Migration failed: $migration"
            fi
        done
    fi
    log_info "✅ Database migrations completed"
else
    log_warn "No migrations directory found"
fi

# Initialize ClickHouse if available and enabled
if [ -n "$CLICKHOUSE_URL" ] && [ "$ANALYTICS_ENABLED" = "true" ]; then
    log_step "🔄 Initializing ClickHouse analytics..."
    if [ -f "/app/analytics.sql" ]; then
        # Try to initialize ClickHouse tables
        curl -X POST "$CLICKHOUSE_URL" -d @/app/analytics.sql 2>/dev/null || log_warn "ClickHouse initialization failed"
    fi
fi

# Set up data directories with proper permissions
log_step "📁 Setting up data directories..."
mkdir -p /data/uploads /data/cache /data/logs
chown -R appuser:appuser /data
chmod 755 /data /data/uploads /data/cache /data/logs

# Validate binary permissions
log_step "🔧 Validating binaries..."
for binary in fechatter_server fechatter_gateway notify_server analytics_server bot_server; do
    if [ -x "/usr/local/bin/$binary" ]; then
        log_info "✅ $binary ready"
    else
        log_error "❌ $binary not executable"
        exit 1
    fi
done

# Start supervisor to manage all services
log_step "🎯 Starting supervisor with all services..."
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/fechatter.conf -n 