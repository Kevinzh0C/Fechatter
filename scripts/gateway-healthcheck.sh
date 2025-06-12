#!/bin/sh
# Docker health check script for Fechatter Gateway

# Configuration
HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://localhost:8080/health}"
TIMEOUT="${HEALTH_CHECK_TIMEOUT:-5}"

# Perform health check
if command -v curl >/dev/null 2>&1; then
    # Use curl if available
    response=$(curl -sf --max-time "$TIMEOUT" "$HEALTH_ENDPOINT" 2>/dev/null)
    exit_code=$?
elif command -v wget >/dev/null 2>&1; then
    # Fall back to wget
    response=$(wget -qO- --timeout="$TIMEOUT" "$HEALTH_ENDPOINT" 2>/dev/null)
    exit_code=$?
else
    echo "Neither curl nor wget available for health check"
    exit 1
fi

# Check response
if [ $exit_code -eq 0 ]; then
    # Check if response contains expected fields
    if echo "$response" | grep -q '"status".*"healthy"'; then
        exit 0
    else
        echo "Health check returned unexpected response: $response"
        exit 1
    fi
else
    echo "Health check failed with exit code: $exit_code"
    exit 1
fi