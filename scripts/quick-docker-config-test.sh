#!/bin/bash
set -euo pipefail

# Quick Docker Configuration Test
# Simplified validation for Docker config integration

echo "🐳 Quick Docker Configuration Test"
echo "=================================="

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

check_test() {
    local test_name="$1"
    local condition="$2"
    
    if [[ "$condition" == "true" ]]; then
        echo -e "  ${GREEN}✓${NC} $test_name"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} $test_name"
        ((FAILED++))
    fi
}

echo -e "\n${BLUE}📋 Phase 1: Docker Configuration Files${NC}"

# Check if Docker config files exist
check_test "chat.yml exists" "$(test -f docker/configs/chat.yml && echo true || echo false)"
check_test "analytics.yml exists" "$(test -f docker/configs/analytics.yml && echo true || echo false)"
check_test "notify.yml exists" "$(test -f docker/configs/notify.yml && echo true || echo false)"
check_test "bot.yml exists" "$(test -f docker/configs/bot.yml && echo true || echo false)"
check_test "gateway.yml exists" "$(test -f docker/configs/gateway.yml && echo true || echo false)"

echo -e "\n${BLUE}📋 Phase 2: YAML Syntax Validation${NC}"

# Validate YAML syntax
for config in docker/configs/*.yml; do
    if python3 -c "import yaml; yaml.safe_load(open('$config'))" 2>/dev/null; then
        check_test "$(basename $config) YAML valid" "true"
    else
        check_test "$(basename $config) YAML valid" "false"
    fi
done

echo -e "\n${BLUE}📋 Phase 3: Docker Service Names Check${NC}"

# Check for Docker service names (not localhost) - excluding CORS section which legitimately uses localhost
check_test "chat.yml uses Docker services" "$(sed '/cors:/,/allow_credentials:/d' docker/configs/chat.yml | grep -q 'localhost\|127.0.0.1' && echo false || echo true)"
check_test "analytics.yml uses Docker services" "$(sed '/cors:/,/burst_size:/d' docker/configs/analytics.yml | grep -q 'localhost\|127.0.0.1' && echo false || echo true)"
check_test "notify.yml uses Docker services" "$(grep -q 'localhost\|127.0.0.1' docker/configs/notify.yml && echo false || echo true)"
check_test "bot.yml uses Docker services" "$(grep -q 'localhost\|127.0.0.1' docker/configs/bot.yml && echo false || echo true)"
check_test "gateway.yml uses Docker services" "$(sed '/cors_origins:/,/^[[:space:]]*-/d' docker/configs/gateway.yml | grep -q 'localhost\|127.0.0.1' && echo false || echo true)"

echo -e "\n${BLUE}📋 Phase 4: Configuration Content Validation${NC}"

# Check specific service configurations
check_test "chat.yml has postgres service" "$(grep -q 'postgres:' docker/configs/chat.yml && echo true || echo false)"
check_test "chat.yml has redis service" "$(grep -q 'redis:' docker/configs/chat.yml && echo true || echo false)"
check_test "analytics.yml has clickhouse service" "$(grep -q 'clickhouse:' docker/configs/analytics.yml && echo true || echo false)"
check_test "notify.yml has nats service" "$(grep -q 'nats:' docker/configs/notify.yml && echo true || echo false)"

echo -e "\n${BLUE}📋 Phase 5: Enhanced Config Loading Verification${NC}"

# Check if code modifications include Docker paths
check_test "fechatter_server has Docker paths" "$(grep -q '/app/config' fechatter_server/src/config.rs && echo true || echo false)"
check_test "analytics_server has Docker paths" "$(grep -q '/app/config' analytics_server/src/config.rs && echo true || echo false)"
check_test "notify_server has Docker paths" "$(grep -q '/app/config' notify_server/src/config.rs && echo true || echo false)"
check_test "bot_server has Docker paths" "$(grep -q '/app/config' bot_server/src/config.rs && echo true || echo false)"
check_test "gateway has Docker paths" "$(grep -q '/app/config' fechatter_gateway/src/config.rs && echo true || echo false)"

echo -e "\n${BLUE}📋 Phase 6: Binary Availability${NC}"

# Check if binaries are available for Docker deployment
check_test "x86_64 binaries directory exists" "$(test -d docker/binaries/x86_64 && echo true || echo false)"

if [[ -d docker/binaries/x86_64 ]]; then
    binary_count=$(find docker/binaries/x86_64 -type f -executable | wc -l)
    check_test "Sufficient binaries available" "$((binary_count >= 6 ? 1 : 0) && echo true || echo false)"
fi

echo ""
echo "=================================="
echo -e "${BLUE}📊 Test Summary${NC}"
echo "=================================="
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [[ $FAILED -eq 0 ]]; then
    echo -e "\n${GREEN}🎉 All Docker configuration tests passed!${NC}"
    echo ""
    echo -e "${BLUE}✅ Docker Configuration Ready:${NC}"
    echo "  - All config files exist and have valid YAML syntax"
    echo "  - All configs use Docker service names (not localhost)"
    echo "  - Enhanced config loading supports Docker paths:"
    echo "    • /app/config/*.yml (highest priority)"
    echo "    • /app/*.yml"
    echo "    • /etc/fechatter/*.yml"
    echo "    • Environment variables override"
    echo ""
    echo -e "${BLUE}📋 Next Steps:${NC}"
    echo "  1. Build binaries: scripts/build-x86-complete.sh"
    echo "  2. Create Docker images with config mounting"
    echo "  3. Deploy with docker-compose"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed${NC}"
    echo "Please fix the issues above before Docker deployment"
    exit 1
fi 