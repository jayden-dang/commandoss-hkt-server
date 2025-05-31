#!/usr/bin/env python3
"""
Test script for ZKPersona Authentication API
Tests the complete authentication flow including:
- Nonce generation
- Wallet login with signature
- Protected endpoint access
"""

import requests
import json
import sys
from datetime import datetime
import base64
import secrets

# API Configuration
BASE_URL = "http://localhost:8080/api/v1/zkpersona"

# Test wallet credentials (mock data for testing)
TEST_WALLET = {
    "address": "0x1234567890abcdef1234567890abcdef12345678",
    "public_key": "dGVzdF9wdWJsaWNfa2V5XzEyMzQ1Njc4OTBhYmNkZWY=",  # base64 encoded test key
    "signature": "AH4gbmV3IGNvbGxlY3Rpb24gd2l0aCByZWR1Y2VkIGdhcyBmZWVzLiBUaGUgY29sbGVjdGlvbiB3aWxsIGNvbnNpc3Qgb2YgMTAwMCB1bmlxdWUgTkZUcyB3aXRoIGRpZmZlcmVudCByYXJpdHkgbGV2ZWxzLgAAAAAAAAAAAAAAAAA="  # base64 encoded test signature
}

# Colors for terminal output
GREEN = '\033[92m'
RED = '\033[91m'
BLUE = '\033[94m'
YELLOW = '\033[93m'
RESET = '\033[0m'

def print_test_header(test_name):
    """Print a formatted test header"""
    print(f"\n{BLUE}{'='*60}{RESET}")
    print(f"{BLUE}Testing: {test_name}{RESET}")
    print(f"{BLUE}{'='*60}{RESET}")

def print_success(message):
    """Print success message"""
    print(f"{GREEN}✓ {message}{RESET}")

def print_error(message):
    """Print error message"""
    print(f"{RED}✗ {message}{RESET}")

def print_info(message):
    """Print info message"""
    print(f"{YELLOW}ℹ {message}{RESET}")

def test_nonce_generation():
    """Test 1: Generate nonce for wallet authentication"""
    print_test_header("Nonce Generation")
    
    endpoint = f"{BASE_URL}/auth/nonce"
    payload = {
        "wallet_address": TEST_WALLET["address"]
    }
    
    print(f"Endpoint: POST {endpoint}")
    print(f"Payload: {json.dumps(payload, indent=2)}")
    
    try:
        response = requests.post(endpoint, json=payload)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"Response: {json.dumps(data, indent=2)}")
            print_success("Nonce generated successfully")
            # Extract message from nested data structure
            if "data" in data and "message" in data["data"]:
                return data["data"]["message"]
            elif "message" in data:
                return data["message"]
            else:
                print_error("Could not find message in response")
                return None
        else:
            print_error(f"Failed to generate nonce: {response.text}")
            return None
    except Exception as e:
        print_error(f"Error: {str(e)}")
        return None

def test_wallet_login(nonce_message=None):
    """Test 2: Login with wallet signature"""
    print_test_header("Wallet Login")
    
    endpoint = f"{BASE_URL}/auth/login"
    
    # If no nonce provided, use a default message
    if not nonce_message:
        nonce_message = "Sign this message to authenticate: 12345678"
    
    payload = {
        "wallet_address": TEST_WALLET["address"],
        "signature": TEST_WALLET["signature"],
        "public_key": TEST_WALLET["public_key"]
    }
    
    print(f"Endpoint: POST {endpoint}")
    print(f"Payload: {json.dumps(payload, indent=2)}")
    
    try:
        response = requests.post(endpoint, json=payload)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"Response: {json.dumps(data, indent=2)}")
            print_success("Login successful")
            
            # Extract auth cookie
            auth_cookie = response.cookies.get('auth-token')
            if auth_cookie:
                print_success(f"Auth cookie received: {auth_cookie[:20]}...")
                return auth_cookie
            else:
                print_info("No auth cookie in response")
                return None
        else:
            print_error(f"Failed to login: {response.text}")
            return None
    except Exception as e:
        print_error(f"Error: {str(e)}")
        return None

def test_protected_endpoint(auth_token=None):
    """Test 3: Access protected generate-proof endpoint"""
    print_test_header("Protected Endpoint Access")
    
    endpoint = f"{BASE_URL}/generate-proof"
    
    # Test behavior data
    behavior_data = {
        "user_actions": [
            {"action": "page_view", "page": "home", "duration": 5.2},
            {"action": "click", "element": "signup_button", "timestamp": datetime.now().isoformat()},
            {"action": "form_submit", "form": "profile", "success": True}
        ],
        "session_info": {
            "browser": "Chrome",
            "device": "Desktop",
            "ip_country": "US"
        }
    }
    
    payload = {
        "behavior_input": behavior_data,
        "session_id": f"test_session_{secrets.token_hex(8)}"
    }
    
    print(f"Endpoint: POST {endpoint}")
    print(f"Payload: {json.dumps(payload, indent=2)}")
    
    # Prepare headers with auth cookie if available
    cookies = {}
    if auth_token:
        cookies['auth-token'] = auth_token
        print_info(f"Using auth token: {auth_token[:20]}...")
    else:
        print_info("No auth token provided - expecting 401")
    
    try:
        response = requests.post(endpoint, json=payload, cookies=cookies)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"Response: {json.dumps(data, indent=2)}")
            print_success("Successfully generated proof")
            return data
        elif response.status_code == 401:
            print_error("Unauthorized - Authentication required")
            return None
        else:
            print_error(f"Failed to generate proof: {response.text}")
            return None
    except Exception as e:
        print_error(f"Error: {str(e)}")
        return None

