#!/bin/bash

# ZKPersona Authentication API Test Script using cURL
# This script tests all authentication endpoints

BASE_URL="http://localhost:8080/api/v1/zkpersona"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test wallet data
WALLET_ADDRESS="0x1234567890abcdef1234567890abcdef12345678"
PUBLIC_KEY="dGVzdF9wdWJsaWNfa2V5XzEyMzQ1Njc4OTBhYmNkZWY="
SIGNATURE="AH4gbmV3IGNvbGxlY3Rpb24gd2l0aCByZWR1Y2VkIGdhcyBmZWVzLiBUaGUgY29sbGVjdGlvbiB3aWxsIGNvbnNpc3Qgb2YgMTAwMCB1bmlxdWUgTkZUcyB3aXRoIGRpZmZlcmVudCByYXJpdHkgbGV2ZWxzLgAAAAAAAAAAAAAAAAA="

echo -e "${BLUE}==================================${NC}"
echo -e "${BLUE}ZKPersona Auth API Tests (cURL)${NC}"
echo -e "${BLUE}==================================${NC}"

# Test 1: Generate Nonce
echo -e "\n${YELLOW}Test 1: Generate Nonce${NC}"
echo -e "${BLUE}POST ${BASE_URL}/auth/nonce${NC}"

curl -X POST "${BASE_URL}/auth/nonce" \
  -H "Content-Type: application/json" \
  -d "{
    \"wallet_address\": \"${WALLET_ADDRESS}\"
  }" \
  -w "\n\nStatus: %{http_code}\n" \
  -s | jq '.' 2>/dev/null || cat

# Test 2: Login with Wallet
echo -e "\n${YELLOW}Test 2: Login with Wallet${NC}"
echo -e "${BLUE}POST ${BASE_URL}/auth/login${NC}"

# Save cookies to file for later use
curl -X POST "${BASE_URL}/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"wallet_address\": \"${WALLET_ADDRESS}\",
    \"signature\": \"${SIGNATURE}\",
    \"public_key\": \"${PUBLIC_KEY}\"
  }" \
  -c cookies.txt \
  -w "\n\nStatus: %{http_code}\n" \
  -s | jq '.' 2>/dev/null || cat

# Test 3: Access Protected Endpoint (without auth)
echo -e "\n${YELLOW}Test 3: Access Protected Endpoint (No Auth)${NC}"
echo -e "${BLUE}POST ${BASE_URL}/generate-proof${NC}"

curl -X POST "${BASE_URL}/generate-proof" \
  -H "Content-Type: application/json" \
  -d '{
    "behavior_input": {
      "user_actions": [
        {"action": "page_view", "page": "home", "duration": 5.2},
        {"action": "click", "element": "signup_button"}
      ],
      "session_info": {
        "browser": "Chrome",
        "device": "Desktop"
      }
    },
    "session_id": "test_session_123"
  }' \
  -w "\n\nStatus: %{http_code}\n" \
  -s | jq '.' 2>/dev/null || cat

# Test 4: Access Protected Endpoint (with auth)
echo -e "\n${YELLOW}Test 4: Access Protected Endpoint (With Auth)${NC}"
echo -e "${BLUE}POST ${BASE_URL}/generate-proof${NC}"

curl -X POST "${BASE_URL}/generate-proof" \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "behavior_input": {
      "user_actions": [
        {"action": "page_view", "page": "home", "duration": 5.2},
        {"action": "click", "element": "signup_button"}
      ],
      "session_info": {
        "browser": "Chrome",
        "device": "Desktop"
      }
    },
    "session_id": "test_session_456"
  }' \
  -w "\n\nStatus: %{http_code}\n" \
  -s | jq '.' 2>/dev/null || cat

# Test 5: Verify Proof
echo -e "\n${YELLOW}Test 5: Verify Proof${NC}"
echo -e "${BLUE}POST ${BASE_URL}/verify${NC}"

curl -X POST "${BASE_URL}/verify" \
  -H "Content-Type: application/json" \
  -d '{
    "proof_data": "bW9ja19wcm9vZl9kYXRh",
    "verification_key": "bW9ja192ZXJpZmljYXRpb25fa2V5",
    "public_signals": {
      "score": 85.5,
      "score_range": [0, 100],
      "behavior_hash": "abc123def456"
    }
  }' \
  -w "\n\nStatus: %{http_code}\n" \
  -s | jq '.' 2>/dev/null || cat

# Clean up
rm -f cookies.txt

echo -e "\n${GREEN}Tests completed!${NC}"