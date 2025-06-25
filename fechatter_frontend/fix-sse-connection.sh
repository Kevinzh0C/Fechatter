#!/bin/bash

# SSE连接错误快速修复脚本
# 根据深度根因分析自动解决SSE连接问题

set -e

echo "🔍 SSE连接错误诊断与修复工具"
echo "================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查函数
check_port() {
    local port=$1
    local service_name=$2
    echo -n "🔍 检查端口 $port ($service_name): "
    
    if lsof -i :$port >/dev/null 2>&1; then
        local pid=$(lsof -ti :$port)
        local process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        echo -e "${YELLOW}占用${NC} (PID: $pid, Process: $process)"
        return 0
    else
        echo -e "${RED}未占用${NC}"
        return 1
    fi
}

check_service() {
    local url=$1
    local service_name=$2
    echo -n "🔍 检查服务 $service_name ($url): "
    
    if curl -s --connect-timeout 3 "$url" >/dev/null 2>&1; then
        echo -e "${GREEN}运行中${NC}"
        return 0
    else
        echo -e "${RED}不可达${NC}"
        return 1
    fi
}

# 步骤1: 环境诊断
echo -e "\n${BLUE}步骤1: 环境诊断${NC}"
echo "================================"

# 检查当前目录
if [[ ! -f "package.json" ]]; then
    echo -e "${RED}❌ 错误: 请在fechatter_frontend目录下运行此脚本${NC}"
    exit 1
fi

# 检查端口占用情况
echo "📊 端口占用情况:"
check_port 1420 "Frontend Dev Server"
frontend_running=$?

check_port 8080 "Gateway Service"
gateway_running=$?

check_port 8081 "Fechatter Server"
server_running=$?

check_port 8082 "Notify Server"
notify_running=$?

# 检查服务健康状态
echo -e "\n📊 服务健康状态:"
check_service "http://127.0.0.1:8080/health" "Gateway Health"
gateway_healthy=$?

check_service "http://127.0.0.1:8080/events" "SSE Endpoint"
sse_healthy=$?

# 步骤2: 问题诊断
echo -e "\n${BLUE}步骤2: 问题诊断${NC}"
echo "================================"

problem_found=false

if [[ $frontend_running -eq 0 ]] && [[ $gateway_running -ne 0 || $gateway_healthy -ne 0 ]]; then
    echo -e "${RED}❌ 根本问题: 前端运行但后端Gateway未启动${NC}"
    echo "   前端尝试连接 http://127.0.0.1:8080/events 但服务不可达"
    problem_found=true
elif [[ $gateway_running -eq 0 ]] && [[ $sse_healthy -ne 0 ]]; then
    echo -e "${YELLOW}⚠️  端口8080被非Gateway服务占用${NC}"
    local pid=$(lsof -ti :8080)
    local process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
    echo "   占用进程: $process (PID: $pid)"
    problem_found=true
fi

if [[ $problem_found == false ]]; then
    echo -e "${GREEN}✅ 未发现明显问题，SSE连接应该正常工作${NC}"
    exit 0
fi

# 步骤3: 自动修复
echo -e "\n${BLUE}步骤3: 自动修复${NC}"
echo "================================"

echo "🔧 开始自动修复流程..."

# 如果前端已运行，先停止
if [[ $frontend_running -eq 0 ]]; then
    echo "🛑 停止当前前端服务..."
    local frontend_pid=$(lsof -ti :1420)
    if [[ -n "$frontend_pid" ]]; then
        kill $frontend_pid
        sleep 2
        echo -e "${GREEN}✅ 前端服务已停止${NC}"
    fi
fi

# 检查并处理端口8080冲突
if [[ $gateway_running -eq 0 ]] && [[ $sse_healthy -ne 0 ]]; then
    echo -e "${YELLOW}⚠️  检测到端口8080冲突，需要手动处理${NC}"
    local pid=$(lsof -ti :8080)
    local process=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
    
    echo "占用端口8080的进程: $process (PID: $pid)"
    echo "这可能是Docker或其他服务，建议检查是否可以安全终止"
    
    read -p "是否要终止此进程? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kill $pid
        sleep 2
        echo -e "${GREEN}✅ 端口8080已释放${NC}"
    else
        echo -e "${YELLOW}⚠️  跳过端口释放，请手动处理端口冲突${NC}"
    fi
fi

# 步骤4: 启动后端服务
echo -e "\n${BLUE}步骤4: 启动后端服务${NC}"
echo "================================"

