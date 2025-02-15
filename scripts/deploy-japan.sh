#!/bin/bash
# deploy-japan.sh - 日本地区优化部署脚本
# 专为 1000人规模 + 200日活用户设计

set -e

echo "🇯🇵 开始部署到 AWS Tokyo (ap-northeast-1)..."
echo "👥 目标规模: 1000用户 / 200日活"

# 配置变量
AWS_REGION="ap-northeast-1"  # Tokyo
CLUSTER_NAME=${CLUSTER_NAME:-fechatter-japan-prod}
NAMESPACE=${NAMESPACE:-fechatter}
IMAGE_TAG=${IMAGE_TAG:-latest}
DOMAIN=${DOMAIN:-fechatter-japan.com}

# 日本特定配置
TIMEZONE="Asia/Tokyo"
BUSINESS_HOURS_START="09:00"
BUSINESS_HOURS_END="22:00"

echo "🔧 配置信息:"
echo "  区域: $AWS_REGION (Tokyo)"
echo "  集群: $CLUSTER_NAME"
echo "  域名: $DOMAIN"
echo "  时区: $TIMEZONE"

# 检查依赖
for cmd in aws kubectl helm; do
    if ! command -v $cmd &> /dev/null; then
        echo "❌ $cmd 未安装"
        exit 1
    fi
done

# 检查 AWS 认证
if ! aws sts get-caller-identity &> /dev/null; then
    echo "❌ AWS 认证失败"
    exit 1
fi

echo "✅ AWS 认证成功"
echo "账户: $(aws sts get-caller-identity --query Account --output text)"

# 设置 AWS 区域
aws configure set region $AWS_REGION

# 更新 kubeconfig
echo "🔧 连接到 EKS 集群..."
aws eks update-kubeconfig --region "$AWS_REGION" --name "$CLUSTER_NAME"

# 验证集群连接
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ 无法连接到 Kubernetes 集群"
    echo "请先创建 EKS 集群: eksctl create cluster --name $CLUSTER_NAME --region $AWS_REGION"
    exit 1
fi

echo "✅ 已连接到 EKS 集群: $CLUSTER_NAME"

# 创建命名空间
echo "📦 创建命名空间..."
kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

# 配置时区
kubectl create configmap timezone-config \
    --from-literal=TZ="$TIMEZONE" \
    --namespace "$NAMESPACE" \
    --dry-run=client -o yaml | kubectl apply -f -

# 创建密钥
echo "🔐 配置应用密钥..."
kubectl create secret generic fechatter-secrets \
    --from-literal=jwt-secret="${JWT_SECRET:-$(openssl rand -base64 32)}" \
    --from-literal=redis-password="${REDIS_PASSWORD:-$(openssl rand -base64 16)}" \
    --from-literal=meili-master-key="${MEILI_MASTER_KEY:-$(openssl rand -base64 32)}" \
    --from-literal=postgres-password="${POSTGRES_PASSWORD:-$(openssl rand -base64 16)}" \
    --namespace "$NAMESPACE" \
    --dry-run=client -o yaml | kubectl apply -f -

# 安装 AWS Load Balancer Controller
echo "🔧 安装 AWS Load Balancer Controller..."
helm repo add eks https://aws.github.io/eks-charts
helm repo update

# 创建 IAM 角色 (如果不存在)
eksctl create iamserviceaccount \
    --cluster="$CLUSTER_NAME" \
    --namespace=kube-system \
    --name=aws-load-balancer-controller \
    --role-name AmazonEKSLoadBalancerControllerRole \
    --attach-policy-arn=arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):policy/AWSLoadBalancerControllerIAMPolicy \
    --approve --region="$AWS_REGION" || true

# 安装 Load Balancer Controller
helm upgrade --install aws-load-balancer-controller eks/aws-load-balancer-controller \
    -n kube-system \
    --set clusterName="$CLUSTER_NAME" \
    --set serviceAccount.create=false \
    --set serviceAccount.name=aws-load-balancer-controller \
    --set region="$AWS_REGION" || true

# 部署基础设施服务 (使用 AWS 托管服务)
echo "🏗️  配置基础设施服务..."

