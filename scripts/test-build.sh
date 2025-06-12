#!/bin/bash
# Quick test build to verify root cause fixes

set -e

echo "🧪 Testing root cause fixes..."

# Test 1: Check Cargo.toml syntax
echo "1️⃣ Testing Cargo.toml syntax..."
if cargo check --workspace --quiet --offline 2>/dev/null; then
    echo "   ✅ Cargo.toml syntax is valid (offline mode)"
elif cargo check --workspace --quiet 2>/dev/null; then
    echo "   ✅ Cargo.toml syntax is valid (online mode)"
else
    echo "   ❌ Cargo.toml syntax errors detected"
    echo "   → Running detailed check..."
    cargo check --workspace 2>&1 | head -10
fi

# Test 2: Check backtrace version
echo "2️⃣ Checking backtrace version compatibility..."
if cargo tree | grep -q "backtrace.*0.3.74"; then
    echo "   ✅ backtrace downgraded to compatible version (0.3.74)"
elif cargo tree | grep -q "backtrace.*0.3.75"; then
    echo "   ⚠️  backtrace still at incompatible version 0.3.75"
else
    echo "   ✅ backtrace version looks good"
fi

# Test 3: Check OpenSSL vendored feature
echo "3️⃣ Checking OpenSSL vendored configuration..."
if grep -q 'openssl.*features.*vendored' Cargo.toml; then
    echo "   ✅ OpenSSL vendored feature configured"
    grep -A1 -B1 "openssl.*vendored" Cargo.toml | head -3
else
    echo "   ❌ OpenSSL vendored feature missing"
fi

# Test 4: Check protobuf files (corrected path)
echo "4️⃣ Checking protobuf files..."
if [ -d "fechatter_protos/fechatter/v1" ] && ls fechatter_protos/fechatter/v1/*.proto >/dev/null 2>&1; then
    PROTO_COUNT=$(ls fechatter_protos/fechatter/v1/*.proto | wc -l)
    echo "   ✅ Protobuf files found: $PROTO_COUNT files in fechatter/v1/"
    echo "   → Files: $(ls fechatter_protos/fechatter/v1/*.proto | xargs -n1 basename | tr '\n' ' ')"
else
    echo "   ⚠️  Protobuf files missing or incomplete"
    echo "   → Checking directory structure..."
    ls -la fechatter_protos/ 2>/dev/null | head -5 || echo "fechatter_protos not found"
fi

# Test 5: Quick dependency resolution (with longer timeout)
echo "5️⃣ Testing dependency resolution..."
if timeout 60 cargo fetch --quiet 2>/dev/null; then
    echo "   ✅ Dependencies can be fetched successfully"
elif cargo fetch --quiet --offline 2>/dev/null; then
    echo "   ✅ Dependencies available offline"
else
    echo "   ⚠️  Dependency fetch failed - checking network/cache issues"
    echo "   → Try: cargo clean && cargo fetch"
fi

# Test 6: Check for Edition 2024 issues
echo "6️⃣ Checking for Edition compatibility..."
if grep -r "edition.*2024" */Cargo.toml 2>/dev/null; then
    echo "   ⚠️  Found Edition 2024 usage, might need Rust nightly"
elif grep -r "edition.*2021" */Cargo.toml 2>/dev/null | head -1; then
    echo "   ✅ Using stable Edition 2021"
else
    echo "   ✅ No edition issues detected"
fi

echo ""
echo "🎯 Root cause analysis complete!"
echo ""
echo "🏗️  Next steps:"
echo "   🐳 Test Docker build: podman build --platform linux/arm64 -t fechatter:test ."
echo "   📦 Test local build:   cargo build --release --bin fechatter_server"
echo "   🔧 Clean and retry:    cargo clean && cargo build"
echo ""
echo "💡 If Docker build fails:"
echo "   • Ensure sufficient memory (4GB+ recommended)"
echo "   • Check protobuf include paths in build output"
echo "   • Verify OpenSSL vendored compilation is working" 