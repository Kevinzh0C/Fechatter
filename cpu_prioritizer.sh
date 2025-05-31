#!/bin/bash
#
# cpu_prioritizer.sh - macOS开发环境CPU资源优化脚本
#
# 此脚本针对特定的开发环境应用程序优化CPU资源分配，包括:
# - rust-analyzer
# - rustc
# - Cursor编辑器 (特别关注网络I/O)
# - OpenAI相关进程
# - Warp终端
#
# 作者: AI助手
# 日期: 2025-05-29
# 版本: 1.0

# 文本格式化
BOLD="\033[1m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
RESET="\033[0m"

# 全局变量
PRIORITY_PIDS=()
ORIGINAL_NICE_VALUES=()
LOWERED_PIDS=()
LOWERED_ORIGINAL_VALUES=()
STOPPED_SERVICES=()
TEMP_FILE="/tmp/cpu_prioritizer_$$.tmp"
LOG_FILE="/tmp/cpu_prioritizer_log_$$.txt"

# 默认值
PRIORITY_NICE_VALUE=-10
LOWER_NICE_VALUE=10
CPU_THRESHOLD=10
DRY_RUN=false
MONITOR_ENABLED=false
MONITOR_INTERVAL=2
RESTORE_MODE=false
AUTO_MODE=true
FOCUS_CURSOR_NETWORK=true

# 打印使用信息
print_usage() {
    echo -e "${BOLD}使用方法:${RESET}"
    echo -e "  $0 [选项]"
    echo
    echo -e "${BOLD}描述:${RESET}"
    echo "  此脚本针对开发环境优化CPU资源分配，优先处理特定的应用程序"
    echo "  (rust-analyzer, rustc, Cursor编辑器, OpenAI, Warp终端)，"
    echo "  并可选择性地降低其他CPU密集型进程的优先级。"
    echo
    echo -e "${BOLD}选项:${RESET}"
    echo "  -h, --help                 显示此帮助信息并退出"
    echo "  -v, --priority-value N     设置优先进程的nice值(默认: -10, 范围: -20到0)"
    echo "  -l, --lower-others         降低其他CPU密集型进程的优先级"
    echo "  -t, --threshold N          降低优先级的CPU使用率阈值%(默认: 10)"
    echo "  -c, --lower-value N        设置降低优先级进程的nice值(默认: 10, 范围: 0到20)"
    echo "  -s, --stop-services        停止非必要的后台服务"
    echo "  -m, --monitor [seconds]    监控优化后的CPU使用情况(默认间隔: 2秒)"
    echo "  -d, --dry-run              显示将要执行的操作而不实际进行更改"
    echo "  -r, --restore              恢复原始进程优先级并启动已停止的服务"
    echo "  -n, --no-auto              禁用自动模式(不自动查找目标进程)"
    echo "  -p, --pids PID1 [PID2...] 指定要优先处理的进程ID"
    echo "  --no-cursor-network        不特别优化Cursor的网络I/O"
    echo
    echo -e "${BOLD}示例:${RESET}"
    echo "  $0                          # 自动模式，查找并优化所有指定的开发工具"
    echo "  $0 -l -s                    # 同时降低其他进程优先级并停止非必要服务"
    echo "  $0 -v -15 -c 15             # 使用自定义优先级值"
    echo "  $0 -r                       # 恢复原始设置"
    echo
}

# 检查脚本是否以sudo/root权限运行
check_privileges() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${YELLOW}警告: 某些操作需要root权限。${RESET}"
        echo -e "考虑使用sudo运行以获取完整功能。"
        echo
        return 1
    fi
    return 0
}

