#!/bin/bash

# Fechatter Fly.io Deployment with Monitoring
# This script deploys all Fechatter services with monitoring enabled

set -e

echo "🚀 Starting Fechatter deployment with monitoring..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v flyctl &> /dev/null; then
        print_error "flyctl CLI not found. Please install it first."
        exit 1
    fi
    
    if ! flyctl auth whoami &> /dev/null; then
        print_error "Not logged into Fly.io. Please run 'flyctl auth login' first."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Deploy monitoring infrastructure first
deploy_monitoring() {
    print_status "Deploying Prometheus monitoring service..."
    
    # Create prometheus data volume if it doesn't exist
    if ! flyctl volumes list -a fechatter-monitoring | grep -q prometheus_data; then
        print_status "Creating Prometheus data volume..."
        flyctl volumes create prometheus_data --region nrt --size 10 -a fechatter-monitoring
    fi
    
    # Deploy monitoring service
    flyctl deploy --config fly.monitoring.toml --dockerfile Dockerfile.monitoring
    
    if [ $? -eq 0 ]; then
        print_success "Monitoring service deployed successfully"
    else
        print_error "Failed to deploy monitoring service"
        exit 1
    fi
}

# Deploy main application services
deploy_services() {
    print_status "Deploying Fechatter services..."
    
    # Deploy main application
    print_status "Deploying fechatter_server..."
    flyctl deploy --dockerfile docker/Dockerfile.fly
    
    # Set required environment variables for metrics
    print_status "Setting environment variables for metrics..."
    flyctl secrets set METRICS_ENABLED=true METRICS_PORT=9090 -a fechatter
    
    print_success "All services deployed successfully"
}

# Configure Grafana Cloud (if credentials provided)
setup_grafana_cloud() {
    if [ -n "$GRAFANA_PROMETHEUS_URL" ] && [ -n "$GRAFANA_PROMETHEUS_USER" ] && [ -n "$GRAFANA_PROMETHEUS_API_KEY" ]; then
        print_status "Configuring Grafana Cloud integration..."
        
        flyctl secrets set \
            GRAFANA_PROMETHEUS_URL="$GRAFANA_PROMETHEUS_URL" \
            GRAFANA_PROMETHEUS_USER="$GRAFANA_PROMETHEUS_USER" \
            GRAFANA_PROMETHEUS_API_KEY="$GRAFANA_PROMETHEUS_API_KEY" \
            -a fechatter-monitoring
        
        print_success "Grafana Cloud integration configured"
    else
        print_warning "Grafana Cloud credentials not provided. Skipping integration."
        print_warning "To enable Grafana Cloud, set: GRAFANA_PROMETHEUS_URL, GRAFANA_PROMETHEUS_USER, GRAFANA_PROMETHEUS_API_KEY"
    fi
}

# Import Grafana dashboards (if credentials provided)
import_dashboards() {
    if [ -n "$GRAFANA_URL" ] && [ -n "$GRAFANA_API_KEY" ]; then
        print_status "Importing Grafana dashboards..."
        
        # Import the overview dashboard
        curl -X POST \
            -H "Authorization: Bearer $GRAFANA_API_KEY" \
            -H "Content-Type: application/json" \
            -d @grafana-dashboards/fechatter-overview.json \
            "$GRAFANA_URL/api/dashboards/db"
        
        if [ $? -eq 0 ]; then
            print_success "Dashboards imported successfully"
        else
            print_warning "Failed to import dashboards. You can import them manually."
        fi
    else
        print_warning "Grafana credentials not provided. Dashboards not imported."
        print_warning "To import dashboards automatically, set: GRAFANA_URL, GRAFANA_API_KEY"
    fi
}

# Verify deployment
verify_deployment() {
    print_status "Verifying deployment..."
    
    # Check main application health
    if curl -f https://fechatter.fly.dev/health &> /dev/null; then
        print_success "Main application is healthy"
    else
        print_warning "Main application health check failed"
    fi
    
    # Check monitoring service
    if curl -f https://fechatter-monitoring.fly.dev/-/healthy &> /dev/null; then
        print_success "Monitoring service is healthy"
    else
        print_warning "Monitoring service health check failed"
    fi
    
    print_status "Deployment verification complete"
}

# Print access information
print_access_info() {
    echo ""
    echo "🎉 Deployment completed!"
    echo ""
    echo "📊 Access Information:"
    echo "  Main Application: https://fechatter.fly.dev"
    echo "  Prometheus:       https://fechatter-monitoring.fly.dev"
    echo ""
    echo "📈 Metrics Endpoints:"
    echo "  fechatter_server: https://fechatter.fly.dev/metrics"
    echo "  notify_server:    https://fechatter.fly.dev:9091/metrics"
    echo "  bot_server:       https://fechatter.fly.dev:9092/metrics"
    echo "  analytics_server: https://fechatter.fly.dev:7778/metrics"
    echo ""
    
    if [ -n "$GRAFANA_URL" ]; then
        echo "🎛️  Grafana Dashboard: $GRAFANA_URL"
    else
        echo "🎛️  Import grafana-dashboards/fechatter-overview.json to your Grafana instance"
    fi
    echo ""
}

# Main execution
main() {
    echo "🚀 Fechatter Deployment with Monitoring"
    echo "======================================="
    echo ""
    
    check_prerequisites
    deploy_monitoring
    deploy_services
    setup_grafana_cloud
    import_dashboards
    verify_deployment
    print_access_info
}

# Run main function
main "$@"