#!/bin/bash
# deploy-gcp.sh - Google Cloud GKE 部署脚本

set -e

echo "🚀 开始部署到 Google Cloud GKE..."

# 配置变量
GCP_PROJECT=${GCP_PROJECT:-fechatter-project}
GCP_REGION=${GCP_REGION:-us-central1}
CLUSTER_NAME=${CLUSTER_NAME:-fechatter-cluster}
NAMESPACE=${NAMESPACE:-fechatter}
IMAGE_TAG=${IMAGE_TAG:-latest}

# 检查 gcloud CLI
if ! command -v gcloud &> /dev/null; then
    echo "❌ gcloud CLI 未安装"
    exit 1
fi

# 检查 kubectl
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl 未安装"
    exit 1
fi

# 检查 GCP 认证
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | head -n1 &> /dev/null; then
    echo "❌ GCP 认证失败"
    exit 1
fi

echo "✅ GCP 认证成功"
echo "项目: $GCP_PROJECT"
echo "区域: $GCP_REGION"

# 设置项目
gcloud config set project "$GCP_PROJECT"

# 获取集群凭据
echo "🔧 获取 GKE 集群凭据..."
gcloud container clusters get-credentials "$CLUSTER_NAME" --region="$GCP_REGION"

# 检查集群连接
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ 无法连接到 Kubernetes 集群"
    exit 1
fi

echo "✅ 已连接到 GKE 集群: $CLUSTER_NAME"

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

# 启用必要的 GCP API
echo "🔧 确保必要的 API 已启用..."
gcloud services enable container.googleapis.com
gcloud services enable compute.googleapis.com
gcloud services enable sqladmin.googleapis.com

# 部署基础设施服务
echo "🏗️  部署基础设施服务..."

# PostgreSQL (使用 Cloud SQL 或部署到集群)
if [ "${USE_CLOUD_SQL:-false}" = "true" ]; then
    echo "使用 Google Cloud SQL PostgreSQL"
    
    # 创建 Cloud SQL 实例
    INSTANCE_NAME="fechatter-postgres-$(date +%s)"
    
    gcloud sql instances create "$INSTANCE_NAME" \
        --database-version=POSTGRES_15 \
        --cpu=2 \
        --memory=7680MB \
        --region="$GCP_REGION" \
        --root-password="${POSTGRES_PASSWORD:-$(openssl rand -base64 16)}" || true
    
    # 创建数据库
    gcloud sql databases create fechatter --instance="$INSTANCE_NAME" || true
    
    # 配置连接
    gcloud sql instances patch "$INSTANCE_NAME" --authorized-networks=0.0.0.0/0 || true
else
    echo "部署 PostgreSQL 到集群"
    kubectl apply -f k8s/base/postgres.yaml -n "$NAMESPACE"
fi

# Redis (使用 Cloud Memorystore 或部署到集群)
if [ "${USE_MEMORYSTORE:-false}" = "true" ]; then
    echo "使用 Google Cloud Memorystore Redis"
    
    # 创建 Redis 实例
    REDIS_INSTANCE="fechatter-redis-$(date +%s)"
    
    gcloud redis instances create "$REDIS_INSTANCE" \
        --size=1 \
        --region="$GCP_REGION" \
        --redis-version=redis_6_x || true
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
echo "🐳 配置容器镜像仓库..."
gcloud auth configure-docker gcr.io

# 部署应用服务
echo "🚀 部署应用服务..."

# 更新镜像标签
sed -i.bak "s|:latest|:$IMAGE_TAG|g" k8s/base/*.yaml
sed -i.bak "s|fechatter/|gcr.io/$GCP_PROJECT/fechatter/|g" k8s/base/*.yaml

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

# 配置 Ingress (使用 Google Cloud Load Balancer)
if [ "${SETUP_INGRESS:-true}" = "true" ]; then
    echo "🔧 配置 Google Cloud Load Balancer..."
    
    # 创建 SSL 证书
    if [ -n "${DOMAIN_NAME:-}" ]; then
        gcloud compute ssl-certificates create fechatter-ssl \
            --domains="$DOMAIN_NAME" \
            --global || true
        
        # 创建带 SSL 的 Ingress
        cat > gcp-ingress.yaml << EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: fechatter-ingress
  namespace: $NAMESPACE
  annotations:
    kubernetes.io/ingress.class: "gce"
    kubernetes.io/ingress.global-static-ip-name: "fechatter-ip"
    ingress.gcp.kubernetes.io/ssl-certificate: "fechatter-ssl"
    kubernetes.io/ingress.allow-http: "false"
spec:
  rules:
  - host: $DOMAIN_NAME
    http:
      paths:
      - path: /*
        pathType: ImplementationSpecific
        backend:
          service:
            name: gateway
            port:
              number: 8080
EOF
        
        kubectl apply -f gcp-ingress.yaml
    fi
fi

# 配置监控
if [ "${SETUP_MONITORING:-false}" = "true" ]; then
    echo "📊 配置 Google Cloud Monitoring..."
    
    # 启用监控和日志记录
    gcloud services enable monitoring.googleapis.com
    gcloud services enable logging.googleapis.com
    
    # 安装 Google Cloud Monitoring 代理
    kubectl apply -f https://raw.githubusercontent.com/GoogleCloudPlatform/k8s-stackdriver/master/resources/stackdriver-agent.yaml
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
echo "🎉 Google Cloud GKE 部署完成！"
echo ""
echo "📋 部署信息:"
echo "  项目: $GCP_PROJECT"
echo "  集群: $CLUSTER_NAME"
echo "  命名空间: $NAMESPACE"
echo "  区域: $GCP_REGION"
echo "  镜像标签: $IMAGE_TAG"
echo ""
echo "🔧 管理命令:"
echo "  查看状态: kubectl get all -n $NAMESPACE"
echo "  查看日志: kubectl logs -f deployment/gateway -n $NAMESPACE"
echo "  扩容服务: kubectl scale deployment/gateway --replicas=3 -n $NAMESPACE"
echo ""
echo "🔗 有用的链接:"
echo "  GKE Console: https://console.cloud.google.com/kubernetes/clusters/details/$GCP_REGION/$CLUSTER_NAME?project=$GCP_PROJECT"
echo "  Cloud Logging: https://console.cloud.google.com/logs/query?project=$GCP_PROJECT"
echo "  Cloud Monitoring: https://console.cloud.google.com/monitoring?project=$GCP_PROJECT"