# 根据进程名查找PID
find_pid_by_name() {
    local process_name="$1"
    local pids=()
    
    # 使用pgrep查找匹配进程名的PID
    pids=($(pgrep -i "$process_name"))
    
    if [ ${#pids[@]} -eq 0 ]; then
        echo -e "${YELLOW}未找到名为\"$process_name\"的进程${RESET}" >&2
        return 1
    fi
    
    # 返回所有匹配的PID
    echo "${pids[@]}"
    return 0
}

# 验证PID
validate_pid() {
    local pid="$1"
    
    # 检查PID是否为数字
    if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}错误: 无效的PID \"$pid\"。PID必须是数字。${RESET}" >&2
        return 1
    fi
    
    # 检查进程是否存在
    if ! ps -p "$pid" > /dev/null; then
        echo -e "${RED}错误: PID为$pid的进程不存在。${RESET}" >&2
        return 1
    fi
    
    return 0
}

# 获取进程当前nice值
get_nice_value() {
    local pid="$1"
    ps -o nice= -p "$pid"
}

# 使用renice设置进程优先级
set_process_priority() {
    local pid="$1"
    local nice_value="$2"
    local current_nice=$(get_nice_value "$pid")
    local process_name=$(ps -p "$pid" -o comm= | sed 's/^-//')
    local cpu_usage=$(ps -p "$pid" -o %cpu= | tr -d ' ')
    
    # 存储原始nice值以便恢复
    echo "$pid:$current_nice" >> "$TEMP_FILE"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "将${BOLD}$process_name${RESET} (PID: $pid, CPU: ${cpu_usage}%)的优先级从$current_nice设置为$nice_value"
    else
        if sudo renice -n "$nice_value" -p "$pid" > /dev/null 2>&1; then
            echo -e "已将${BOLD}$process_name${RESET} (PID: $pid, CPU: ${cpu_usage}%)的优先级从$current_nice设置为$nice_value"
            return 0
        else
            echo -e "${RED}无法设置$process_name (PID: $pid)的优先级${RESET}" >&2
            return 1
        fi
    fi
}

# 找到并降低CPU密集型进程的优先级
lower_cpu_intensive_processes() {
    local threshold="$1"
    local nice_value="$2"
    local count=0
    
    echo -e "\n${BOLD}查找CPU密集型进程(>${threshold}% CPU使用率)...${RESET}"
    
    # 获取CPU密集型进程列表，排除我们的优先进程
    local exclude_pids="${PRIORITY_PIDS[*]}"
    exclude_pids+=" $$" # 排除此脚本
    
    # 创建一个逗号分隔的列表用于ps
    local exclude_list=""
    for pid in $exclude_pids; do
        exclude_list+="$pid,"
    done
    exclude_list=${exclude_list%,}
    
    # 查找CPU密集型进程
    local intensive_pids=()
    while read -r pid cpu_usage comm; do
        # 检查这个PID是否在我们的排除列表中
        if echo " $exclude_pids " | grep -q " $pid "; then
            continue
        fi
        
        if validate_pid "$pid" > /dev/null 2>&1; then
            intensive_pids+=("$pid")
            current_nice=$(get_nice_value "$pid")
            
            echo "$pid:$current_nice" >> "$TEMP_FILE"
            LOWERED_PIDS+=("$pid")
            LOWERED_ORIGINAL_VALUES+=("$current_nice")
            
            if [ "$DRY_RUN" = true ]; then
                echo -e "将降低${BOLD}$comm${RESET} (PID: $pid, CPU: ${cpu_usage}%)的优先级，从$current_nice到$nice_value"
            else
                if sudo renice -n "$nice_value" -p "$pid" > /dev/null 2>&1; then
                    echo -e "已降低${BOLD}$comm${RESET} (PID: $pid, CPU: ${cpu_usage}%)的优先级，从$current_nice到$nice_value"
                    count=$((count + 1))
                else
                    echo -e "${RED}无法降低$comm (PID: $pid)的优先级${RESET}" >&2
                fi
            fi
        fi
    done < <(ps -eo pid,%cpu,comm | awk -v threshold="$threshold" '$2 > threshold {print $1, $2, $3}' | sort -k2,2nr | head -n 15)
    
    if [ "$count" -eq 0 ] && [ "$DRY_RUN" = false ]; then
        echo "未找到CPU使用率超过${threshold}%的进程。"
    fi
}

# 停止非必要的后台服务
stop_background_services() {
    echo -e "\n${BOLD}停止非必要的后台服务...${RESET}"
    
    # 可以临时停止的非必要服务列表
    # 根据您的系统和需求调整此列表
    local services=(
        "com.apple.photoanalysisd"         # 照片分析
        "com.apple.spotlight"              # Spotlight索引
        "com.apple.mdworker"               # Spotlight索引工作器
        "com.apple.bird"                   # iCloud同步
        "com.apple.cloudd"                 # iCloud服务
        "com.apple.syncdefaultsd"          # 同步服务
        "com.apple.CalendarAgent"          # 日历后台代理
        "com.apple.notificationcenterui"   # 通知中心
        "com.apple.dock.extra"             # Dock额外功能
        "com.apple.AMPDevicesAgent"        # Apple移动设备服务
        "com.apple.CoreLocationAgent"      # 位置服务
        "com.apple.systemstatsd"           # 系统统计
        "com.apple.screensharing"          # 屏幕共享
        "com.apple.AirPlayXPCHelper"       # AirPlay服务
        "com.apple.iCloudHelper"           # iCloud帮助程序
        "com.apple.SafariCloudHistoryPushAgent" # Safari历史同步
    )
    
    for service in "${services[@]}"; do
        if launchctl list | grep -q "$service"; then
            if [ "$DRY_RUN" = true ]; then
                echo -e "将停止服务: $service"
            else
                if sudo launchctl unload -w /System/Library/LaunchAgents/$service.plist 2>/dev/null || \
                   sudo launchctl unload -w /System/Library/LaunchDaemons/$service.plist 2>/dev/null || \
                   launchctl unload -w ~/Library/LaunchAgents/$service.plist 2>/dev/null; then
                    echo -e "已停止服务: $service"
                    STOPPED_SERVICES+=("$service")
                else
                    echo -e "${YELLOW}无法停止服务: $service${RESET}"
                fi
            fi
        fi
    done
    
    # 特别处理一些系统进程
    local system_processes=(
        "mds"                  # Spotlight主进程
        "mds_stores"           # Spotlight存储进程
        "backupd"              # Time Machine备份
        "systemstats"          # 系统统计
    )
    
    for process in "${system_processes[@]}"; do
        pids=$(pgrep -x "$process")
        if [ -n "$pids" ]; then
            if [ "$DRY_RUN" = true ]; then
                echo -e "将降低进程优先级: $process"
            else
                for pid in $pids; do
                    current_nice=$(get_nice_value "$pid")
                    echo "$pid:$current_nice" >> "$TEMP_FILE"
                    if sudo renice -n 20 -p "$pid" > /dev/null 2>&1; then
                        echo -e "已降低进程${BOLD}$process${RESET} (PID: $pid)的优先级从$current_nice到20"
                    fi
                done
            fi
        fi
    done
    
    if [ ${#STOPPED_SERVICES[@]} -eq 0 ] && [ "$DRY_RUN" = false ]; then
        echo "没有服务被停止。"
    fi
}

# 启动之前停止的服务
start_background_services() {
    echo -e "\n${BOLD}启动之前停止的服务...${RESET}"
    
    if [ ${#STOPPED_SERVICES[@]} -eq 0 ]; then
        echo "之前没有服务被停止。"
        return
    fi
    
    for service in "${STOPPED_SERVICES[@]}"; do
        if [ "$DRY_RUN" = true ]; then
            echo -e "将启动服务: $service"
        else
            if sudo launchctl load -w /System/Library/LaunchAgents/$service.plist 2>/dev/null || \
               sudo launchctl load -w /System/Library/LaunchDaemons/$service.plist 2>/dev/null || \
               launchctl load -w ~/Library/LaunchAgents/$service.plist 2>/dev/null; then
                echo -e "已启动服务: $service"
            else
                echo -e "${YELLOW}无法启动服务: $service${RESET}"
            fi
        fi
    done
}

# 监控CPU使用情况
monitor_cpu_usage() {
    local interval="$1"
    echo -e "\n${BOLD}监控CPU使用情况(按Ctrl+C停止)...${RESET}"
    
    # 检查优先进程是否仍在运行
    for pid in "${PRIORITY_PIDS[@]}"; do
        if ! ps -p "$pid" > /dev/null; then
            echo -e "${YELLOW}警告: 优先进程$pid不再运行。${RESET}"
        fi
    done
    
    # 创建top命令的标题
    echo "优先进程的CPU使用情况:"
    
    # 使用top监控CPU使用情况
    if [ ${#PRIORITY_PIDS[@]} -gt 0 ]; then
        top -pid $(echo "${PRIORITY_PIDS[@]}" | tr ' ' ',') -stats pid,command,cpu -l 5 -s "$interval"
    else
        top -o cpu -stats pid,command,cpu -l 5 -s "$interval"
    fi
}

# 特别优化Cursor的网络I/O
optimize_cursor_network() {
    echo -e "\n${BOLD}特别优化Cursor的网络I/O...${RESET}"
    
    # 找到Cursor的相关网络进程
    local cursor_net_pids=$(sudo lsof -i | grep -i cursor | awk '{print $2}' | sort -u)
    
    if [ -z "$cursor_net_pids" ]; then
        echo "未找到Cursor的网络相关进程。"
        return
    fi
    
    echo "找到Cursor的网络相关进程:"
    
    for pid in $cursor_net_pids; do
        if validate_pid "$pid" > /dev/null 2>&1; then
            process_name=$(ps -p "$pid" -o comm= | sed 's/^-//')
            current_nice=$(get_nice_value "$pid")
            
            if [[ " ${PRIORITY_PIDS[@]} " =~ " ${pid} " ]]; then
                echo -e "${BOLD}$process_name${RESET} (PID: $pid) 已在优先列表中，跳过"
                continue
            fi
            
            # 为Cursor的网络进程设置更高的优先级
            echo "$pid:$current_nice" >> "$TEMP_FILE"
            PRIORITY_PIDS+=("$pid")
            
            if [ "$DRY_RUN" = true ]; then
                echo -e "将优化${BOLD}$process_name${RESET} (PID: $pid)的网络I/O优先级，从$current_nice到$PRIORITY_NICE_VALUE"
            else
                if sudo renice -n "$PRIORITY_NICE_VALUE" -p "$pid" > /dev/null 2>&1; then
                    echo -e "已优化${BOLD}$process_name${RESET} (PID: $pid)的网络I/O优先级，从$current_nice到$PRIORITY_NICE_VALUE"
                else
                    echo -e "${RED}无法优化$process_name (PID: $pid)的网络I/O优先级${RESET}" >&2
                fi
            fi
        fi
    done
    
    # 提高网络相关服务的优先级
    local net_services=("networkd" "mDNSResponder")
    
    for service in "${net_services[@]}"; do
        local net_pids=$(pgrep -x "$service")
        if [ -n "$net_pids" ]; then
            for pid in $net_pids; do
                current_nice=$(get_nice_value "$pid")
                echo "$pid:$current_nice" >> "$TEMP_FILE"
                
                if [ "$DRY_RUN" = true ]; then
                    echo -e "将优化网络服务${BOLD}$service${RESET} (PID: $pid)的优先级，从$current_nice到$PRIORITY_NICE_VALUE"
                else
                    if sudo renice -n "$PRIORITY_NICE_VALUE" -p "$pid" > /dev/null 2>&1; then
                        echo -e "已优化网络服务${BOLD}$service${RESET} (PID: $pid)的优先级，从$current_nice到$PRIORITY_NICE_VALUE"
                    else
                        echo -e "${RED}无法优化网络服务$service (PID: $pid)的优先级${RESET}" >&2
                    fi
                fi
            done
        fi
    done
}

# 恢复原始进程优先级
restore_original_settings() {
    echo -e "${BOLD}恢复原始进程优先级...${RESET}"
    
    if [ ! -f "$TEMP_FILE" ]; then
        echo -e "${YELLOW}未找到保存的设置以恢复。${RESET}"
        return 1
    fi
    
    while IFS=: read -r pid nice_value; do
        if ps -p "$pid" > /dev/null 2>&1; then
            process_name=$(ps -p "$pid" -o comm= | sed 's/^-//')
            current_nice=$(get_nice_value "$pid")
            
            if [ "$DRY_RUN" = true ]; then
                echo -e "将恢复${BOLD}$process_name${RESET} (PID: $pid)的优先级，从$current_nice到$nice_value"
            else
                if sudo renice -n "$nice_value" -p "$pid" > /dev/null 2>&1; then
                    echo -e "已恢复${BOLD}$process_name${RESET} (PID: $pid)的优先级，从$current_nice到$nice_value"
                else
                    echo -e "${RED}无法恢复$process_name (PID: $pid)的优先级${RESET}" >&2
                fi
            fi
        else
            echo -e "${YELLOW}PID为$pid的进程不再存在，跳过恢复。${RESET}"
        fi
    done < "$TEMP_FILE"
    
    # 启动之前停止的服务
    start_background_services
    
    # 移除临时文件
    if [ "$DRY_RUN" = false ]; then
        rm -f "$TEMP_FILE"
        echo -e "${GREEN}原始设置已恢复。${RESET}"
    fi
}

# 自动查找目标进程
auto_find_target_processes() {
    echo -e "\n${BOLD}自动查找目标开发环境进程...${RESET}"
    
    # 目标进程模式
    local targets=(
        "rust-analyzer"
        "rustc"
        "cargo"
        "Cursor"
        "cursor"
        "openai"
        "OpenAI"
        "ChatGPT"
        "chatgpt"
        "Warp"
        "warp"
    )
    
    local found_pids=()
    
    for target in "${targets[@]}"; do
        echo -e "查找${BOLD}$target${RESET}相关进程..."
        local pids=$(find_pid_by_name "$target" 2>/dev/null)
        
        if [ -n "$pids" ]; then
            for pid in $pids; do
                if validate_pid "$pid" > /dev/null 2>&1; then
                    process_name=$(ps -p "$pid" -o comm= | sed 's/^-//')
                    cpu_usage=$(ps -p "$pid" -o %cpu= | tr -d ' ')
                    
                    # 如果进程CPU使用率大于0.5%，考虑它为活动进程
                    if (( $(echo "$cpu_usage > 0.5" | bc -l) )); then
                        echo -e "找到活动进程: ${BOLD}$process_name${RESET} (PID: $pid, CPU: ${cpu_usage}%)"
                        found_pids+=("$pid")
                    fi
                fi
            done
        fi
    done
    
    # 去重
    found_pids=($(echo "${found_pids[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))
    
    # 如果找到的进程超过5个，只选择CPU使用率最高的5个
    if [ ${#found_pids[@]} -gt 5 ]; then
        echo -e "${YELLOW}找到超过5个目标进程，只选择CPU使用率最高的5个${RESET}"
        
        # 创建临时文件来存储PID和CPU使用率
        local cpu_temp_file="/tmp/cpu_usage_$$.tmp"
        for pid in "${found_pids[@]}"; do
            cpu_usage=$(ps -p "$pid" -o %cpu= | tr -d ' ')
            echo "$pid $cpu_usage" >> "$cpu_temp_file"
        done
        
        # 按CPU使用率排序并获取前5个PID
        found_pids=($(sort -k2,2nr "$cpu_temp_file" | head -n 5 | awk '{print $1}'))
        rm -f "$cpu_temp_file"
    fi
    
    if [ ${#found_pids[@]} -eq 0 ]; then
        echo -e "${YELLOW}未找到任何目标进程。${RESET}"
        return 1
    fi
    
    # 更新全局变量
    PRIORITY_PIDS=("${found_pids[@]}")
    echo -e "\n${GREEN}已自动找到${#PRIORITY_PIDS[@]}个目标进程${RESET}"
    
    # 显示找到的进程
    echo -e "\n${BOLD}将优先处理以下进程:${RESET}"
    for pid in "${PRIORITY_PIDS[@]}"; do
        process_name=$(ps -p "$pid" -o comm= | sed 's/^-//')
        cpu_usage=$(ps -p "$pid" -o %cpu= | tr -d ' ')
        echo -e "- ${BOLD}$process_name${RESET} (PID: $pid, CPU: ${cpu_usage}%)"
    done
    
    return 0
}

# 解析命令行参数
parse_arguments() {
    LOWER_OTHERS=false
    STOP_SERVICES=false
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                print_usage
                exit 0
                ;;
            -v|--priority-value)
                if [[ $# -lt 2 ]] || ! [[ "$2" =~ ^-?[0-9]+$ ]]; then
                    echo -e "${RED}错误: -v选项需要一个数值。${RESET}" >&2
                    exit 1
                fi
                PRIORITY_NICE_VALUE="$2"
                # 验证nice值范围
                if [ "$PRIORITY_NICE_VALUE" -lt -20 ] || [ "$PRIORITY_NICE_VALUE" -gt 0 ]; then
                    echo -e "${RED}错误: 优先级nice值必须在-20到0之间。${RESET}" >&2
                    exit 1
                fi
                shift 2
                ;;
            -l|--lower-others)
                LOWER_OTHERS=true
                shift
                ;;
            -t|--threshold)
                if [[ $# -lt 2 ]] || ! [[ "$2" =~ ^[0-9]+$ ]]; then
                    echo -e "${RED}错误: -t选项需要一个数值。${RESET}" >&2
                    exit 1
                fi
                CPU_THRESHOLD="$2"
                shift 2
                ;;
            -c|--lower-value)
                if [[ $# -lt 2 ]] || ! [[ "$2" =~ ^[0-9]+$ ]]; then
                    echo -e "${RED}错误: -c选项需要一个数值。${RESET}" >&2
                    exit 1
                fi
                LOWER_NICE_VALUE="$2"
                # 验证nice值范围
                if [ "$LOWER_NICE_VALUE" -lt 0 ] || [ "$LOWER_NICE_VALUE" -gt 20 ]; then
                    echo -e "${RED}错误: 降低优先级的nice值必须在0到20之间。${RESET}" >&2
                    exit 1
                fi
                shift 2
                ;;
            -s|--stop-services)
                STOP_SERVICES=true
                shift
                ;;
            -m|--monitor)
                MONITOR_ENABLED=true
                if [[ $# -gt 1 ]] && [[ "$2" =~ ^[0-9]+$ ]]; then
                    MONITOR_INTERVAL="$2"
                    shift 2
                else
                    shift
                fi
                ;;
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -r|--restore)
                RESTORE_MODE=true
                shift
                ;;
            -n|--no-auto)
                AUTO_MODE=false
                shift
                ;;
            -p|--pids)
                AUTO_MODE=false
                shift
                while [[ $# -gt 0 ]] && ! [[ "$1" =~ ^- ]]; do
                    if validate_pid "$1"; then
                        PRIORITY_PIDS+=("$1")
                    fi
                    shift
                done
                if [ ${#PRIORITY_PIDS[@]} -eq 0 ]; then
                    echo -e "${RED}错误: 使用-p选项未提供有效的PID。${RESET}" >&2
                    exit 1
                fi
                ;;
            --no-cursor-network)
                FOCUS_CURSOR_NETWORK=false
                shift
                ;;
            *)
                echo -e "${RED}错误: 未知选项: $1${RESET}" >&2
                print_usage
                exit 1
                ;;
        esac
    done
}

# 主函数
main() {
    echo -e "${BOLD}macOS开发环境CPU优化器${RESET}"
    echo -e "版本1.0 - $(date '+%Y-%m-%d')"
    echo -e "目标: 优化rust-analyzer, rustc, Cursor编辑器, OpenAI和Warp终端的CPU资源分配"
    echo
    
    # 检查是否处于恢复模式
    if [ "$RESTORE_MODE" = true ]; then
        check_privileges
        restore_original_settings
        exit 0
    fi
    
    # 自动查找目标进程
    if [ "$AUTO_MODE" = true ]; then
        auto_find_target_processes
    fi
    
    # 显示配置
    echo -e "\n${BOLD}配置:${RESET}"
    echo -e "- 干运行模式: ${DRY_RUN}"
    echo -e "- 优先级nice值: ${PRIORITY_NICE_VALUE}"
    echo -e "- 降低其他进程优先级: ${LOWER_OTHERS}"
    if [ "$LOWER_OTHERS" = true ]; then
        echo -e "- CPU阈值: ${CPU_THRESHOLD}%"
        echo -e "- 降低优先级nice值: ${LOWER_NICE_VALUE}"
    fi
    echo -e "- 停止非必要服务: ${STOP_SERVICES}"
    echo -e "- 特别优化Cursor网络I/O: ${FOCUS_CURSOR_NETWORK}"
    
    # 检查是否需要root权限
    check_privileges
    
    # 存储原始值以便稍后恢复
    [ "$DRY_RUN" = false ] && > "$TEMP_FILE"
    
    # 设置指定进程的优先级
    if [ ${#PRIORITY_PIDS[@]} -gt 0 ]; then
        echo -e "\n${BOLD}设置进程优先级...${RESET}"
        for pid in "${PRIORITY_PIDS[@]}"; do
            ORIGINAL_NICE_VALUES+=($(get_nice_value "$pid"))
            set_process_priority "$pid" "$PRIORITY_NICE_VALUE"
        done
    else
        echo -e "\n${YELLOW}警告: 未指定任何进程进行优先处理。${RESET}"
        echo -e "您可以使用-p选项指定PID或让脚本自动查找目标进程。"
    fi
    
    # 如果需要，特别优化Cursor的网络I/O
    if [ "$FOCUS_CURSOR_NETWORK" = true ]; then
        optimize_cursor_network
    fi
    
    # 如果请求，降低CPU密集型进程的优先级
    if [ "$LOWER_OTHERS" = true ]; then
        lower_cpu_intensive_processes "$CPU_THRESHOLD" "$LOWER_NICE_VALUE"
    fi
    
    # 如果请求，停止后台服务
    if [ "$STOP_SERVICES" = true ]; then
        stop_background_services
    fi
    
    # 如果请求，监控CPU使用情况
    if [ "$MONITOR_ENABLED" = true ]; then
        monitor_cpu_usage "$MONITOR_INTERVAL"
    fi
    
    echo -e "\n${GREEN}CPU优化完成。${RESET}"
    echo -e "- 优化适用于${BOLD}当天的开发会话${RESET}"
    echo -e "- 重启电脑将恢复正常系统配置"
    echo -e "- 要手动恢复原始设置，运行: $0 -r"
}

# 解析命令行参数
parse_arguments "$@"

# 运行主函数
main

