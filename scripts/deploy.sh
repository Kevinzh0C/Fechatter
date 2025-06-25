#!/bin/bash
# deploy.sh - Unified Fechatter Deployment Script
# Supports multiple cloud platforms and environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Show help
show_help() {
    cat << EOF
🚀 Fechatter Unified Deployment Script

Usage: $0 <platform> [options]

Platforms:
  fly         Deploy to Fly.io (recommended for demos)
  aws         Deploy to AWS EKS
  azure       Deploy to Azure AKS  
  gcp         Deploy to Google Cloud GKE
  japan       Deploy to AWS Tokyo (optimized for 1000 users)
  local       Start local development environment

Options:
  --env <env>         Environment (dev/staging/prod)
  --region <region>   Cloud region
  --cluster <name>    Kubernetes cluster name
  --domain <domain>   Custom domain name
  --monitoring        Enable monitoring setup
  --help             Show this help

Examples:
  $0 fly                           # Deploy demo to Fly.io
  $0 aws --env prod --monitoring   # Production AWS deployment
  $0 japan --domain mydomain.com  # Japan region with custom domain
  $0 local                         # Start local development

Environment Variables:
  JWT_SECRET           JWT signing secret
  OPENAI_API_KEY      OpenAI API key (optional)
  REDIS_PASSWORD      Redis password
  MEILI_MASTER_KEY    Meilisearch master key
EOF
}

# Parse arguments
PLATFORM=""
ENVIRONMENT="dev"
REGION=""
CLUSTER_NAME=""
DOMAIN=""
ENABLE_MONITORING=false

while [[ $# -gt 0 ]]; do
    case $1 in
        fly|aws|azure|gcp|japan|local)
            PLATFORM="$1"
            shift
            ;;
        --env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --region)
            REGION="$2"
            shift 2
            ;;
        --cluster)
            CLUSTER_NAME="$2"
            shift 2
            ;;
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --monitoring)
            ENABLE_MONITORING=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate platform
if [[ -z "$PLATFORM" ]]; then
    print_error "Please specify a platform"
    show_help
    exit 1
fi

# Deploy to Fly.io
deploy_fly() {
    print_step "Deploying to Fly.io..."
    
    # Check flyctl
    if ! command -v flyctl &> /dev/null; then
        print_error "flyctl not installed. Install: https://fly.io/docs/hands-on/install-flyctl/"
        exit 1
    fi
    
    # Check login
    if ! flyctl auth whoami &> /dev/null; then
        print_error "Not logged in to Fly.io. Run: flyctl auth login"
        exit 1
    fi
    
    local app_name="fechatter-${ENVIRONMENT}"
    local region="${REGION:-nrt}" # Tokyo by default
    
    print_info "App: $app_name, Region: $region"
    
    # Create app if not exists
    if ! flyctl apps list | grep -q "$app_name"; then
        print_info "Creating app $app_name..."
        flyctl apps create "$app_name" --region "$region"
    fi
    
    # Set secrets
    print_info "Setting secrets..."
    flyctl secrets set \
        JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}" \
        REDIS_PASSWORD="${REDIS_PASSWORD:-$(openssl rand -base64 16)}" \
        MEILI_MASTER_KEY="${MEILI_MASTER_KEY:-$(openssl rand -base64 32)}" \
        ENVIRONMENT="$ENVIRONMENT" \
        --app "$app_name"
    
    if [[ -n "$OPENAI_API_KEY" ]]; then
        flyctl secrets set OPENAI_API_KEY="$OPENAI_API_KEY" --app "$app_name"
    fi
    
    # Create volume if needed
    if ! flyctl volumes list --app "$app_name" | grep -q "fechatter_data"; then
        print_info "Creating data volume..."
        flyctl volumes create fechatter_data --size 10 --region "$region" --app "$app_name"
    fi
    
    # Deploy
    print_info "Deploying application..."
    flyctl deploy --app "$app_name"
    
    # Get URL
    local url="https://$app_name.fly.dev"
    print_info "Deployment complete! URL: $url"
    
    # Health check
    sleep 10
    if curl -s "$url/health" | grep -q "ok"; then
        print_info "✅ Health check passed!"
    else
        print_warn "⚠️ Health check failed, check logs: flyctl logs --app $app_name"
    fi
}

# Deploy to AWS
deploy_aws() {
    print_step "Deploying to AWS EKS..."
    
    local aws_region="${REGION:-us-west-2}"
    local cluster_name="${CLUSTER_NAME:-fechatter-cluster}"
    
    # Check dependencies
    for cmd in aws kubectl helm; do
        if ! command -v $cmd &> /dev/null; then
            print_error "$cmd not installed"
            exit 1
        fi
    done
    
    # Check AWS auth
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS authentication failed"
        exit 1
    fi
    
    print_info "Region: $aws_region, Cluster: $cluster_name"
    
    # Update kubeconfig
    aws eks update-kubeconfig --region "$aws_region" --name "$cluster_name"
    
    # Deploy using existing AWS script
    AWS_REGION="$aws_region" CLUSTER_NAME="$cluster_name" DOMAIN="$DOMAIN" \
    SETUP_MONITORING="$ENABLE_MONITORING" ./scripts/deploy-aws.sh
}

# Deploy to Azure
deploy_azure() {
    print_step "Deploying to Azure AKS..."
    
    local azure_location="${REGION:-eastus}"
    local cluster_name="${CLUSTER_NAME:-fechatter-cluster}"
    
    # Use existing Azure script
    AZURE_LOCATION="$azure_location" CLUSTER_NAME="$cluster_name" \
    SETUP_MONITORING="$ENABLE_MONITORING" ./scripts/deploy-azure.sh
}

# Deploy to GCP
deploy_gcp() {
    print_step "Deploying to Google Cloud GKE..."
    
    local gcp_region="${REGION:-us-central1}"
    local cluster_name="${CLUSTER_NAME:-fechatter-cluster}"
    
    # Use existing GCP script
    GCP_REGION="$gcp_region" CLUSTER_NAME="$cluster_name" \
    SETUP_MONITORING="$ENABLE_MONITORING" ./scripts/deploy-gcp.sh
}

# Deploy to Japan (AWS Tokyo optimized)
deploy_japan() {
    print_step "Deploying to Japan (AWS Tokyo)..."
    
    # Use existing Japan script
    DOMAIN="$DOMAIN" SETUP_MONITORING="$ENABLE_MONITORING" ./scripts/deploy-japan.sh
}

# Start local development
deploy_local() {
    print_step "Starting local development environment..."
    
    # Check if in project root
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Please run from project root directory"
        exit 1
    fi
    
    # Start infrastructure
    print_info "Starting infrastructure services..."
    ./scripts/cloud-detect.sh
    
    # Wait for services
    print_info "Waiting for services..."
    ./scripts/wait-for-services.sh echo "Services ready"
    
    # Start application
    print_info "Starting application services..."
    ./scripts/start-dev.sh
}

# Main execution
main() {
    print_info "🚀 Fechatter Deployment Script"
    print_info "Platform: $PLATFORM | Environment: $ENVIRONMENT"
    
    case $PLATFORM in
        fly)
            deploy_fly
            ;;
        aws)
            deploy_aws
            ;;
        azure)
            deploy_azure
            ;;
        gcp)
            deploy_gcp
            ;;
        japan)
            deploy_japan
            ;;
        local)
            deploy_local
            ;;
        *)
            print_error "Unsupported platform: $PLATFORM"
            exit 1
            ;;
    esac
    
    print_info "🎉 Deployment completed!"
}

# Run main function
main "$@" 