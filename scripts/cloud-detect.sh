#!/bin/bash
# cloud-detect.sh - 检测运行环境并选择合适的脚本

set -e

echo "🔍 检测运行环境..."

# 检测是否在 Kubernetes 环境
if [ -f /var/run/secrets/kubernetes.io/serviceaccount/token ]; then
    echo "✅ 检测到 Kubernetes 环境"
    export RUNTIME_ENV="kubernetes"
    export NAMESPACE=${POD_NAMESPACE:-default}
    
elif [ -n "$DOCKER_HOST" ] || [ -S /var/run/docker.sock ]; then
    echo "✅ 检测到 Docker 环境"
    export RUNTIME_ENV="docker"
    
elif command -v podman >/dev/null 2>&1; then
    echo "✅ 检测到 Podman 环境"
    export RUNTIME_ENV="podman"
    
else
    echo "✅ 检测到本地环境"
    export RUNTIME_ENV="local"
fi

# 检测云平台
if [ -n "$AWS_REGION" ] || curl -s -m 2 http://169.254.169.254/latest/meta-data/ >/dev/null 2>&1; then
    echo "🌐 检测到 AWS 环境"
    export CLOUD_PROVIDER="aws"
    
elif [ -n "$AZURE_CLIENT_ID" ] || curl -s -m 2 -H "Metadata:true" "http://169.254.169.254/metadata/instance" >/dev/null 2>&1; then
    echo "🌐 检测到 Azure 环境"
    export CLOUD_PROVIDER="azure"
    
elif [ -n "$GOOGLE_CLOUD_PROJECT" ] || curl -s -m 2 -H "Metadata-Flavor: Google" "http://metadata.google.internal/computeMetadata/v1/instance/" >/dev/null 2>&1; then
    echo "🌐 检测到 Google Cloud 环境"
    export CLOUD_PROVIDER="gcp"
    
else
    echo "🌐 未检测到特定云平台"
    export CLOUD_PROVIDER="generic"
fi

echo "📊 环境信息:"
echo "  运行时: $RUNTIME_ENV"
echo "  云平台: $CLOUD_PROVIDER"

# 根据环境选择合适的脚本
case $RUNTIME_ENV in
    "kubernetes")
        exec ./scripts/wait-for-services-k8s.sh "$@"
        ;;
    "docker"|"podman")
        exec ./scripts/wait-for-services.sh "$@"
        ;;
    "local")
        exec ./scripts/wait-for-services-local.sh "$@"
        ;;
    *)
        echo "❌ 未知环境，使用默认脚本"
        exec ./scripts/wait-for-services.sh "$@"
        ;;
esac