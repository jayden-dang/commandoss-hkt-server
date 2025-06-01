#!/bin/bash

# Comprehensive API Test Script for JDBlog Server
# Tests all available routes including ZK-Persona, Analytics, Vulnerabilities, Patches, Developers, SUI, and GitHub

BASE_URL="http://localhost:8080"
API_V1="${BASE_URL}/api/v1"
ZKPERSONA_URL="${API_V1}/zkpersona"
ANALYTICS_URL="${API_V1}/analytics"
VULNERABILITIES_URL="${API_V1}/vulnerabilities"
PATCHES_URL="${API_V1}/patches"
DEVELOPERS_URL="${API_V1}/developers"
SUI_URL="${API_V1}/sui"
GITHUB_URL="${API_V1}/github"
RPC_URL="${BASE_URL}/api"

echo "🧪 Testing All JDBlog API Routes"
echo "================================"
echo "Base URL: $BASE_URL"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results tracking
PASSED=0
FAILED=0

# Helper function to test endpoint
test_endpoint() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="$5"
    
    echo -e "${BLUE}📝 Testing: $name${NC}"
    echo "   $method $url"
    
    if [ "$method" = "GET" ]; then
        RESPONSE=$(curl -s -w "\n%{http_code}" "$url")
    else
        RESPONSE=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi
    
    # Extract HTTP status code (last line)
    HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    # Extract response body (all but last line)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "$expected_status" ]; then
        echo -e "   ${GREEN}✅ PASS${NC} (Status: $HTTP_CODE)"
        ((PASSED++))
        if [ -n "$BODY" ] && command -v jq >/dev/null 2>&1; then
            echo "$BODY" | jq '.' 2>/dev/null | head -5
        else
            echo "$BODY" | head -3
        fi
    else
        echo -e "   ${RED}❌ FAIL${NC} (Expected: $expected_status, Got: $HTTP_CODE)"
        ((FAILED++))
        echo "   Response: $BODY"
    fi
    echo ""
}

# ===============================
# ZKPERSONA AUTH TESTS
# ===============================
echo -e "${YELLOW}🔐 ZKPERSONA AUTH TESTS${NC}"
echo "========================="

# Test generate nonce (POST)
NONCE_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${ZKPERSONA_URL}/auth/nonce" \
    -H "Content-Type: application/json" \
    -d '{
        "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    }')

NONCE_HTTP_CODE=$(echo "$NONCE_RESPONSE" | tail -1)
NONCE_BODY=$(echo "$NONCE_RESPONSE" | sed '$d')

echo -e "${BLUE}📝 Testing: Generate Nonce${NC}"
echo "   POST ${ZKPERSONA_URL}/auth/nonce"
if [ "$NONCE_HTTP_CODE" = "200" ]; then
    echo -e "   ${GREEN}✅ PASS${NC} (Status: $NONCE_HTTP_CODE)"
    ((PASSED++))
    echo "$NONCE_BODY" | jq '.' 2>/dev/null || echo "$NONCE_BODY"
    NONCE=$(echo "$NONCE_BODY" | jq -r '.data.nonce // empty' 2>/dev/null)
else
    echo -e "   ${RED}❌ FAIL${NC} (Expected: 200, Got: $NONCE_HTTP_CODE)"
    ((FAILED++))
    echo "   Error Response:"
    echo "$NONCE_BODY" | jq '.' 2>/dev/null || echo "$NONCE_BODY"
fi
echo ""

# Test login (POST) - using mock signature
if [ -n "$NONCE" ]; then
    LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${ZKPERSONA_URL}/auth/login" \
        -H "Content-Type: application/json" \
        -d '{
            "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "signature": "mock_signature_for_testing",
            "public_key": "mock_public_key_for_testing"
        }')

    LOGIN_HTTP_CODE=$(echo "$LOGIN_RESPONSE" | tail -1)
    LOGIN_BODY=$(echo "$LOGIN_RESPONSE" | sed '$d')

    echo -e "${BLUE}📝 Testing: Login${NC}"
    echo "   POST ${ZKPERSONA_URL}/auth/login"
    if [ "$LOGIN_HTTP_CODE" = "200" ]; then
        echo -e "   ${GREEN}✅ PASS${NC} (Status: $LOGIN_HTTP_CODE)"
        ((PASSED++))
        echo "$LOGIN_BODY" | jq '.' 2>/dev/null || echo "$LOGIN_BODY"
        JWT_TOKEN=$(echo "$LOGIN_BODY" | jq -r '.data.access_token // empty' 2>/dev/null)
    else
        echo -e "   ${RED}❌ FAIL${NC} (Expected: 200, Got: $LOGIN_HTTP_CODE)"
        ((FAILED++))
        echo "   Response: $LOGIN_BODY"
    fi
    echo ""
