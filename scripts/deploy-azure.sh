#!/bin/bash
# deploy-azure.sh - Azure AKS 部署脚本

set -e

echo "🚀 开始部署到 Azure AKS..."

# 配置变量
AZURE_RESOURCE_GROUP=${AZURE_RESOURCE_GROUP:-fechatter-rg}
AZURE_LOCATION=${AZURE_LOCATION:-eastus}
CLUSTER_NAME=${CLUSTER_NAME:-fechatter-cluster}
NAMESPACE=${NAMESPACE:-fechatter}
IMAGE_TAG=${IMAGE_TAG:-latest}

# 检查 Azure CLI
if ! command -v az &> /dev/null; then
    echo "❌ Azure CLI 未安装"
    exit 1
fi

# 检查 kubectl
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl 未安装"
    exit 1
fi

# 检查 Azure 认证
if ! az account show &> /dev/null; then
    echo "❌ Azure 认证失败，请运行 'az login'"
    exit 1
fi

echo "✅ Azure 认证成功"
echo "订阅: $(az account show --query name -o tsv)"
echo "位置: $AZURE_LOCATION"

# 获取 AKS 凭据
echo "🔧 获取 AKS 集群凭据..."
az aks get-credentials --resource-group "$AZURE_RESOURCE_GROUP" --name "$CLUSTER_NAME"

# 检查集群连接
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ 无法连接到 Kubernetes 集群"
    exit 1
fi

echo "✅ 已连接到 AKS 集群: $CLUSTER_NAME"

# 创建命名空间
echo "📦 创建命名空间..."
kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

# 创建密钥
echo "🔐 配置密钥..."
kubectl create secret generic fechatter-secrets \
    --from-literal=jwt-secret="${JWT_SECRET:-$(openssl rand -base64 32)}" \
    --from-literal=redis-password="${REDIS_PASSWORD:-$(openssl rand -base64 16)}" \
    --from-literal=meili-master-key="${MEILI_MASTER_KEY:-$(openssl rand -base64 32)}" \
    --namespace "$NAMESPACE" \
    --dry-run=client -o yaml | kubectl apply -f -

# 部署基础设施服务
echo "🏗️  部署基础设施服务..."

# PostgreSQL (使用 Azure Database 或部署到集群)
if [ "${USE_AZURE_DATABASE:-false}" = "true" ]; then
    echo "使用 Azure Database for PostgreSQL"
    
    # 创建 PostgreSQL 服务器
    SERVER_NAME="fechatter-postgres-$(date +%s)"
    
    az postgres server create \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --name "$SERVER_NAME" \
        --location "$AZURE_LOCATION" \
        --admin-user postgres \
        --admin-password "${POSTGRES_PASSWORD:-$(openssl rand -base64 16)}" \
        --sku-name GP_Gen5_2 \
        --version 15 || true
    
    # 创建数据库
    az postgres db create \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --server-name "$SERVER_NAME" \
        --name fechatter || true
    
    # 配置防火墙规则
    az postgres server firewall-rule create \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --server-name "$SERVER_NAME" \
        --name AllowAzureServices \
        --start-ip-address 0.0.0.0 \
        --end-ip-address 0.0.0.0 || true
else
    echo "部署 PostgreSQL 到集群"
    kubectl apply -f k8s/base/postgres.yaml -n "$NAMESPACE"
fi

# Redis (使用 Azure Cache for Redis 或部署到集群)
if [ "${USE_AZURE_REDIS:-false}" = "true" ]; then
    echo "使用 Azure Cache for Redis"
    
    # 创建 Redis 缓存
    REDIS_NAME="fechatter-redis-$(date +%s)"
    
    az redis create \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --name "$REDIS_NAME" \
        --location "$AZURE_LOCATION" \
        --sku Basic \
        --vm-size c0 || true
else
    echo "部署 Redis 到集群"
    kubectl apply -f k8s/base/redis.yaml -n "$NAMESPACE"
fi

# 其他基础设施服务
kubectl apply -f k8s/base/nats.yaml -n "$NAMESPACE"
kubectl apply -f k8s/base/meilisearch.yaml -n "$NAMESPACE"
kubectl apply -f k8s/base/clickhouse.yaml -n "$NAMESPACE"

# 等待基础设施服务就绪
echo "⏳ 等待基础设施服务就绪..."
kubectl wait --for=condition=available deployment/postgres -n "$NAMESPACE" --timeout=300s || true
kubectl wait --for=condition=available deployment/redis -n "$NAMESPACE" --timeout=300s || true
kubectl wait --for=condition=available deployment/nats -n "$NAMESPACE" --timeout=300s || true
kubectl wait --for=condition=available deployment/meilisearch -n "$NAMESPACE" --timeout=300s || true

# 配置容器镜像仓库
echo "🐳 配置 Azure Container Registry..."
ACR_NAME="${ACR_NAME:-fechatteracr}"

