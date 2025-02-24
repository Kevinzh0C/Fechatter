#!/bin/bash

# Start Fechatter Frontend with Bot Mock API
# This script starts both the frontend dev server and the bot mock API server

set -e

echo "🚀 Starting Fechatter with Bot Mock API..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to cleanup background processes
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down services...${NC}"
    if [ ! -z "$BOT_PID" ]; then
        kill $BOT_PID 2>/dev/null || true
        echo -e "${GREEN}✅ Bot Mock API stopped${NC}"
    fi
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
        echo -e "${GREEN}✅ Frontend dev server stopped${NC}"
    fi
    exit 0
}

# Trap ctrl+c and cleanup
trap cleanup SIGINT SIGTERM

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo -e "${RED}❌ Error: Must be run from fechatter_frontend directory${NC}"
    exit 1
fi

# Install bot mock dependencies if needed
if [ ! -d "server/node_modules" ]; then
    echo -e "${BLUE}📦 Installing bot mock API dependencies...${NC}"
    cd server
    npm install --silent
    cd ..
    echo -e "${GREEN}✅ Bot mock dependencies installed${NC}"
fi

# Start bot mock API server in background
echo -e "${BLUE}🤖 Starting Bot Mock API server on port 3001...${NC}"
cd server
node bot-mock.js &
BOT_PID=$!
cd ..

# Wait a moment for bot server to start
sleep 2

# Check if bot server is running
if curl -s http://localhost:3001/health > /dev/null; then
    echo -e "${GREEN}✅ Bot Mock API server is running on http://localhost:3001${NC}"
else
    echo -e "${RED}❌ Failed to start Bot Mock API server${NC}"
    cleanup
    exit 1
fi

# Start frontend development server
echo -e "${BLUE}🌐 Starting Frontend development server...${NC}"

# Parse command line arguments for port
PORT=5177
while [[ $# -gt 0 ]]; do
    case $1 in
        --port)
            PORT="$2"
            shift 2
            ;;
        *)
            echo -e "${YELLOW}Unknown option: $1${NC}"
            shift
            ;;
    esac
done

echo -e "${GREEN}📋 Services starting:${NC}"
echo -e "   ${BLUE}🤖 Bot Mock API:${NC} http://localhost:3001"
echo -e "   ${BLUE}🌐 Frontend:${NC} http://localhost:${PORT}"
echo -e "   ${YELLOW}Press Ctrl+C to stop all services${NC}"

# Start frontend with specified port
yarn dev --port $PORT &
FRONTEND_PID=$!

# Wait for frontend process
wait $FRONTEND_PID 