else
    echo -e "${RED}⚠️  Skipping Login - no nonce from previous step${NC}"
    echo ""
fi

# ===============================
# ANALYTICS SERVICE TESTS
# ===============================
echo -e "${YELLOW}📊 ANALYTICS SERVICE TESTS${NC}"
echo "=========================="

# Get metrics (GET)
test_endpoint "Get Metrics" "GET" "${ANALYTICS_URL}/metrics" "" "200"

# Get top repositories (GET)
test_endpoint "Get Top Repositories" "GET" "${ANALYTICS_URL}/top-repositories?limit=10&time_period=week" "" "200"

# Get activity trends (GET)
test_endpoint "Get Activity Trends" "GET" "${ANALYTICS_URL}/trends/activity?period=daily&days=7" "" "200"

# Get vulnerability trends (GET)
test_endpoint "Get Vulnerability Trends" "GET" "${ANALYTICS_URL}/trends/vulnerabilities?period=weekly&weeks=4" "" "200"

# ===============================
# VULNERABILITY SERVICE TESTS
# ===============================
echo -e "${YELLOW}🐛 VULNERABILITY SERVICE TESTS${NC}"
echo "==============================="

# Generate test repository ID
REPO_ID=$(uuidgen 2>/dev/null || echo "550e8400-e29b-41d4-a716-446655440000")

# List vulnerabilities (GET)
test_endpoint "List Vulnerabilities" "GET" "${VULNERABILITIES_URL}?page=1&limit=10" "" "200"

# Get vulnerability by ID (GET) - Using a dummy UUID
test_endpoint "Get Vulnerability by ID" "GET" "${VULNERABILITIES_URL}/550e8400-e29b-41d4-a716-446655440001" "" "404"

# Search vulnerabilities (POST)
test_endpoint "Search Vulnerabilities" "POST" "${VULNERABILITIES_URL}/search" '{
    "query": "sql injection",
    "limit": 5
}' "200"

# Get vulnerability types (GET)
test_endpoint "Get Vulnerability Types" "GET" "${VULNERABILITIES_URL}/types" "" "200"

# Get vulnerabilities by severity (GET)
test_endpoint "Get Vulnerabilities by Severity" "GET" "${VULNERABILITIES_URL}/severity/high?page=1&limit=10" "" "200"

# Get repository vulnerabilities (GET)
test_endpoint "Get Repository Vulnerabilities" "GET" "${VULNERABILITIES_URL}/repository/${REPO_ID}?page=1&limit=10" "" "200"

# Get repository vulnerability summary (GET)
test_endpoint "Get Repository Vulnerability Summary" "GET" "${VULNERABILITIES_URL}/repository/${REPO_ID}/summary" "" "200"

# ===============================
# PATCH SERVICE TESTS
# ===============================
echo -e "${YELLOW}🔧 PATCH SERVICE TESTS${NC}"
echo "======================="

# List patches (GET)
test_endpoint "List Patches" "GET" "${PATCHES_URL}?page=1&limit=10" "" "200"

# Get patch by ID (GET) - Using a dummy UUID
test_endpoint "Get Patch by ID" "GET" "${PATCHES_URL}/550e8400-e29b-41d4-a716-446655440002" "" "404"

# Get patches by vulnerability (GET)
test_endpoint "Get Patches by Vulnerability" "GET" "${PATCHES_URL}/vulnerability/550e8400-e29b-41d4-a716-446655440001" "" "200"

# Get patches by repository (GET)
test_endpoint "Get Patches by Repository" "GET" "${PATCHES_URL}/repository/${REPO_ID}?page=1&limit=10" "" "200"

# Get AI patch suggestions (POST)
test_endpoint "Get AI Patch Suggestions" "POST" "${PATCHES_URL}/ai-suggestions" '{
    "vulnerability_id": "550e8400-e29b-41d4-a716-446655440001",
    "context": {
        "file_path": "src/main.rs",
        "code_snippet": "let query = format!(\"SELECT * FROM users WHERE id = {}\", user_id);",
        "language": "rust"
    }
}' "200"

# ===============================
# DEVELOPER SERVICE TESTS
# ===============================
echo -e "${YELLOW}👨‍💻 DEVELOPER SERVICE TESTS${NC}"
echo "============================"

