#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}==================================${NC}"
echo -e "${BLUE}ZKPersona Authentication API Tests${NC}"
echo -e "${BLUE}==================================${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check Python
if ! command_exists python3; then
    echo -e "${RED}Error: Python 3 is not installed${NC}"
    exit 1
fi

# Check if requests module is installed
if ! python3 -c "import requests" 2>/dev/null; then
    echo -e "${YELLOW}Installing requests module...${NC}"
    pip3 install requests
fi

# Check if server is running
echo -e "\n${YELLOW}Checking if server is running...${NC}"
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health | grep -q "200\|404"; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not running${NC}"
    echo -e "${YELLOW}Starting server in background...${NC}"
    
    # Start the server in background
    cd "$(dirname "$0")"
    cargo run --bin web_server > server.log 2>&1 &
    SERVER_PID=$!
    
    echo -e "${YELLOW}Waiting for server to start...${NC}"
    sleep 5
    
    # Check again
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health | grep -q "200\|404"; then
        echo -e "${GREEN}✓ Server started successfully${NC}"
    else
        echo -e "${RED}✗ Failed to start server${NC}"
        echo -e "${YELLOW}Check server.log for errors${NC}"
        exit 1
    fi
fi

# Run the Python test script
echo -e "\n${BLUE}Running authentication tests...${NC}"
python3 test_auth_api.py

# If we started the server, ask if user wants to keep it running
if [ ! -z "$SERVER_PID" ]; then
    echo -e "\n${YELLOW}Server is running with PID: $SERVER_PID${NC}"
    read -p "Keep server running? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Stopping server...${NC}"
        kill $SERVER_PID
        echo -e "${GREEN}✓ Server stopped${NC}"
    else
        echo -e "${GREEN}✓ Server will continue running${NC}"
        echo -e "${YELLOW}To stop later, run: kill $SERVER_PID${NC}"
    fi
fi