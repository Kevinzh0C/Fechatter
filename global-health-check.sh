#!/bin/bash

# Comprehensive global health check script for all Fechatter services
# Includes application services, databases, and infrastructure components

check_service() {
    local service=$1
    local port=$2
    local endpoint=${3:-/health}
    
    local start_time=$(date +%s%3N)
    local response=$(curl -s -m 5 "http://localhost:$port$endpoint" 2>/dev/null)
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))
    
    if [[ $exit_code -eq 0 ]] && [[ -n "$response" ]]; then
        echo "\"$service\":{\"status\":\"healthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:$port$endpoint\"}"
    else
        echo "\"$service\":{\"status\":\"unhealthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:$port$endpoint\",\"error\":\"Connection failed\"}"
    fi
}

check_postgres() {
    local start_time=$(date +%s%3N)
    local response=$(PGPASSWORD=fechatter_password psql -h localhost -p 5432 -U fechatter -d fechatter -c "SELECT 1;" -t 2>/dev/null)
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))
    
    if [[ $exit_code -eq 0 ]]; then
        echo "\"postgres\":{\"status\":\"healthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:5432\"}"
    else
        echo "\"postgres\":{\"status\":\"unhealthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:5432\",\"error\":\"Database connection failed\"}"
    fi
}

check_redis() {
    local start_time=$(date +%s%3N)
    local response=$(redis-cli -h localhost -p 6379 -a fechatter_redis_pass ping 2>/dev/null)
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))
    
    if [[ $exit_code -eq 0 ]] && [[ "$response" == "PONG" ]]; then
        echo "\"redis\":{\"status\":\"healthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:6379\"}"
    else
        echo "\"redis\":{\"status\":\"unhealthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:6379\",\"error\":\"Redis connection failed\"}"
    fi
}

check_nats() {
    local start_time=$(date +%s%3N)
    local response=$(curl -s -m 3 "http://localhost:8222/varz" 2>/dev/null)
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local latency=$((end_time - start_time))
    
    if [[ $exit_code -eq 0 ]] && [[ -n "$response" ]]; then
        echo "\"nats\":{\"status\":\"healthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:4222\"}"
    else
        echo "\"nats\":{\"status\":\"unhealthy\",\"latency_ms\":$latency,\"endpoint\":\"localhost:4222\",\"error\":\"NATS connection failed\"}"
    fi
}

# Get current timestamp
timestamp=$(date +%s%3N)

# Check application services
fechatter_status=$(check_service "fechatter_server" "6688")
analytics_status=$(check_service "analytics_server" "6690") 
notify_status=$(check_service "notify_server" "6687")
bot_status=$(check_service "bot_server" "6686")

# Check infrastructure services
postgres_status=$(check_postgres)
redis_status=$(check_redis)
nats_status=$(check_nats)
meilisearch_status=$(check_service "meilisearch" "7700" "/health")
clickhouse_status=$(check_service "clickhouse" "8123" "/ping")

# Determine overall status
healthy_services=0
total_services=9

for status in "$fechatter_status" "$analytics_status" "$notify_status" "$bot_status" "$postgres_status" "$redis_status" "$nats_status" "$meilisearch_status" "$clickhouse_status"; do
    if [[ "$status" == *"healthy"* ]]; then
        ((healthy_services++))
    fi
done

if [[ $healthy_services -eq $total_services ]]; then
    overall_status="healthy"
elif [[ $healthy_services -gt $((total_services / 2)) ]]; then
    overall_status="degraded"
else
    overall_status="unhealthy"
fi

# Output JSON
echo "{"
echo "  \"status\": \"$overall_status\","
echo "  \"gateway\": \"nginx-proxy\","
echo "  \"version\": \"1.0.0\","
echo "  \"timestamp\": $timestamp,"
echo "  \"summary\": {"
echo "    \"healthy_services\": $healthy_services,"
echo "    \"total_services\": $total_services,"
echo "    \"availability\": \"$((healthy_services * 100 / total_services))%\""
echo "  },"
echo "  \"application_services\": {"
echo "    $fechatter_status,"
echo "    $analytics_status,"
echo "    $notify_status,"
echo "    $bot_status"
echo "  },"
echo "  \"infrastructure_services\": {"
echo "    $postgres_status,"
echo "    $redis_status,"
echo "    $nats_status,"
echo "    $meilisearch_status,"
echo "    $clickhouse_status"
echo "  }"
echo "}" 