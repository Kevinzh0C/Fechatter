#!/bin/bash
# port-manager.sh - Fechatter 端口管理脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fechatter 服务端口配置
SERVICES="postgres:5432 redis:6379 nats-client:4222 nats-http:8222 nats-cluster:6222 meilisearch:7700 clickhouse-http:8123 clickhouse-native:9000 gateway:8080 fechatter-server:6688 notify-server:6687 bot-server:6686 analytics-server:6690"

# 显示帮助信息
show_help() {
    echo -e "${BLUE}Fechatter 端口管理工具${NC}"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  check       检查端口占用状态"
    echo "  clean       清理占用的端口"
    echo "  kill-all    强制停止所有相关进程"
    echo "  show        显示端口配置"
    echo "  help        显示此帮助信息"
    echo ""
}

# 检查端口占用
check_ports() {
    echo -e "${BLUE}🔍 检查 Fechatter 端口占用状态...${NC}"
    echo ""
    
    local occupied_count=0
    
    for service_port in $SERVICES; do
        local service=$(echo "$service_port" | cut -d: -f1)
        local port=$(echo "$service_port" | cut -d: -f2)
        
        if lsof -i ":$port" >/dev/null 2>&1; then
            local process=$(lsof -i ":$port" -t 2>/dev/null | head -1)
            local process_name=$(ps -p "$process" -o comm= 2>/dev/null || echo "未知")
            echo -e "${RED}❌ 端口 $port ($service) 被占用 - 进程: $process_name (PID: $process)${NC}"
            occupied_count=$((occupied_count + 1))
        else
            echo -e "${GREEN}✅ 端口 $port ($service) 可用${NC}"
        fi
    done
    
    echo ""
    if [ $occupied_count -gt 0 ]; then
        echo -e "${YELLOW}发现 $occupied_count 个端口被占用${NC}"
        echo -e "${YELLOW}运行 '$0 clean' 来清理这些端口${NC}"
        return 1
    else
        echo -e "${GREEN}🎉 所有端口都可用！${NC}"
        return 0
    fi
}

# 清理端口
clean_ports() {
    echo -e "${YELLOW}🧹 清理占用的端口...${NC}"
    echo ""
    
    local cleaned=0
    
    for service_port in $SERVICES; do
        local service=$(echo "$service_port" | cut -d: -f1)
        local port=$(echo "$service_port" | cut -d: -f2)
        
        if lsof -i ":$port" >/dev/null 2>&1; then
            local processes=$(lsof -i ":$port" -t 2>/dev/null)
            for process in $processes; do
                local process_name=$(ps -p "$process" -o comm= 2>/dev/null || echo "未知")
                echo -e "${YELLOW}停止进程: $process_name (PID: $process) 占用端口 $port${NC}"
                
                # 先尝试优雅停止
                if kill "$process" 2>/dev/null; then
                    sleep 2
                    # 检查是否还在运行
                    if kill -0 "$process" 2>/dev/null; then
                        echo -e "${RED}优雅停止失败，强制停止...${NC}"
                        kill -9 "$process" 2>/dev/null || true
                    fi
                    echo -e "${GREEN}✅ 进程 $process 已停止${NC}"
                    cleaned=$((cleaned + 1))
                else
                    echo -e "${RED}❌ 无法停止进程 $process${NC}"
                fi
            done
        fi
    done
    
    echo ""
    if [ $cleaned -gt 0 ]; then
        echo -e "${GREEN}🎉 成功清理 $cleaned 个进程${NC}"
        echo -e "${BLUE}等待 3 秒确保端口释放...${NC}"
        sleep 3
    else
        echo -e "${GREEN}✅ 没有需要清理的端口${NC}"
    fi
}

# 强制停止所有相关进程
kill_all() {
    echo -e "${RED}⚠️  强制停止所有 Fechatter 相关进程...${NC}"
    echo ""
    
    # 停止容器
    echo -e "${YELLOW}停止 Docker/Podman 容器...${NC}"
    podman stop $(podman ps -q) 2>/dev/null || true
    docker stop $(docker ps -q) 2>/dev/null || true
    
    # 停止已知的服务进程
    local services="postgres redis-server clickhouse meilisearch nats-server fechatter notify bot analytics gateway"
    
    for service in $services; do
        if pgrep -f "$service" >/dev/null 2>&1; then
            echo -e "${YELLOW}停止 $service 进程...${NC}"
            pkill -f "$service" 2>/dev/null || true
        fi
    done
    
    echo -e "${BLUE}等待 5 秒确保所有进程停止...${NC}"
    sleep 5
    
    echo -e "${GREEN}✅ 清理完成${NC}"
}

# 显示端口配置
show_ports() {
    echo -e "${BLUE}📋 Fechatter 端口配置${NC}"
    echo ""
    echo -e "${YELLOW}基础设施服务:${NC}"
    echo "  PostgreSQL      : 5432"
    echo "  Redis           : 6379"
    echo "  NATS (客户端)   : 4222"
    echo "  NATS (监控)     : 8222"
    echo "  NATS (集群)     : 6222"
    echo "  Meilisearch     : 7700"
    echo "  ClickHouse (HTTP): 8123"
    echo "  ClickHouse (原生): 9000"
    echo ""
    echo -e "${YELLOW}应用服务:${NC}"
    echo "  API 网关        : 8080"
    echo "  主服务          : 6688"
    echo "  通知服务 (SSE)  : 6687"
    echo "  AI 机器人       : 6686"
    echo "  分析服务        : 6690"
    echo ""
    echo -e "${BLUE}💡 注意: 项目使用 SSE (Server-Sent Events) 而不是 WebSocket 进行实时通知${NC}"
}

# 主函数
main() {
    case "${1:-help}" in
        "check")
            check_ports
            ;;
        "clean")
            clean_ports
            ;;
        "kill-all")
            kill_all
            ;;
        "show")
            show_ports
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# 执行主函数
main "$@" 