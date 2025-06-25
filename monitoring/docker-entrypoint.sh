#!/bin/sh

# Docker entrypoint for Prometheus with environment variable substitution
# This allows dynamic configuration based on Fly.io environment

set -e

echo "🚀 Starting Prometheus with environment substitution..."

# Default values for environment variables
export FLY_REGION=${FLY_REGION:-"nrt"}
export FLY_APP_NAME=${FLY_APP_NAME:-"fechatter"}

# Substitute environment variables in prometheus config
envsubst < /etc/prometheus/prometheus.yml > /tmp/prometheus.yml

# Validate the configuration
if ! /bin/prometheus --config.file=/tmp/prometheus.yml --dry-run; then
    echo "❌ Prometheus configuration validation failed"
    exit 1
fi

echo "✅ Prometheus configuration validated successfully"

# Move the processed config to the final location
mv /tmp/prometheus.yml /etc/prometheus/prometheus.yml

# Print configuration info
echo "📊 Prometheus Configuration:"
echo "  Region: $FLY_REGION"
echo "  App Name: $FLY_APP_NAME"
echo "  Config: /etc/prometheus/prometheus.yml"

# Check if Grafana Cloud credentials are provided
if [ -n "$GRAFANA_PROMETHEUS_URL" ]; then
    echo "🔗 Grafana Cloud integration enabled"
else
    echo "⚠️  Grafana Cloud integration not configured"
fi

echo "🚀 Starting Prometheus server..."

# Start Prometheus with the provided arguments
exec /bin/prometheus "$@"