# 切换到项目根目录
cd ..

echo "📍 当前目录: $(pwd)"

# 检查是否有Makefile或启动脚本
if [[ -f "Makefile" ]]; then
    echo "🚀 发现Makefile，尝试启动本地服务..."
    
    # 检查可用的make目标
    if make -n start-local >/dev/null 2>&1; then
        echo "执行: make start-local"
        make start-local &
        startup_pid=$!
        
        echo "等待服务启动 (10秒)..."
        sleep 10
        
        # 检查服务是否启动成功
        if check_service "http://127.0.0.1:8080/health" "Gateway" >/dev/null; then
            echo -e "${GREEN}✅ 后端服务启动成功${NC}"
        else
            echo -e "${RED}❌ 后端服务启动失败，请手动检查${NC}"
            echo "请尝试: make start-local 或 docker-compose up"
        fi
    else
        echo -e "${YELLOW}⚠️  未找到 start-local 目标，请手动启动后端服务${NC}"
        echo "建议命令:"
        echo "  - make start"
        echo "  - docker-compose up"
        echo "  - cargo run (如果是Rust项目)"
    fi
elif [[ -f "docker-compose.yml" ]]; then
    echo "🐳 发现docker-compose.yml，尝试启动服务..."
    docker-compose up -d
    
    echo "等待服务启动 (15秒)..."
    sleep 15
    
    if check_service "http://127.0.0.1:8080/health" "Gateway" >/dev/null; then
        echo -e "${GREEN}✅ Docker服务启动成功${NC}"
    else
        echo -e "${RED}❌ Docker服务启动失败，请检查日志${NC}"
        echo "检查命令: docker-compose logs"
    fi
else
    echo -e "${YELLOW}⚠️  未找到标准启动配置，请手动启动后端服务${NC}"
    echo "项目根目录应该包含 Makefile 或 docker-compose.yml"
fi

# 步骤5: 重启前端服务
echo -e "\n${BLUE}步骤5: 重启前端服务${NC}"
echo "================================"

cd fechatter_frontend

echo "🚀 启动前端开发服务器..."
yarn dev &
frontend_pid=$!

echo "等待前端服务启动 (5秒)..."
sleep 5

# 最终验证
echo -e "\n${BLUE}步骤6: 最终验证${NC}"
echo "================================"

echo "🔍 验证修复结果..."

# 检查服务状态
check_service "http://127.0.0.1:8080/events" "SSE Endpoint"
sse_status=$?

check_port 1420 "Frontend Dev Server"
frontend_status=$?

if [[ $sse_status -eq 0 ]] && [[ $frontend_status -eq 0 ]]; then
    echo -e "\n${GREEN}🎉 修复成功!${NC}"
    echo "================================"
    echo -e "${GREEN}✅ SSE端点可达: http://127.0.0.1:8080/events${NC}"
    echo -e "${GREEN}✅ 前端服务运行: http://localhost:1420${NC}"
    echo ""
    echo "现在可以在浏览器中访问前端应用，SSE连接错误应该已解决。"
    echo ""
    echo "🔍 验证步骤:"
    echo "1. 打开浏览器访问 http://localhost:1420"
    echo "2. 打开开发者工具 → Console"
    echo "3. 观察是否还有 'SSE connection error' 错误"
    echo "4. 检查Network标签中的EventSource连接状态"
else
    echo -e "\n${RED}❌ 修复未完全成功${NC}"
    echo "================================"
    
    if [[ $sse_status -ne 0 ]]; then
        echo -e "${RED}❌ SSE端点仍不可达${NC}"
        echo "请手动启动后端服务或检查端口配置"
    fi
    
    if [[ $frontend_status -ne 0 ]]; then
        echo -e "${RED}❌ 前端服务启动失败${NC}"
        echo "请手动运行: yarn dev"
    fi
    
    echo ""
    echo "📋 手动修复步骤:"
    echo "1. 确保所有后端服务运行: make start-local"
    echo "2. 确保前端服务运行: yarn dev"
    echo "3. 检查端口冲突: lsof -i :8080"
    echo "4. 查看详细分析: cat SSE_ERROR_ROOT_CAUSE_ANALYSIS.md"
fi

echo ""
echo "📚 相关文档:"
echo "- 详细根因分析: SSE_ERROR_ROOT_CAUSE_ANALYSIS.md"
echo "- 修复验证指南: HOW_TO_VERIFY_FIXES.md"
echo "- 导航错误修复: NAVIGATION_SSE_FIX_SUMMARY.md" 