#!/bin/bash
# prepare-configs.sh - Prepare configuration files from templates

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration directory
CONFIG_TEMPLATE_DIR="fly/config"
CONFIG_OUTPUT_DIR="fly/config-processed"

# Create output directory
mkdir -p "$CONFIG_OUTPUT_DIR"

# Function to substitute environment variables in config files
substitute_env_vars() {
    local input_file=$1
    local output_file=$2
    
    print_info "Processing $input_file..."
    
    # Create a temporary file
    local temp_file=$(mktemp)
    
    # Copy original file
    cp "$input_file" "$temp_file"
    
    # List of environment variables to substitute
    local env_vars=(
        "DATABASE_URL"
        "REDIS_URL"
        "NATS_URL"
        "JWT_SECRET"
        "MEILISEARCH_URL"
        "MEILISEARCH_KEY"
        "CLICKHOUSE_HOST"
        "CLICKHOUSE_PORT"
        "CLICKHOUSE_USER"
        "CLICKHOUSE_PASSWORD"
        "OPENAI_API_KEY"
    )
    
    # Perform substitution
    for var in "${env_vars[@]}"; do
        if [ -n "${!var}" ]; then
            # Use sed to replace ${VAR} with actual value
            sed -i "s|\${$var}|${!var}|g" "$temp_file"
            print_info "  Substituted $var"
        else
            print_warn "  Variable $var not set"
        fi
    done
    
    # Move processed file to output
    mv "$temp_file" "$output_file"
    print_info "  Created $output_file"
}

# Process all YAML files
for config_file in "$CONFIG_TEMPLATE_DIR"/*.yml; do
    if [ -f "$config_file" ]; then
        filename=$(basename "$config_file")
        substitute_env_vars "$config_file" "$CONFIG_OUTPUT_DIR/$filename"
    fi
done

print_info "Configuration preparation complete!"
print_info "Processed configs are in: $CONFIG_OUTPUT_DIR"

# Create a script to set these as fly secrets
cat > fly/set-secrets.sh << 'EOF'
#!/bin/bash
# set-secrets.sh - Set Fly.io secrets for configuration

echo "Setting Fly.io secrets..."

# Read each processed config file and set as secret
for config_file in fly/config-processed/*.yml; do
    if [ -f "$config_file" ]; then
        filename=$(basename "$config_file" .yml)
        secret_name="CONFIG_$(echo $filename | tr '[:lower:]' '[:upper:]')_YML"
        
        echo "Setting $secret_name from $config_file"
        flyctl secrets set "$secret_name=$(cat $config_file)" --app fechatter-demo
    fi
done

# Set other required secrets
flyctl secrets set \
    DATABASE_URL="$DATABASE_URL" \
    REDIS_URL="$REDIS_URL" \
    NATS_URL="$NATS_URL" \
    JWT_SECRET="$JWT_SECRET" \
    MEILISEARCH_URL="$MEILISEARCH_URL" \
    MEILISEARCH_KEY="$MEILISEARCH_KEY" \
    CLICKHOUSE_HOST="$CLICKHOUSE_HOST" \
    CLICKHOUSE_PORT="$CLICKHOUSE_PORT" \
    CLICKHOUSE_USER="$CLICKHOUSE_USER" \
    CLICKHOUSE_PASSWORD="$CLICKHOUSE_PASSWORD" \
    OPENAI_API_KEY="$OPENAI_API_KEY" \
    --app fechatter-demo

echo "Secrets configured!"
EOF

chmod +x fly/set-secrets.sh

print_info "Created fly/set-secrets.sh for setting Fly.io secrets" 