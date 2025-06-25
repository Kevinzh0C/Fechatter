#!/bin/bash
# Start the Fechatter gateway with the updated configuration

cd /Users/zhangkaiqi/Rust/Fechatter

echo "🚀 Starting Fechatter Gateway on port 8080..."
echo "📋 Using configuration: fechatter_gateway/gateway.yml"
echo ""

# Build the gateway first
echo "🔨 Building gateway..."
cargo build --package fechatter_gateway --release

# Run the gateway
echo "🌟 Starting gateway..."
./target/release/fechatter_gateway --config fechatter_gateway/gateway.yml