# List developers (GET)
test_endpoint "List Developers" "GET" "${DEVELOPERS_URL}?page=1&limit=10" "" "200"

# Get developer by ID (GET) - Using a dummy UUID
test_endpoint "Get Developer by ID" "GET" "${DEVELOPERS_URL}/550e8400-e29b-41d4-a716-446655440003" "" "404"

# Search developers (POST)
test_endpoint "Search Developers" "POST" "${DEVELOPERS_URL}/search" '{
    "query": "rust",
    "limit": 5
}' "200"

# Get developer statistics (GET)
test_endpoint "Get Developer Statistics" "GET" "${DEVELOPERS_URL}/550e8400-e29b-41d4-a716-446655440003/statistics" "" "404"

# Get developer's repositories (GET)
test_endpoint "Get Developer Repositories" "GET" "${DEVELOPERS_URL}/550e8400-e29b-41d4-a716-446655440003/repositories?page=1&limit=10" "" "404"

# Get developer's contributions (GET)
test_endpoint "Get Developer Contributions" "GET" "${DEVELOPERS_URL}/550e8400-e29b-41d4-a716-446655440003/contributions?days=30" "" "404"

# Get top developers (GET)
test_endpoint "Get Top Developers" "GET" "${DEVELOPERS_URL}/top?metric=reputation&limit=10" "" "200"

# ===============================
# GITHUB SERVICE TESTS
# ===============================
echo -e "${YELLOW}🐙 GITHUB SERVICE TESTS${NC}"
echo "========================"

# GitHub webhook test (POST)
test_endpoint "GitHub Webhook" "POST" "${GITHUB_URL}/webhook" '{
    "action": "opened",
    "pull_request": {
        "id": 1,
        "number": 123,
        "title": "Test PR"
    }
}' "200"

# Get repository info (GET)
test_endpoint "Get GitHub Repository Info" "GET" "${GITHUB_URL}/repository/owner/repo" "" "200"

# Analyze repository (POST)
test_endpoint "Analyze GitHub Repository" "POST" "${GITHUB_URL}/analyze" '{
    "owner": "test",
    "repo": "test-repo",
    "branch": "main"
}' "200"

# ===============================
# SUI SERVICE TESTS
# ===============================
echo -e "${YELLOW}⚡ SUI SERVICE TESTS${NC}"
echo "===================="

# Test SUI health check (GET)
test_endpoint "SUI Health Check" "GET" "${SUI_URL}/health" "" "200"

# Test SUI connection test (GET)
test_endpoint "SUI Test Connection" "GET" "${SUI_URL}/test-connection" "" "200"

# Get SUI network info (GET)
test_endpoint "Get SUI Network Info" "GET" "${SUI_URL}/network-info" "" "200"

# Sponsor transaction (POST)
test_endpoint "Sponsor Transaction" "POST" "${SUI_URL}/sponsor-transaction" '{
    "transaction_bytes": "mock_transaction_data",
    "gas_budget": 1000000
}' "200"

# ===============================
# ZKPERSONA PROOF GENERATION (PROTECTED)
# ===============================
echo -e "${YELLOW}🔐 ZKPERSONA PROOF GENERATION${NC}"
echo "=============================="

# Test generate proof (requires authentication)
GENERATE_DATA='{
    "behavior_input": {
        "transactions": [
            {"type": "swap", "amount": 1000, "timestamp": "2024-01-01T00:00:00Z"},
            {"type": "stake", "amount": 500, "timestamp": "2024-01-02T00:00:00Z"}
        ],
        "interactions": {
            "dao_votes": 3,
            "nft_trades": 2,
            "defi_protocols": ["uniswap", "aave", "compound"]
        }
    },
    "session_id": "demo-session-123"
}'

