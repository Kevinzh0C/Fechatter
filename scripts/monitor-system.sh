#!/bin/bash

# Fechatter系统负载监控脚本
# 用于监控系统资源使用情况并发出警告

LOG_FILE="/var/log/fechatter-monitor.log"
ALERT_THRESHOLD_LOAD=3.5    # 单核系统合理阈值
ALERT_THRESHOLD_CPU=80
ALERT_THRESHOLD_MEM=85

# 颜色定义
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# 日志函数
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a $LOG_FILE
}

# 检查系统负载
check_load() {
    local load_avg=$(uptime | awk '{print $10}' | sed 's/,//')
    local load_num=$(echo $load_avg | sed 's/,//')
    
    echo -e "${GREEN}系统负载检查:${NC}"
    uptime
    
    if (( $(echo "$load_num > $ALERT_THRESHOLD_LOAD" | bc -l) )); then
        echo -e "${RED}⚠️  警告: 系统负载过高 ($load_avg > $ALERT_THRESHOLD_LOAD)${NC}"
        log "HIGH LOAD WARNING: $load_avg"
        return 1
    else
        echo -e "${GREEN}✅ 系统负载正常 ($load_avg)${NC}"
        return 0
    fi
}

# 检查CPU使用率
check_cpu() {
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | sed 's/%us,//')
    local cpu_num=$(echo $cpu_usage | sed 's/%//')
    
    echo -e "\n${GREEN}CPU使用率检查:${NC}"
    top -bn1 | grep "Cpu(s)"
    
    if (( $(echo "$cpu_num > $ALERT_THRESHOLD_CPU" | bc -l) )); then
        echo -e "${RED}⚠️  警告: CPU使用率过高 ($cpu_usage% > $ALERT_THRESHOLD_CPU%)${NC}"
        log "HIGH CPU WARNING: $cpu_usage%"
        return 1
    else
        echo -e "${GREEN}✅ CPU使用率正常 ($cpu_usage%)${NC}"
        return 0
    fi
}

# 检查内存使用率
check_memory() {
    local mem_info=$(free | grep Mem)
    local total=$(echo $mem_info | awk '{print $2}')
    local used=$(echo $mem_info | awk '{print $3}')
    local mem_percent=$(( used * 100 / total ))
    
    echo -e "\n${GREEN}内存使用检查:${NC}"
    free -h
    
    if [ $mem_percent -gt $ALERT_THRESHOLD_MEM ]; then
        echo -e "${RED}⚠️  警告: 内存使用率过高 ($mem_percent% > $ALERT_THRESHOLD_MEM%)${NC}"
        log "HIGH MEMORY WARNING: $mem_percent%"
        return 1
    else
        echo -e "${GREEN}✅ 内存使用率正常 ($mem_percent%)${NC}"
        return 0
    fi
}

# 检查Docker容器状态
check_containers() {
    echo -e "\n${GREEN}Docker容器状态检查:${NC}"
    
    # 检查不健康的容器
    local unhealthy=$(docker ps --format "table {{.Names}}\t{{.Status}}" | grep -E "(unhealthy|restarting)" || true)
    
    if [ -n "$unhealthy" ]; then
        echo -e "${RED}⚠️  发现不健康的容器:${NC}"
        echo "$unhealthy"
        log "UNHEALTHY CONTAINERS: $unhealthy"
        return 1
    else
        echo -e "${GREEN}✅ 所有容器状态正常${NC}"
        return 0
    fi
}

# 检查ClickHouse资源使用
check_clickhouse() {
    echo -e "\n${GREEN}ClickHouse资源检查:${NC}"
    
    local ch_stats=$(docker stats --no-stream --format "{{.CPUPerc}}\t{{.MemUsage}}" fechatter-clickhouse-vcr 2>/dev/null || echo "N/A N/A")
    local cpu_percent=$(echo $ch_stats | awk '{print $1}' | sed 's/%//')
    local mem_usage=$(echo $ch_stats | awk '{print $2}')
    
    if [ "$cpu_percent" != "N/A" ]; then
        echo "ClickHouse CPU: $cpu_percent%, Memory: $mem_usage"
        
        if (( $(echo "$cpu_percent > 60" | bc -l) )); then
            echo -e "${YELLOW}⚠️  ClickHouse CPU使用率较高: $cpu_percent%${NC}"
            log "CLICKHOUSE HIGH CPU: $cpu_percent%"
        else
            echo -e "${GREEN}✅ ClickHouse资源使用正常${NC}"
        fi
    else
        echo -e "${RED}❌ 无法获取ClickHouse状态${NC}"
        return 1
    fi
}

# 主监控函数
main() {
    echo "======================================"
    echo "Fechatter系统监控报告 - $(date)"
    echo "======================================"
    
    local issues=0
    
    check_load || ((issues++))
    check_cpu || ((issues++))
    check_memory || ((issues++))
    check_containers || ((issues++))
    check_clickhouse || ((issues++))
    
    echo -e "\n======================================"
    if [ $issues -eq 0 ]; then
        echo -e "${GREEN}✅ 系统状态良好，无需关注${NC}"
        log "SYSTEM STATUS: HEALTHY"
    else
        echo -e "${YELLOW}⚠️  发现 $issues 个需要关注的问题${NC}"
        log "SYSTEM STATUS: $issues ISSUES FOUND"
    fi
    echo "======================================"
    
    return $issues
}

# 如果作为脚本直接运行
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi 