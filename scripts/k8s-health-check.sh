#!/bin/bash
# k8s-health-check.sh - Kubernetes 环境健康检查脚本

set -e

NAMESPACE=${1:-fechatter}
TIMEOUT=${2:-300}

echo "🔍 检查 Kubernetes 集群中的 Fechatter 服务健康状态..."
echo "命名空间: $NAMESPACE"
echo "超时时间: ${TIMEOUT}s"

# 检查命名空间是否存在
if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    echo "❌ 命名空间 $NAMESPACE 不存在"
    exit 1
fi

# 检查部署状态
check_deployment() {
    local deployment=$1
    local replicas_ready
    local replicas_desired
    
    echo "检查部署: $deployment"
    
    if ! kubectl get deployment "$deployment" -n "$NAMESPACE" >/dev/null 2>&1; then
        echo "⚠️  部署 $deployment 不存在，跳过"
        return 0
    fi
    
    # 等待部署就绪
    if kubectl wait --for=condition=available deployment/"$deployment" -n "$NAMESPACE" --timeout="${TIMEOUT}s"; then
        replicas_ready=$(kubectl get deployment "$deployment" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
        replicas_desired=$(kubectl get deployment "$deployment" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
        echo "✅ 部署 $deployment 就绪 ($replicas_ready/$replicas_desired)"
        return 0
    else
        echo "❌ 部署 $deployment 未就绪"
        kubectl describe deployment "$deployment" -n "$NAMESPACE"
        return 1
    fi
}

# 检查服务可达性
check_service() {
    local service=$1
    local port=$2
    local path=${3:-"/health"}
    
    echo "检查服务: $service"
    
    if ! kubectl get service "$service" -n "$NAMESPACE" >/dev/null 2>&1; then
        echo "⚠️  服务 $service 不存在，跳过"
        return 0
    fi
    
    # 端口转发测试连接
    local local_port=$((8000 + RANDOM % 1000))
    kubectl port-forward service/"$service" "$local_port:$port" -n "$NAMESPACE" >/dev/null 2>&1 &
    local pf_pid=$!
    
    sleep 3
    
    if curl -f -s "http://localhost:$local_port$path" >/dev/null 2>&1; then
        echo "✅ 服务 $service 响应正常"
        kill $pf_pid 2>/dev/null || true
        return 0
    else
        echo "❌ 服务 $service 无响应"
        kill $pf_pid 2>/dev/null || true
        return 1
    fi
}

# 检查 Pod 状态
check_pods() {
    local label_selector=$1
    local pod_count
    local running_count
    
    echo "检查 Pods: $label_selector"
    
    pod_count=$(kubectl get pods -l "$label_selector" -n "$NAMESPACE" --no-headers | wc -l)
    if [ "$pod_count" -eq 0 ]; then
        echo "⚠️  没有找到匹配的 Pod: $label_selector"
        return 0
    fi
    
    running_count=$(kubectl get pods -l "$label_selector" -n "$NAMESPACE" --field-selector=status.phase=Running --no-headers | wc -l)
    
    if [ "$running_count" -eq "$pod_count" ]; then
        echo "✅ 所有 Pod 运行正常: $label_selector ($running_count/$pod_count)"
        return 0
    else
        echo "❌ 部分 Pod 未运行: $label_selector ($running_count/$pod_count)"
        kubectl get pods -l "$label_selector" -n "$NAMESPACE"
        return 1
    fi
}

# 检查资源使用情况
check_resources() {
    echo "📊 检查资源使用情况..."
    
    echo "CPU 和内存使用:"
    kubectl top pods -n "$NAMESPACE" 2>/dev/null || echo "⚠️  Metrics Server 不可用"
    
    echo ""
    echo "存储使用:"
    kubectl get pvc -n "$NAMESPACE"
    
    echo ""
    echo "网络策略:"
    kubectl get networkpolicies -n "$NAMESPACE" 2>/dev/null || echo "⚠️  没有网络策略"
}

# 运行健康检查
echo "🚀 开始健康检查..."

# 1. 检查基础设施服务
echo ""
echo "📊 检查基础设施服务..."
check_deployment "postgres"
check_deployment "redis"
check_deployment "nats"
check_deployment "meilisearch"
check_deployment "clickhouse"

# 2. 检查应用服务
echo ""
echo "🚀 检查应用服务..."
check_deployment "fechatter-server"
check_deployment "notify-server"
check_deployment "bot-server"
check_deployment "analytics-server"
check_deployment "gateway"

# 3. 检查服务连通性
echo ""
echo "🔗 检查服务连通性..."
check_service "gateway" 8080 "/health"
check_service "fechatter-server" 6688 "/health"
check_service "notify-server" 6687 "/health"
check_service "bot-server" 6686 "/health"
check_service "analytics-server" 6690 "/health"

# 4. 检查 Pod 状态
echo ""
echo "🐳 检查 Pod 状态..."
check_pods "app=postgres"
check_pods "app=redis"
check_pods "app=nats"
check_pods "app=meilisearch"
check_pods "app=fechatter-server"
check_pods "app=gateway"

# 5. 检查资源使用
echo ""
check_resources

# 6. 端到端测试
echo ""
echo "🧪 运行端到端测试..."

# 获取网关 URL
GATEWAY_URL=""
if kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null; then
    GATEWAY_URL="http://$(kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')"
elif kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null; then
    GATEWAY_URL="http://$(kubectl get service gateway -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}')"
else
    echo "⚠️  无法获取网关外部 URL，使用端口转发"
    kubectl port-forward service/gateway 8080:8080 -n "$NAMESPACE" >/dev/null 2>&1 &
    GATEWAY_PF_PID=$!
    GATEWAY_URL="http://localhost:8080"
    sleep 3
fi

# 测试关键端点
if curl -f -s "$GATEWAY_URL/health" >/dev/null; then
    echo "✅ 网关健康检查通过"
else
    echo "❌ 网关健康检查失败"
fi

if curl -f -s "$GATEWAY_URL/api/v1/health" >/dev/null; then
    echo "✅ API 路由测试通过"
else
    echo "❌ API 路由测试失败"
fi

# 清理端口转发
if [ -n "$GATEWAY_PF_PID" ]; then
    kill $GATEWAY_PF_PID 2>/dev/null || true
fi

echo ""
echo "🎉 健康检查完成！"

# 生成报告
echo ""
echo "=== Fechatter Kubernetes 集群状态报告 ==="
echo "时间: $(date)"
echo "命名空间: $NAMESPACE"
echo ""

echo "部署状态:"
kubectl get deployments -n "$NAMESPACE" -o wide

echo ""
echo "服务状态:"
kubectl get services -n "$NAMESPACE" -o wide

echo ""
echo "Pod 状态:"
kubectl get pods -n "$NAMESPACE" -o wide

echo ""
echo "Ingress 状态:"
kubectl get ingress -n "$NAMESPACE" 2>/dev/null || echo "没有 Ingress 资源"

echo ""
echo "📊 集群健康检查完成！"