if [ -n "$JWT_TOKEN" ]; then
    GENERATE_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${ZKPERSONA_URL}/generate-proof" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -d "$GENERATE_DATA")

    GENERATE_HTTP_CODE=$(echo "$GENERATE_RESPONSE" | tail -1)
    GENERATE_BODY=$(echo "$GENERATE_RESPONSE" | sed '$d')

    echo -e "${BLUE}📝 Testing: Generate Proof (Protected)${NC}"
    echo "   POST ${ZKPERSONA_URL}/generate-proof"
    if [ "$GENERATE_HTTP_CODE" = "200" ]; then
        echo -e "   ${GREEN}✅ PASS${NC} (Status: $GENERATE_HTTP_CODE)"
        ((PASSED++))
        echo "$GENERATE_BODY" | jq '.' 2>/dev/null || echo "$GENERATE_BODY"
        
        # Extract proof data for verification
        PROOF_DATA=$(echo "$GENERATE_BODY" | jq -r '.data.proof_data // empty' 2>/dev/null)
        VERIFICATION_KEY=$(echo "$GENERATE_BODY" | jq -r '.data.verification_key // empty' 2>/dev/null)
        PUBLIC_SIGNALS=$(echo "$GENERATE_BODY" | jq '.data.public_signals // {}' 2>/dev/null)
    else
        echo -e "   ${RED}❌ FAIL${NC} (Expected: 200, Got: $GENERATE_HTTP_CODE)"
        ((FAILED++))
        echo "   Response: $GENERATE_BODY"
    fi
    echo ""
else
    echo -e "${RED}⚠️  Skipping Generate Proof - no JWT token from login${NC}"
    echo ""
fi

# ===============================
# ZKPERSONA PROOF VERIFICATION (PUBLIC)
# ===============================
echo -e "${YELLOW}🔍 ZKPERSONA PROOF VERIFICATION${NC}"
echo "================================"

# Test verify proof (public endpoint)
if [ -n "$PROOF_DATA" ] && [ -n "$VERIFICATION_KEY" ]; then
    VERIFY_DATA=$(cat <<EOF
{
    "proof_data": "$PROOF_DATA",
    "verification_key": "$VERIFICATION_KEY",
    "public_signals": $PUBLIC_SIGNALS
}
EOF
    )
    
    VERIFY_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${ZKPERSONA_URL}/verify" \
        -H "Content-Type: application/json" \
        -d "$VERIFY_DATA")

    VERIFY_HTTP_CODE=$(echo "$VERIFY_RESPONSE" | tail -1)
    VERIFY_BODY=$(echo "$VERIFY_RESPONSE" | sed '$d')

    echo -e "${BLUE}📝 Testing: Verify Proof${NC}"
    echo "   POST ${ZKPERSONA_URL}/verify"
    if [ "$VERIFY_HTTP_CODE" = "200" ]; then
        echo -e "   ${GREEN}✅ PASS${NC} (Status: $VERIFY_HTTP_CODE)"
        ((PASSED++))
        echo "$VERIFY_BODY" | jq '.' 2>/dev/null || echo "$VERIFY_BODY"
    else
        echo -e "   ${RED}❌ FAIL${NC} (Expected: 200, Got: $VERIFY_HTTP_CODE)"
        ((FAILED++))
        echo "   Response: $VERIFY_BODY"
    fi
    echo ""
else
    echo -e "${RED}⚠️  Skipping Verify Proof - no proof data from generate${NC}"
    echo ""
fi

# ===============================
# RPC ENDPOINT TEST
# ===============================
echo -e "${YELLOW}🔧 RPC ENDPOINT TEST${NC}"
echo "===================="

# Test RPC endpoint
test_endpoint "RPC Endpoint" "POST" "${RPC_URL}/rpc" '{
    "jsonrpc": "2.0",
    "method": "test_method",
    "params": {},
    "id": 1
}' "200"

# ===============================
# SUMMARY
# ===============================
echo -e "${YELLOW}📊 TEST SUMMARY${NC}"
echo "==============="
echo -e "✅ Passed: ${GREEN}$PASSED${NC}"
echo -e "❌ Failed: ${RED}$FAILED${NC}"
echo -e "📊 Total:  $(($PASSED + $FAILED))"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed. Check the server logs and endpoints.${NC}"
    exit 1
fi

echo ""
echo "📋 Test Coverage:"
echo "- ZK-Persona Auth: Nonce generation, wallet login"
echo "- Analytics: Metrics, top repositories, trends"
echo "- Vulnerabilities: CRUD operations, search, filtering"
echo "- Patches: CRUD operations, AI suggestions"
echo "- Developers: CRUD operations, statistics, rankings"
echo "- GitHub: Webhook handling, repository analysis"
echo "- SUI: Health checks, transaction sponsorship"
echo "- ZK-Persona Proof: Generation and verification"
echo "- RPC: JSON-RPC interface"
echo ""
echo "🔧 Notes:"
echo "- Make sure the server is running on localhost:8080"
echo "- Install 'jq' for better JSON formatting"
echo "- Some endpoints return mock data or placeholders"
echo "- Authentication is required for protected endpoints"