def test_verify_proof(proof_data=None):
    """Test 4: Verify a ZK proof"""
    print_test_header("Proof Verification")
    
    endpoint = f"{BASE_URL}/verify"
    
    # Use provided proof or create mock data
    if proof_data:
        payload = {
            "proof_data": proof_data.get("proof_data", ""),
            "verification_key": proof_data.get("verification_key", ""),
            "public_signals": proof_data.get("public_signals", {})
        }
    else:
        # Mock proof data for testing
        payload = {
            "proof_data": base64.b64encode(b"mock_proof_data").decode(),
            "verification_key": base64.b64encode(b"mock_verification_key").decode(),
            "public_signals": {
                "score": 85.5,
                "score_range": [0, 100],
                "behavior_hash": "abc123def456",
                "model_version": "ai-scoring-v1.0",
                "timestamp": datetime.now().timestamp()
            }
        }
    
    print(f"Endpoint: POST {endpoint}")
    print(f"Payload: {json.dumps(payload, indent=2)}")
    
    try:
        response = requests.post(endpoint, json=payload)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"Response: {json.dumps(data, indent=2)}")
            if data.get("valid"):
                print_success("Proof verified successfully")
            else:
                print_error("Proof verification failed")
            return data
        else:
            print_error(f"Failed to verify proof: {response.text}")
            return None
    except Exception as e:
        print_error(f"Error: {str(e)}")
        return None

def test_invalid_wallet():
    """Test 5: Test with invalid wallet address"""
    print_test_header("Invalid Wallet Address")
    
    endpoint = f"{BASE_URL}/auth/nonce"
    payload = {
        "wallet_address": "invalid_address"
    }
    
    print(f"Endpoint: POST {endpoint}")
    print(f"Payload: {json.dumps(payload, indent=2)}")
    
    try:
        response = requests.post(endpoint, json=payload)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code == 400:
            print_success("Correctly rejected invalid wallet address")
        else:
            print_error(f"Expected 400, got {response.status_code}: {response.text}")
    except Exception as e:
        print_error(f"Error: {str(e)}")

def test_expired_nonce():
    """Test 6: Test with expired nonce (simulation)"""
    print_test_header("Expired Nonce")
    
    print_info("This test simulates an expired nonce scenario")
    print_info("In real scenario, you would wait 5+ minutes after generating nonce")
    
    # For testing, we'll just use an invalid signature
    endpoint = f"{BASE_URL}/auth/login"
    payload = {
        "wallet_address": TEST_WALLET["address"],
        "signature": "invalid_signature",
        "public_key": TEST_WALLET["public_key"]
    }
    
    print(f"Endpoint: POST {endpoint}")
    
    try:
        response = requests.post(endpoint, json=payload)
        print(f"\nStatus Code: {response.status_code}")
        
        if response.status_code in [401, 404, 410]:  # 410 Gone for expired
            print_success("Correctly rejected invalid/expired nonce")
        else:
            print_error(f"Expected error status, got {response.status_code}")
    except Exception as e:
        print_error(f"Error: {str(e)}")

def run_all_tests():
    """Run all authentication tests"""
    print(f"\n{BLUE}{'='*60}{RESET}")
    print(f"{BLUE}ZKPersona Authentication API Test Suite{RESET}")
    print(f"{BLUE}{'='*60}{RESET}")
    print(f"Target: {BASE_URL}")
    print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    
    # Test 1: Generate nonce
    nonce_message = test_nonce_generation()
    
    # Test 2: Login with wallet
    auth_token = None
    if nonce_message:
        print_info("Using generated nonce for login")
        auth_token = test_wallet_login(nonce_message)
    else:
        print_info("Testing login with default message")
        auth_token = test_wallet_login()
    
    # Test 3: Access protected endpoint without auth
    print_info("\nTesting protected endpoint WITHOUT authentication")
    test_protected_endpoint(None)
    
    # Test 4: Access protected endpoint with auth
    if auth_token:
        print_info("\nTesting protected endpoint WITH authentication")
        proof_data = test_protected_endpoint(auth_token)
        
        # Test 5: Verify the generated proof
        if proof_data:
            test_verify_proof(proof_data)
    
    # Test 6: Verify with mock proof
    print_info("\nTesting verification with mock proof data")
    test_verify_proof(None)
    
    # Test 7: Invalid wallet address
    test_invalid_wallet()
    
    # Test 8: Expired nonce
    test_expired_nonce()
    
    print(f"\n{BLUE}{'='*60}{RESET}")
    print(f"{BLUE}Test Suite Completed{RESET}")
    print(f"{BLUE}{'='*60}{RESET}")

if __name__ == "__main__":
    try:
        # Check if server is running
        try:
            response = requests.get(f"http://localhost:8080/health", timeout=2)
        except:
            print_error("Server is not running on localhost:8080")
            print_info("Please start the server with: cargo run --bin web_server")
            sys.exit(1)
        
        run_all_tests()
    except KeyboardInterrupt:
        print_error("\nTest interrupted by user")
        sys.exit(1)