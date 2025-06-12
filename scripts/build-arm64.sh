#!/bin/bash
# Build script specifically for ARM64 architecture

set -e

echo "🔨 Building Fechatter for ARM64 architecture..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
podman system prune -f || true

# Build with explicit platform
echo "🏗️ Building Docker image for ARM64..."
podman build \
    --platform linux/arm64 \
    --build-arg TARGETARCH=arm64 \
    -t fechatter:arm64 \
    -f Dockerfile \
    .

# Tag the image
echo "🏷️ Tagging image..."
podman tag fechatter:arm64 fechatter:latest

# Show build info
echo "✅ Build completed successfully!"
echo ""
echo "📦 Built images:"
podman images | grep fechatter

echo ""
echo "🚀 To run a service:"
echo "   podman run --rm -it fechatter:latest /usr/local/bin/fechatter_server"
echo ""
echo "💡 To extract binaries:"
echo "   podman run --rm -v ./output:/output fechatter:latest sh -c 'cp /usr/local/bin/* /output/'" 