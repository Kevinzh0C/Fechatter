#!/bin/bash

# Backend Optimization Scripts - No Code Changes Required
# 后端优化脚本 - 无需修改代码

echo "🚀 Fechatter Backend Optimization Script"
echo "========================================"
echo ""

# Server connection info
SERVER_IP="45.77.178.85"
SERVER_PORT="8080"
CONTAINER_NAME="fechatter_backend"

echo "📍 Target Server: $SERVER_IP:$SERVER_PORT"
echo ""

# ========================================
# 1. DATABASE OPTIMIZATION (Most Important)
# ========================================
echo "1️⃣ DATABASE OPTIMIZATION COMMANDS"
echo "================================="
echo ""
echo "Connect to your database and run these commands:"
echo ""

cat << 'EOF'
-- Check existing indexes
SHOW INDEXES FROM messages;

-- Create critical indexes for message queries
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_chat_created ON messages(chat_id, created_at DESC);

-- Create index for user lookups
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id);

-- Create indexes for chat table
CREATE INDEX IF NOT EXISTS idx_chats_workspace_id ON chats(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats(updated_at DESC);

-- Create indexes for chat_members table
CREATE INDEX IF NOT EXISTS idx_chat_members_chat_id ON chat_members(chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_members_user_id ON chat_members(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_members_chat_user ON chat_members(chat_id, user_id);

-- Analyze tables to update statistics
ANALYZE TABLE messages;
ANALYZE TABLE chats;
ANALYZE TABLE chat_members;

-- Check query performance after indexes
EXPLAIN SELECT * FROM messages WHERE chat_id = 6 ORDER BY created_at DESC LIMIT 15;
EOF

echo ""
echo "Expected improvement: 8s -> 500ms-1s (16x faster)"
echo ""

# ========================================
# 2. DOCKER OPTIMIZATION
# ========================================
echo "2️⃣ DOCKER OPTIMIZATION COMMANDS"
echo "==============================="
echo ""
echo "SSH into your server and run these commands:"
echo ""

cat << 'EOF'
# Check current container resources
docker stats fechatter_backend --no-stream

# Update container resource limits (without restart)
docker update fechatter_backend \
  --cpus="2" \
  --memory="2g" \
  --memory-swap="4g"

# Check Docker logs for errors
docker logs fechatter_backend --tail 100 | grep -i error

# Monitor real-time performance
docker stats fechatter_backend

# Alternative: Create optimized docker-compose.yml
cat > docker-compose-optimized.yml << 'COMPOSE'
version: '3.8'
services:
  backend:
    image: fechatter_backend
    ports:
      - "8080:6688"
    environment:
      - RUST_LOG=info
      - DATABASE_POOL_SIZE=20
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
COMPOSE

# Apply with: docker-compose -f docker-compose-optimized.yml up -d
EOF

echo ""
echo "Expected improvement: Better resource utilization, reduced overhead"
echo ""

# ========================================
# 3. SERVER OPTIMIZATION
# ========================================
echo "3️⃣ SERVER OPTIMIZATION COMMANDS"
echo "==============================="
echo ""
echo "Run these on your Vultr VPS:"
echo ""

cat << 'EOF'
# Increase system limits
sudo bash -c 'cat >> /etc/security/limits.conf << LIMITS
* soft nofile 65535
* hard nofile 65535
* soft nproc 65535
* hard nproc 65535
LIMITS'

# Optimize network settings
sudo bash -c 'cat >> /etc/sysctl.conf << SYSCTL
# Network optimizations
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_max_tw_buckets = 5000
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_congestion_control = bbr
net.core.netdev_max_backlog = 5000

# Memory optimizations
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
SYSCTL'

# Apply settings
sudo sysctl -p

# Install monitoring tools
sudo apt-get update
sudo apt-get install -y htop iotop nethogs

# Monitor system performance
htop  # CPU and memory usage
iotop # Disk I/O
nethogs # Network usage by process
EOF

echo ""
echo "Expected improvement: Better network handling, reduced latency"
echo ""

# ========================================
# 4. DATABASE CONNECTION TUNING
# ========================================
echo "4️⃣ DATABASE CONNECTION TUNING"
echo "============================="
echo ""
echo "MySQL/MariaDB configuration (my.cnf):"
echo ""

cat << 'EOF'
[mysqld]
# Connection pool settings
max_connections = 200
max_connect_errors = 1000000

# Query cache (if using older MySQL)
query_cache_type = 1
query_cache_size = 128M
query_cache_limit = 2M

# InnoDB optimizations
innodb_buffer_pool_size = 1G  # Set to 70% of available RAM
innodb_log_file_size = 256M
innodb_flush_log_at_trx_commit = 2
innodb_flush_method = O_DIRECT

# Temp table and join buffer
tmp_table_size = 64M
max_heap_table_size = 64M
join_buffer_size = 8M

# Restart MySQL after changes:
# sudo systemctl restart mysql
EOF

echo ""
echo "Expected improvement: Faster query execution, better caching"
echo ""

# ========================================
# 5. QUICK MONITORING COMMANDS
# ========================================
echo "5️⃣ MONITORING COMMANDS"
echo "===================="
echo ""

cat << 'EOF'
# Monitor Docker container
watch -n 1 'docker stats fechatter_backend --no-stream'

# Check slow queries (MySQL)
SHOW FULL PROCESSLIST;
SHOW STATUS LIKE 'Slow_queries';

# Monitor system resources
vmstat 1 10  # System statistics
iostat -x 1  # Disk I/O statistics

# Test API response time
time curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://45.77.178.85:8080/api/chat/6/messages

# Continuous latency test
while true; do
  echo -n "$(date): "
  curl -o /dev/null -s -w "%{time_total}s\n" \
    http://45.77.178.85:8080/api/health
  sleep 1
done
EOF

echo ""
echo "========================================"
echo "📊 EXPECTED OVERALL IMPROVEMENT"
echo "========================================"
echo ""
echo "Before optimization: 8234ms"
echo "After database indexes: ~500-1000ms (8-16x faster)"
echo "After Docker optimization: ~400-800ms (10-20x faster)"
echo "After full optimization: ~200-500ms (16-40x faster)"
echo ""
echo "🎯 Most important: CREATE THE DATABASE INDEXES FIRST!"
echo "" 