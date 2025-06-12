#!/bin/bash

set -euo pipefail

echo "Starting minimal test..."

# Test 1: Simple file check
echo "Test 1: File exists"
if [[ -f docker/configs/chat.yml ]]; then
    echo "✓ chat.yml exists"
else
    echo "✗ chat.yml missing"
fi

# Test 2: YAML validation  
echo "Test 2: YAML validation"
if python3 -c "import yaml; yaml.safe_load(open('docker/configs/chat.yml'))" 2>/dev/null; then
    echo "✓ YAML valid"
else
    echo "✗ YAML invalid"
fi

# Test 3: Service name check
echo "Test 3: Docker service names"
# Exclude entire CORS section and then check for localhost
if sed '/cors:/,/allow_credentials:/d' docker/configs/chat.yml | grep -q 'localhost\|127.0.0.1'; then
    echo "✗ Contains localhost (excluding CORS section)"
else
    echo "✓ Uses Docker service names"
fi

echo "Minimal test completed successfully" 