# 连接 ACR 到 AKS
az aks update \
    --name "$CLUSTER_NAME" \
    --resource-group "$AZURE_RESOURCE_GROUP" \
    --attach-acr "$ACR_NAME" || true

# 部署应用服务
echo "🚀 部署应用服务..."

# 更新镜像标签
sed -i.bak "s|:latest|:$IMAGE_TAG|g" k8s/base/*.yaml
sed -i.bak "s|fechatter/|$ACR_NAME.azurecr.io/fechatter/|g" k8s/base/*.yaml

kubectl apply -f k8s/base/fechatter-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/base/notify-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/base/bot-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/base/analytics-server.yaml -n "$NAMESPACE"

# 等待应用服务就绪
echo "⏳ 等待应用服务就绪..."
kubectl wait --for=condition=available deployment/fechatter-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/notify-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/bot-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/analytics-server -n "$NAMESPACE" --timeout=300s

# 部署网关
echo "🌐 部署 API 网关..."
kubectl apply -f k8s/base/gateway.yaml -n "$NAMESPACE"
kubectl wait --for=condition=available deployment/gateway -n "$NAMESPACE" --timeout=300s

# 配置 Ingress (使用 Azure Application Gateway)
if [ "${SETUP_APP_GATEWAY:-false}" = "true" ]; then
    echo "🔧 配置 Azure Application Gateway..."
    
    # 安装 Application Gateway Ingress Controller
    helm repo add application-gateway-kubernetes-ingress https://appgwingress.blob.core.windows.net/ingress-azure-helm-package/
    helm repo update
    
    # 创建应用网关
    APP_GATEWAY_NAME="fechatter-appgw"
    
    az network application-gateway create \
        --name "$APP_GATEWAY_NAME" \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --location "$AZURE_LOCATION" \
        --vnet-name aks-vnet \
        --subnet appgw-subnet \
        --capacity 2 \
        --sku Standard_v2 \
        --http-settings-cookie-based-affinity Disabled \
        --frontend-port 80 \
        --http-settings-port 80 \
        --http-settings-protocol Http || true
fi

# 配置监控
if [ "${SETUP_MONITORING:-false}" = "true" ]; then
    echo "📊 配置 Azure Monitor..."
    
    # 启用容器监控
    az aks enable-addons \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --name "$CLUSTER_NAME" \
        --addons monitoring || true
    
    # 创建 Log Analytics 工作区
    WORKSPACE_NAME="fechatter-logs"
    
    az monitor log-analytics workspace create \
        --resource-group "$AZURE_RESOURCE_GROUP" \
        --workspace-name "$WORKSPACE_NAME" \
        --location "$AZURE_LOCATION" || true
fi

# 运行健康检查
echo "🔍 运行健康检查..."
./scripts/k8s-health-check.sh "$NAMESPACE"

# 获取服务 URL
echo "📋 服务信息:"
GATEWAY_URL=$(kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "pending")

if [ "$GATEWAY_URL" != "pending" ]; then
    echo "✅ 网关 URL: http://$GATEWAY_URL"
    echo "🧪 测试 API 连接..."
    
    # 等待 Load Balancer 就绪
    sleep 30
    
    if curl -f -s "http://$GATEWAY_URL/health" > /dev/null; then
        echo "✅ API 连接测试成功"
    else
        echo "⚠️  API 连接测试失败，可能需要等待 Load Balancer 完全就绪"
    fi
else
    echo "⏳ Load Balancer 正在配置中..."
    echo "使用以下命令检查状态:"
    echo "kubectl get service gateway -n $NAMESPACE -w"
fi

echo ""
echo "🎉 Azure AKS 部署完成！"
echo ""
echo "📋 部署信息:"
echo "  资源组: $AZURE_RESOURCE_GROUP"
echo "  集群: $CLUSTER_NAME"
echo "  命名空间: $NAMESPACE"
echo "  位置: $AZURE_LOCATION"
echo "  镜像标签: $IMAGE_TAG"
echo ""
echo "🔧 管理命令:"
echo "  查看状态: kubectl get all -n $NAMESPACE"
echo "  查看日志: kubectl logs -f deployment/gateway -n $NAMESPACE"
echo "  扩容服务: kubectl scale deployment/gateway --replicas=3 -n $NAMESPACE"
echo ""
echo "🔗 有用的链接:"
echo "  Azure Portal: https://portal.azure.com/#@/resource/subscriptions/$(az account show --query id -o tsv)/resourceGroups/$AZURE_RESOURCE_GROUP/overview"
echo "  AKS 集群: https://portal.azure.com/#@/resource/subscriptions/$(az account show --query id -o tsv)/resourceGroups/$AZURE_RESOURCE_GROUP/providers/Microsoft.ContainerService/managedClusters/$CLUSTER_NAME/overview"