# 使用 AWS RDS PostgreSQL
if [ "${USE_AWS_RDS:-true}" = "true" ]; then
    echo "✅ 使用 AWS RDS PostgreSQL (推荐)"
    
    # 创建数据库连接配置
    kubectl create configmap database-config \
        --from-literal=DATABASE_URL="postgresql://postgres:${POSTGRES_PASSWORD}@fechatter-japan-db.cluster-xxx.ap-northeast-1.rds.amazonaws.com:5432/fechatter" \
        --namespace "$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
else
    echo "部署 PostgreSQL 到集群"
    kubectl apply -f k8s/japan/postgres.yaml -n "$NAMESPACE"
fi

# 使用 AWS ElastiCache Redis
if [ "${USE_AWS_ELASTICACHE:-true}" = "true" ]; then
    echo "✅ 使用 AWS ElastiCache Redis (推荐)"
    
    # 创建 Redis 连接配置
    kubectl create configmap redis-config \
        --from-literal=REDIS_URL="redis://fechatter-japan-cache.xxx.cache.amazonaws.com:6379" \
        --namespace "$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
else
    echo "部署 Redis 到集群"
    kubectl apply -f k8s/japan/redis.yaml -n "$NAMESPACE"
fi

# 部署其他基础设施服务到集群
kubectl apply -f k8s/japan/nats.yaml -n "$NAMESPACE"
kubectl apply -f k8s/japan/meilisearch.yaml -n "$NAMESPACE"

# 等待服务就绪
echo "⏳ 等待基础设施服务就绪..."
kubectl wait --for=condition=available deployment/nats -n "$NAMESPACE" --timeout=300s || true
kubectl wait --for=condition=available deployment/meilisearch -n "$NAMESPACE" --timeout=300s || true

# 部署应用服务 (日本优化配置)
echo "🚀 部署应用服务..."

