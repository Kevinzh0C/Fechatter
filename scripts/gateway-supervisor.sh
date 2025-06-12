#!/bin/bash
# Gateway Supervisor Script - Ensures high availability

set -euo pipefail

# Configuration
GATEWAY_BIN="${GATEWAY_BIN:-fechatter_gateway}"
GATEWAY_CONFIG="${GATEWAY_CONFIG:-gateway.yaml}"
RESTART_DELAY=5
MAX_RESTARTS=10
RESTART_WINDOW=300  # 5 minutes
LOG_FILE="${LOG_FILE:-/tmp/gateway-supervisor.log}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level=$1
    shift
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $*" | tee -a "$LOG_FILE"
}

# Signal handlers
cleanup() {
    log "INFO" "Supervisor shutting down..."
    if [[ -n "${GATEWAY_PID:-}" ]] && kill -0 "$GATEWAY_PID" 2>/dev/null; then
        log "INFO" "Stopping gateway process (PID: $GATEWAY_PID)..."
        kill -TERM "$GATEWAY_PID" || true
        sleep 2
        if kill -0 "$GATEWAY_PID" 2>/dev/null; then
            log "WARN" "Gateway didn't stop gracefully, forcing..."
            kill -KILL "$GATEWAY_PID" || true
        fi
    fi
    exit 0
}

trap cleanup EXIT INT TERM

# Check if gateway binary exists
if ! command -v "$GATEWAY_BIN" &> /dev/null; then
    log "ERROR" "Gateway binary not found: $GATEWAY_BIN"
    exit 1
fi

# Initialize restart tracking
declare -a restart_times=()

# Main supervisor loop
log "INFO" "Gateway Supervisor starting..."
log "INFO" "Gateway binary: $GATEWAY_BIN"
log "INFO" "Configuration: $GATEWAY_CONFIG"
log "INFO" "Max restarts: $MAX_RESTARTS in $RESTART_WINDOW seconds"

while true; do
    # Clean old restart times outside the window
    current_time=$(date +%s)
    new_restart_times=()
    for time in "${restart_times[@]:-}"; do
        if (( current_time - time < RESTART_WINDOW )); then
            new_restart_times+=("$time")
        fi
    done
    restart_times=("${new_restart_times[@]}")

    # Check restart limit
    if (( ${#restart_times[@]} >= MAX_RESTARTS )); then
        log "ERROR" "Gateway crashed $MAX_RESTARTS times in $RESTART_WINDOW seconds. Exiting."
        exit 1
    fi

    # Start gateway
    log "INFO" "Starting gateway (attempt $((${#restart_times[@]} + 1))/$MAX_RESTARTS)..."
    
    # Set environment variables for better stability
    export RUST_BACKTRACE=1
    export RUST_LOG="${RUST_LOG:-info,fechatter_gateway=debug}"
    
    # Start the gateway in background
    "$GATEWAY_BIN" --config "$GATEWAY_CONFIG" &
    GATEWAY_PID=$!
    
    # Record restart time
    restart_times+=("$current_time")
    
    log "INFO" "Gateway started with PID: $GATEWAY_PID"
    
    # Wait for gateway to exit
    wait_result=0
    wait "$GATEWAY_PID" || wait_result=$?
    
    if [[ $wait_result -eq 0 ]]; then
        log "INFO" "Gateway exited normally"
        break
    else
        log "ERROR" "Gateway crashed with exit code: $wait_result"
        
        # Check if it was a signal
        if [[ $wait_result -gt 128 ]]; then
            signal=$((wait_result - 128))
            log "ERROR" "Gateway received signal: $signal"
        fi
    fi
    
    # Wait before restart
    log "INFO" "Waiting $RESTART_DELAY seconds before restart..."
    sleep "$RESTART_DELAY"
done

log "INFO" "Gateway Supervisor exiting"