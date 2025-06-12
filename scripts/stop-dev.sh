#!/bin/bash

# Fechatter Development Environment Stop Script
# Gracefully stops all running services

echo "🛑 Stopping Fechatter Development Environment"
echo "============================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to stop service by PID file
stop_service() {
  local service_name=$1
  local pid_file="logs/${service_name}.pid"
  
  if [ -f "$pid_file" ]; then
    local pid=$(cat "$pid_file")
    
    if kill -0 "$pid" 2>/dev/null; then
      echo -e "${BLUE}🛑 Stopping $service_name (PID: $pid)...${NC}"
      
      # Try graceful shutdown first
      kill -TERM "$pid" 2>/dev/null
      
      # Wait for graceful shutdown
      local attempts=0
      while [ $attempts -lt 10 ] && kill -0 "$pid" 2>/dev/null; do
        sleep 1
        attempts=$((attempts + 1))
      done
      
      # Force kill if still running
      if kill -0 "$pid" 2>/dev/null; then
        echo -e "${YELLOW}⚠️ Force stopping $service_name...${NC}"
        kill -KILL "$pid" 2>/dev/null
      fi
      
      echo -e "${GREEN}✅ $service_name stopped${NC}"
    else
      echo -e "${YELLOW}⚠️ $service_name was not running${NC}"
    fi
    
    # Remove PID file
    rm -f "$pid_file"
  else
    echo -e "${YELLOW}⚠️ No PID file found for $service_name${NC}"
  fi
}

# Function to stop service by port
stop_by_port() {
  local port=$1
  local service_name=$2
  
  local pid=$(lsof -t -i:$port 2>/dev/null)
  if [ -n "$pid" ]; then
    echo -e "${BLUE}🛑 Found process on port $port (PID: $pid) - stopping $service_name...${NC}"
    kill -TERM "$pid" 2>/dev/null
    sleep 2
    
    # Force kill if still running
    if kill -0 "$pid" 2>/dev/null; then
      kill -KILL "$pid" 2>/dev/null
    fi
    
    echo -e "${GREEN}✅ Process on port $port stopped${NC}"
  fi
}

# Stop services in reverse order
echo -e "${BLUE}🎯 Stopping services...${NC}"

# Stop by PID files first
stop_service "frontend"
stop_service "gateway" 
stop_service "bot_server"
stop_service "analytics_server"
stop_service "notify_server"
stop_service "fechatter_server"

echo ""

# Stop any remaining processes by port
echo -e "${BLUE}🔍 Checking for remaining processes...${NC}"
stop_by_port 1420 "frontend"
stop_by_port 8080 "gateway"
# Note: bot_server is now a NATS subscriber, no HTTP port to check
stop_by_port 6690 "analytics_server"
stop_by_port 6687 "notify_server"
stop_by_port 6688 "fechatter_server"

# Clean up log files if requested
if [ "$1" = "--clean" ] || [ "$1" = "-c" ]; then
  echo ""
  echo -e "${BLUE}🧹 Cleaning up log files...${NC}"
  
  if [ -d "logs" ]; then
    rm -rf logs/
    echo -e "${GREEN}✅ Log files cleaned${NC}"
  fi
fi

echo ""
echo -e "${GREEN}🎉 All services stopped successfully!${NC}"
echo ""
echo -e "${YELLOW}💡 Tips:${NC}"
echo "  • Use 'scripts/start-dev.sh' to start the environment again"
echo "  • Use 'scripts/stop-dev.sh --clean' to also remove log files"
echo ""
echo -e "${BLUE}🔍 Port Status:${NC}"
for port in 1420 8080 6690 6687 6688; do
  if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "  ${RED}❌ Port $port: still in use${NC}"
  else
    echo -e "  ${GREEN}✅ Port $port: available${NC}"
  fi
done 