# 更新镜像标签
for file in k8s/japan/*.yaml; do
    sed -i.bak "s|:latest|:$IMAGE_TAG|g" "$file"
    sed -i.bak "s|ghcr.io/|ghcr.io/$GITHUB_REPOSITORY_OWNER/|g" "$file" 2>/dev/null || true
done

# 应用日本优化的配置
kubectl apply -f k8s/japan/fechatter-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/japan/notify-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/japan/bot-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/japan/analytics-server.yaml -n "$NAMESPACE"
kubectl apply -f k8s/japan/gateway.yaml -n "$NAMESPACE"

# 等待应用服务就绪
echo "⏳ 等待应用服务就绪..."
kubectl wait --for=condition=available deployment/fechatter-server -n "$NAMESPACE" --timeout=600s
kubectl wait --for=condition=available deployment/notify-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/bot-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/analytics-server -n "$NAMESPACE" --timeout=300s
kubectl wait --for=condition=available deployment/gateway -n "$NAMESPACE" --timeout=300s

# 配置 ALB Ingress
echo "🌐 配置 Application Load Balancer..."

# 获取 ACM 证书 ARN (需要预先在 ACM 中申请)
ACM_CERT_ARN=$(aws acm list-certificates \
    --region "$AWS_REGION" \
    --query "CertificateSummaryList[?DomainName=='$DOMAIN'].CertificateArn" \
    --output text 2>/dev/null || echo "")

if [ -n "$ACM_CERT_ARN" ]; then
    echo "✅ 找到 SSL 证书: $ACM_CERT_ARN"
    
    # 创建带 SSL 的 Ingress
    cat > japan-ingress.yaml << EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: fechatter-japan-ingress
  namespace: $NAMESPACE
  annotations:
    kubernetes.io/ingress.class: alb
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/certificate-arn: $ACM_CERT_ARN
    alb.ingress.kubernetes.io/listen-ports: '[{"HTTP": 80}, {"HTTPS": 443}]'
    alb.ingress.kubernetes.io/ssl-redirect: '443'
    alb.ingress.kubernetes.io/healthcheck-path: /health
    alb.ingress.kubernetes.io/healthcheck-interval-seconds: '30'
    alb.ingress.kubernetes.io/healthcheck-timeout-seconds: '5'
    alb.ingress.kubernetes.io/success-codes: '200'
spec:
  rules:
  - host: $DOMAIN
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: gateway
            port:
              number: 8080
  - host: api.$DOMAIN
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: gateway
            port:
              number: 8080
EOF
    
    kubectl apply -f japan-ingress.yaml
else
    echo "⚠️  未找到 SSL 证书，请在 AWS ACM 中为 $DOMAIN 申请证书"
    echo "命令: aws acm request-certificate --domain-name $DOMAIN --subject-alternative-names api.$DOMAIN --validation-method DNS --region $AWS_REGION"
fi

# 设置 HPA (水平自动扩缩容)
echo "📈 配置自动扩缩容..."
kubectl apply -f k8s/japan/hpa.yaml -n "$NAMESPACE"

# 配置监控
echo "📊 配置监控..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# 安装 Prometheus + Grafana
helm upgrade --install monitoring prometheus-community/kube-prometheus-stack \
    --namespace monitoring \
    --create-namespace \
    --set grafana.adminPassword="${GRAFANA_PASSWORD:-admin123}" \
    --set prometheus.prometheusSpec.retention=7d \
    --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=20Gi

# 运行健康检查
echo "🔍 运行健康检查..."
./scripts/k8s-health-check.sh "$NAMESPACE"

# 获取服务信息
echo "📋 获取服务信息..."
GATEWAY_URL=""
if kubectl get ingress fechatter-japan-ingress -n "$NAMESPACE" &>/dev/null; then
    GATEWAY_URL=$(kubectl get ingress fechatter-japan-ingress -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "pending")
fi

if [ -n "$GATEWAY_URL" ] && [ "$GATEWAY_URL" != "pending" ]; then
    echo "✅ 服务 URL: https://$DOMAIN"
    echo "🧪 测试 API 连接..."
    
    # 等待 ALB 完全就绪
    sleep 60
    
    if curl -f -s -k "https://$DOMAIN/health" > /dev/null; then
        echo "✅ API 连接测试成功"
    else
        echo "⚠️  API 连接测试失败，ALB 可能还在配置中"
    fi
else
    echo "⏳ ALB 正在配置中，请等待几分钟"
    echo "检查状态: kubectl get ingress fechatter-japan-ingress -n $NAMESPACE -w"
fi

# 显示资源使用情况
echo "📊 当前资源使用情况:"
kubectl top nodes 2>/dev/null || echo "需要安装 metrics-server"
kubectl top pods -n "$NAMESPACE" 2>/dev/null || echo "Pods 指标收集中..."

echo ""
echo "🎉 日本地区部署完成！"
echo ""
echo "📋 部署信息:"
echo "  🌏 区域: $AWS_REGION (Tokyo)"
echo "  🏢 集群: $CLUSTER_NAME"
echo "  📦 命名空间: $NAMESPACE"
echo "  🌐 域名: $DOMAIN"
echo "  🏷️  镜像标签: $IMAGE_TAG"
echo "  🕐 时区: $TIMEZONE"
echo ""
echo "👥 规模配置:"
echo "  📊 目标用户: 1000人"
echo "  📈 日活用户: 200人"
echo "  ⚡ 峰值在线: ~50-80人"
echo "  💬 日消息量: ~4,000-10,000条"
echo ""
echo "🔧 管理命令:"
echo "  查看状态: kubectl get all -n $NAMESPACE"
echo "  查看日志: kubectl logs -f deployment/gateway -n $NAMESPACE"
echo "  扩容网关: kubectl scale deployment/gateway --replicas=4 -n $NAMESPACE"
echo "  查看监控: kubectl port-forward -n monitoring svc/monitoring-grafana 3000:80"
echo ""
echo "📈 监控面板:"
echo "  Grafana: kubectl port-forward -n monitoring svc/monitoring-grafana 3000:80"
echo "  用户名: admin"
echo "  密码: ${GRAFANA_PASSWORD:-admin123}"
echo ""
echo "🔗 AWS 控制台:"
echo "  EKS: https://ap-northeast-1.console.aws.amazon.com/eks/home?region=ap-northeast-1#/clusters/$CLUSTER_NAME"
echo "  CloudWatch: https://ap-northeast-1.console.aws.amazon.com/cloudwatch/home?region=ap-northeast-1"
echo "  RDS: https://ap-northeast-1.console.aws.amazon.com/rds/home?region=ap-northeast-1"
echo ""
echo "✨ 部署优化建议:"
echo "  🎯 使用 Reserved Instances 节省成本"
echo "  📊 监控 CPU/内存使用率并调整资源配置"
echo "  🔄 设置定期备份和灾难恢复计划"
echo "  🚀 考虑启用 AWS Fargate 进一步优化成本"