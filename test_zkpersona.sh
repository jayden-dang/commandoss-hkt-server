#!/bin/bash

# ZK-Persona API Test Script
# This script demonstrates the new /generate-proof and /verify endpoints

BASE_URL="http://localhost:8080/api/v1/zkpersona"

echo "🧪 Testing ZK-Persona API Endpoints"
echo "=================================="

# Test 1: Generate Proof
echo ""
echo "📝 Test 1: Generate Proof"
echo "POST ${BASE_URL}/generate-proof"

GENERATE_RESPONSE=$(curl -s -X POST "${BASE_URL}/generate-proof" \
  -H "Content-Type: application/json" \
  -d '{
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
  }')

echo "Response:"
echo "$GENERATE_RESPONSE" | jq '.'

# Extract proof data for verification test
PROOF_DATA=$(echo "$GENERATE_RESPONSE" | jq -r '.data.proof_data // empty')
VERIFICATION_KEY=$(echo "$GENERATE_RESPONSE" | jq -r '.data.verification_key // empty')
PUBLIC_SIGNALS=$(echo "$GENERATE_RESPONSE" | jq '.data.public_signals // {}')

# Test 2: Verify Proof
echo ""
echo "🔍 Test 2: Verify Proof"
echo "POST ${BASE_URL}/verify"

if [ -n "$PROOF_DATA" ] && [ -n "$VERIFICATION_KEY" ]; then
  VERIFY_RESPONSE=$(curl -s -X POST "${BASE_URL}/verify" \
    -H "Content-Type: application/json" \
    -d "{
      \"proof_data\": \"$PROOF_DATA\",
      \"verification_key\": \"$VERIFICATION_KEY\",
      \"public_signals\": $PUBLIC_SIGNALS
    }")

  echo "Response:"
  echo "$VERIFY_RESPONSE" | jq '.'
else
  echo "❌ Cannot verify proof - proof generation failed"
fi

echo ""
echo "✅ ZK-Persona API test complete!"
echo ""
echo "📋 Summary:"
echo "- Behavior input → AI scoring → ZK proof generation"
echo "- Proof verification with public signals"
echo "- Ready for EZKL integration!"