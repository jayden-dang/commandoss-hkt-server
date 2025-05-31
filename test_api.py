#!/usr/bin/env python3
"""
Simple API Test Script for ZK-Persona Endpoints
Usage: python3 test_api.py
"""

import requests
import json
import sys
from typing import Dict, Any

BASE_URL = "http://localhost:8080"

def test_endpoint(method: str, endpoint: str, data: Dict[Any, Any] = None, description: str = ""):
    """Test a single API endpoint"""
    print(f"\n🧪 Testing: {description}")
    print(f"   {method} {endpoint}")
    
    url = f"{BASE_URL}{endpoint}"
    
    try:
        if method == "GET":
            response = requests.get(url, timeout=10)
        elif method == "POST":
            response = requests.post(url, json=data, timeout=10)
        else:
            print(f"   ❌ Unsupported method: {method}")
            return None
        
        # Print result
        if response.status_code in [200, 201]:
            print(f"   ✅ Success ({response.status_code})")
            try:
                json_response = response.json()
                print(f"   📄 Response: {json.dumps(json_response, indent=2)[:200]}...")
                return json_response
            except:
                print(f"   📄 Response: {response.text[:200]}...")
                return response.text
        else:
            print(f"   ❌ Failed ({response.status_code})")
            print(f"   📄 Error: {response.text[:200]}...")
            return None
            
    except requests.exceptions.ConnectionError:
        print(f"   🔌 Connection Error: Is the server running at {BASE_URL}?")
        return None
    except requests.exceptions.Timeout:
        print(f"   ⏰ Timeout: Server took too long to respond")
        return None
    except Exception as e:
        print(f"   💥 Error: {str(e)}")
        return None

def main():
    print("🚀 ZK-Persona API Test Suite")
    print(f"🎯 Testing against: {BASE_URL}")
    print("=" * 50)
    
    # Health checks
    test_endpoint("GET", "/api/v1/sui/health", None, "Health Check")
    test_endpoint("GET", "/api/v1/sui/test-connection", None, "Test Connection")
    
    # ZK-Persona proof generation and verification flow
    generate_response = test_endpoint("POST", "/api/v1/zkpersona/generate-proof", {
        "session_id": "test-session-123",
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
        }
    }, "Generate ZK Proof (Full Pipeline)")
    
    # If proof generation succeeded, test verification
    if generate_response and isinstance(generate_response, dict) and generate_response.get("data"):
        data = generate_response["data"]
        verify_data = {
            "proof_data": data.get("proof_data"),
            "verification_key": data.get("verification_key"),
            "public_signals": data.get("public_signals")
        }
        test_endpoint("POST", "/api/v1/zkpersona/verify", verify_data, "Verify ZK Proof")
    else:
        print("\n🧪 Testing: Verify ZK Proof")
        print("   ❌ Skipped - proof generation failed")
    
    # RPC endpoint (currently returns "Unknown method" for all methods)
    test_endpoint("POST", "/api/rpc", {
        "id": 1,
        "method": "test_method",
        "params": {}
    }, "RPC: Test JSON-RPC Endpoint")
    
    print("\n" + "=" * 50)
    print("🏁 Test Suite Complete!")
    print(f"\n💡 To start the server, run: cargo run")
    print(f"🌐 Server should be accessible at {BASE_URL}")

if __name__ == "__main__":
    main()