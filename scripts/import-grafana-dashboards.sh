#!/bin/bash

# Import Grafana Dashboards for Fechatter
# This script imports pre-configured dashboards to Grafana

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
GRAFANA_URL=${GRAFANA_URL:-""}
GRAFANA_API_KEY=${GRAFANA_API_KEY:-""}
DASHBOARD_DIR="grafana-dashboards"

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if [ -z "$GRAFANA_URL" ]; then
        print_error "GRAFANA_URL environment variable is required"
        echo "Example: export GRAFANA_URL=https://your-org.grafana.net"
        exit 1
    fi
    
    if [ -z "$GRAFANA_API_KEY" ]; then
        print_error "GRAFANA_API_KEY environment variable is required"
        echo "Generate an API key from your Grafana instance:"
        echo "  1. Go to Configuration > API Keys"
        echo "  2. Create a new key with Editor role"
        echo "  3. export GRAFANA_API_KEY=your_api_key"
        exit 1
    fi
    
    if [ ! -d "$DASHBOARD_DIR" ]; then
        print_error "Dashboard directory '$DASHBOARD_DIR' not found"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        print_warning "jq not found. JSON responses will not be formatted."
    fi
    
    print_success "Prerequisites check passed"
}

# Test Grafana connection
test_connection() {
    print_status "Testing Grafana connection..."
    
    response=$(curl -s -w "%{http_code}" -o /dev/null \
        -H "Authorization: Bearer $GRAFANA_API_KEY" \
        "$GRAFANA_URL/api/org")
    
    if [ "$response" = "200" ]; then
        print_success "Successfully connected to Grafana"
    else
        print_error "Failed to connect to Grafana (HTTP $response)"
        print_error "Please check your GRAFANA_URL and GRAFANA_API_KEY"
        exit 1
    fi
}

# Import a single dashboard
import_dashboard() {
    local dashboard_file="$1"
    local dashboard_name=$(basename "$dashboard_file" .json)
    
    print_status "Importing dashboard: $dashboard_name"
    
    # Prepare the dashboard JSON for import
    local dashboard_json=$(cat "$dashboard_file")
    local import_payload=$(echo "$dashboard_json" | jq '{
        dashboard: .,
        overwrite: true,
        inputs: [
            {
                name: "DS_PROMETHEUS",
                type: "datasource",
                pluginId: "prometheus",
                value: "Prometheus"
            }
        ]
    }')
    
    # Import the dashboard
    local response=$(curl -s -w "%{http_code}" \
        -X POST \
        -H "Authorization: Bearer $GRAFANA_API_KEY" \
        -H "Content-Type: application/json" \
        -d "$import_payload" \
        "$GRAFANA_URL/api/dashboards/import")
    
    local http_code="${response: -3}"
    local response_body="${response%???}"
    
    if [ "$http_code" = "200" ]; then
        print_success "Dashboard '$dashboard_name' imported successfully"
        
        # Extract dashboard URL if jq is available
        if command -v jq &> /dev/null; then
            local dashboard_url=$(echo "$response_body" | jq -r '.dashboardUrl // empty')
            if [ -n "$dashboard_url" ]; then
                echo "  📊 Dashboard URL: $GRAFANA_URL$dashboard_url"
            fi
        fi
    else
        print_error "Failed to import dashboard '$dashboard_name' (HTTP $http_code)"
        if command -v jq &> /dev/null && echo "$response_body" | jq . &> /dev/null; then
            echo "$response_body" | jq .
        else
            echo "$response_body"
        fi
    fi
}

# Import all dashboards
import_all_dashboards() {
    print_status "Importing all dashboards from $DASHBOARD_DIR..."
    
    local imported_count=0
    local failed_count=0
    
    for dashboard_file in "$DASHBOARD_DIR"/*.json; do
        if [ -f "$dashboard_file" ]; then
            if import_dashboard "$dashboard_file"; then
                ((imported_count++))
            else
                ((failed_count++))
            fi
            echo ""
        fi
    done
    
    echo "📊 Import Summary:"
    echo "  ✅ Successfully imported: $imported_count"
    echo "  ❌ Failed to import: $failed_count"
    echo ""
}

# Create datasource if it doesn't exist
create_prometheus_datasource() {
    print_status "Checking Prometheus datasource..."
    
    # Check if Prometheus datasource exists
    local response=$(curl -s -w "%{http_code}" \
        -H "Authorization: Bearer $GRAFANA_API_KEY" \
        "$GRAFANA_URL/api/datasources/name/Prometheus")
    
    local http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        print_success "Prometheus datasource already exists"
        return 0
    fi
    
    print_status "Creating Prometheus datasource..."
    
    # Create Prometheus datasource
    local datasource_config='{
        "name": "Prometheus",
        "type": "prometheus",
        "url": "https://fechatter-monitoring.fly.dev",
        "access": "proxy",
        "isDefault": true,
        "basicAuth": false
    }'
    
    local response=$(curl -s -w "%{http_code}" \
        -X POST \
        -H "Authorization: Bearer $GRAFANA_API_KEY" \
        -H "Content-Type: application/json" \
        -d "$datasource_config" \
        "$GRAFANA_URL/api/datasources")
    
    local http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        print_success "Prometheus datasource created successfully"
    else
        print_warning "Failed to create Prometheus datasource (HTTP $http_code)"
        print_warning "You may need to create it manually in Grafana"
    fi
}

# Main execution
main() {
    echo "📊 Grafana Dashboard Import Tool"
    echo "================================"
    echo ""
    echo "Grafana URL: $GRAFANA_URL"
    echo "Dashboard Directory: $DASHBOARD_DIR"
    echo ""
    
    check_prerequisites
    test_connection
    create_prometheus_datasource
    import_all_dashboards
    
    echo "🎉 Dashboard import completed!"
    echo ""
    echo "🔗 Access your dashboards at: $GRAFANA_URL/dashboards"
    echo ""
}

# Show usage if no arguments and no environment variables
if [ $# -eq 0 ] && [ -z "$GRAFANA_URL" ] && [ -z "$GRAFANA_API_KEY" ]; then
    echo "Usage: $0"
    echo ""
    echo "Environment variables required:"
    echo "  GRAFANA_URL     - Your Grafana instance URL"
    echo "  GRAFANA_API_KEY - Your Grafana API key"
    echo ""
    echo "Example:"
    echo "  export GRAFANA_URL=https://your-org.grafana.net"
    echo "  export GRAFANA_API_KEY=your_api_key"
    echo "  $0"
    exit 1
fi

# Run main function
main "$@"