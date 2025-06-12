#!/bin/bash
# Fix all identified root cause dependency issues

set -e

echo "🔧 Fixing root cause dependency issues..."

# Root Cause 1: Rust version compatibility
echo "📦 Fixing Rust version compatibility issues..."
if cargo tree | grep -q "backtrace.*0.3.75"; then
    echo "   → Downgrading backtrace to compatible version"
    cargo update backtrace --precise 0.3.74
fi

# Root Cause 2: OpenSSL vendored feature missing
echo "🔐 Ensuring OpenSSL vendored feature is enabled..."
if ! grep -q 'openssl.*features.*vendored' Cargo.toml; then
    echo "   → Adding OpenSSL vendored feature to Cargo.toml"
    # Create backup
    cp Cargo.toml Cargo.toml.backup
    
    # Add or modify openssl dependency
    if grep -q '^openssl = ' Cargo.toml; then
        sed -i 's/^openssl = .*/openssl = { version = "*", features = ["vendored"] }/' Cargo.toml
    else
        echo -e '\n[workspace.dependencies]' >> Cargo.toml
        echo 'openssl = { version = "*", features = ["vendored"] }' >> Cargo.toml
    fi
    
    echo "   ✅ OpenSSL vendored feature added"
else
    echo "   ✅ OpenSSL vendored feature already configured"
fi

# Root Cause 3: Update potentially problematic dependencies
echo "📦 Updating potentially problematic dependencies..."
cargo update

# Root Cause 4: Verify protobuf files
echo "🔍 Checking protobuf configuration..."
if [ -d "fechatter_protos" ]; then
    echo "   ✅ Protobuf files directory found"
    if ls fechatter_protos/*.proto >/dev/null 2>&1; then
        echo "   ✅ Proto files found: $(ls fechatter_protos/*.proto | wc -l) files"
    else
        echo "   ⚠️  No .proto files found in fechatter_protos/"
    fi
else
    echo "   ⚠️  fechatter_protos directory not found"
fi

# Root Cause 5: Check for Edition 2024 issues
echo "🔍 Checking for Edition 2024 compatibility..."
if grep -r "edition.*2024" */Cargo.toml 2>/dev/null; then
    echo "   ⚠️  Found Edition 2024 usage, might need Rust nightly"
    echo "   💡 Consider changing to edition = \"2021\" for stability"
fi

echo "✅ Root cause dependency fixes completed!"
echo ""
echo "🏗️  Ready to build with:"
echo "   ./scripts/build-arm64.sh    # For ARM64 (Apple Silicon)"
echo "   ./scripts/deploy-podman.sh build    # For general deployment"
echo ""
echo "💡 If issues persist:"
echo "   1. Check that .sqlx/ directory exists for offline mode"
echo "   2. Ensure all protobuf files are present"
echo "   3. Verify Docker/Podman has sufficient memory (>4GB recommended)" 