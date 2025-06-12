#!/bin/bash
# deploy-aws.sh - AWS EKS 部署脚本

set -e

echo "🚀 开始部署到 AWS EKS..."

# 配置变量
AWS_REGION=${AWS_REGION:-us-west-2}
CLUSTER_NAME=${CLUSTER_NAME:-fechatter-cluster}
NAMESPACE=${NAMESPACE:-fechatter}
IMAGE_TAG=${IMAGE_TAG:-latest}

# 检查 AWS CLI
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI 未安装"
    exit 1
fi

# 检查 kubectl
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl 未安装"
    exit 1
fi

# 检查 AWS 认证
if ! aws sts get-caller-identity &> /dev/null; then
    echo "❌ AWS 认证失败"
    exit 1
fi

echo "✅ AWS 认证成功"
echo "账户: $(aws sts get-caller-identity --query Account --output text)"
echo "区域: $AWS_REGION"

# 更新 kubeconfig
echo "🔧 更新 kubeconfig..."
aws eks update-kubeconfig --region "$AWS_REGION" --name "$CLUSTER_NAME"

# 检查集群连接
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ 无法连接到 Kubernetes 集群"
    exit 1
fi

echo "✅ 已连接到 EKS 集群: $CLUSTER_NAME"

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

# PostgreSQL (使用 AWS RDS 或部署到集群)
if [ "${USE_AWS_RDS:-false}" = "true" ]; then
    echo "使用 AWS RDS PostgreSQL"
    # 这里可以添加 RDS 实例创建逻辑
else
    echo "部署 PostgreSQL 到集群"
    kubectl apply -f k8s/base/postgres.yaml -n "$NAMESPACE"
fi

# Redis (使用 AWS ElastiCache 或部署到集群)
if [ "${USE_AWS_ELASTICACHE:-false}" = "true" ]; then
    echo "使用 AWS ElastiCache Redis"
    # 这里可以添加 ElastiCache 集群创建逻辑
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

# 部署应用服务
echo "🚀 部署应用服务..."

# 更新镜像标签
sed -i.bak "s|:latest|:$IMAGE_TAG|g" k8s/base/*.yaml

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

# 配置 AWS Load Balancer Controller (如果需要)
if [ "${SETUP_ALB:-false}" = "true" ]; then
    echo "🔧 配置 AWS Load Balancer Controller..."
    
    # 安装 AWS Load Balancer Controller
    curl -o iam_policy.json https://raw.githubusercontent.com/kubernetes-sigs/aws-load-balancer-controller/v2.7.2/docs/install/iam_policy.json
    
    aws iam create-policy \
        --policy-name AWSLoadBalancerControllerIAMPolicy \
        --policy-document file://iam_policy.json || true
    
    # 创建服务账户
    eksctl create iamserviceaccount \
        --cluster="$CLUSTER_NAME" \
        --namespace=kube-system \
        --name=aws-load-balancer-controller \
        --role-name AmazonEKSLoadBalancerControllerRole \
        --attach-policy-arn=arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):policy/AWSLoadBalancerControllerIAMPolicy \
        --approve || true
fi

# 运行健康检查
echo "🔍 运行健康检查..."
./scripts/k8s-health-check.sh "$NAMESPACE"

# 获取服务 URL
echo "📋 服务信息:"
GATEWAY_URL=$(kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "pending")

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

# 设置监控 (可选)
if [ "${SETUP_MONITORING:-false}" = "true" ]; then
    echo "📊 设置监控..."
    
    # 安装 Prometheus 和 Grafana
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    helm install prometheus prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --create-namespace
fi

echo ""
echo "🎉 AWS EKS 部署完成！"
echo ""
echo "📋 部署信息:"
echo "  集群: $CLUSTER_NAME"
echo "  命名空间: $NAMESPACE"
echo "  区域: $AWS_REGION"
echo "  镜像标签: $IMAGE_TAG"
echo ""
echo "🔧 管理命令:"
echo "  查看状态: kubectl get all -n $NAMESPACE"
echo "  查看日志: kubectl logs -f deployment/gateway -n $NAMESPACE"
echo "  扩容服务: kubectl scale deployment/gateway --replicas=3 -n $NAMESPACE"
echo ""
echo "🔗 有用的链接:"
echo "  AWS EKS Console: https://$AWS_REGION.console.aws.amazon.com/eks/home?region=$AWS_REGION#/clusters/$CLUSTER_NAME"
echo "  CloudWatch Logs: https://$AWS_REGION.console.aws.amazon.com/cloudwatch/home?region=$AWS_REGION#logsV2:log-groups"