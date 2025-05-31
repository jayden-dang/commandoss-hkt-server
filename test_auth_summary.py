#!/usr/bin/env python3
"""
Summary test script for ZKPersona Authentication API
Shows clear pass/fail status for each test
"""

import requests
import json
import base64
from datetime import datetime

# API Configuration
BASE_URL = "http://localhost:8080/api/v1/zkpersona"

# Test results
test_results = []

def add_result(test_name, passed, details=""):
    """Add a test result"""
    test_results.append({
        "test": test_name,
        "passed": passed,
        "details": details
    })

def run_test(test_name, func):
    """Run a test and catch exceptions"""
    try:
        result = func()
        return result
    except Exception as e:
        add_result(test_name, False, f"Exception: {str(e)}")
        return False

def test_1_nonce_generation():
    """Test nonce generation"""
    test_name = "Nonce Generation"
    
    try:
        response = requests.post(
            f"{BASE_URL}/auth/nonce",
            json={"wallet_address": "0x1234567890abcdef1234567890abcdef12345678"}
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get("status") == 0 and "data" in data and "message" in data["data"]:
                add_result(test_name, True, "Successfully generated nonce")
                return True
            else:
                add_result(test_name, False, "Invalid response format")
        else:
            add_result(test_name, False, f"Status code: {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def test_2_invalid_wallet():
    """Test invalid wallet validation"""
    test_name = "Invalid Wallet Validation"
    
    try:
        response = requests.post(
            f"{BASE_URL}/auth/nonce",
            json={"wallet_address": "invalid_address"}
        )
        
        if response.status_code == 400:
            add_result(test_name, True, "Correctly rejected invalid wallet")
            return True
        else:
            add_result(test_name, False, f"Expected 400, got {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def test_3_login_invalid_signature():
    """Test login with invalid signature"""
    test_name = "Login with Invalid Signature"
    
    try:
        response = requests.post(
            f"{BASE_URL}/auth/login",
            json={
                "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
                "signature": "invalid_signature",
                "public_key": "invalid_key"
            }
        )
        
        if response.status_code == 401:
            add_result(test_name, True, "Correctly rejected invalid signature")
            return True
        else:
            add_result(test_name, False, f"Expected 401, got {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def test_4_protected_endpoint_no_auth():
    """Test protected endpoint without auth"""
    test_name = "Protected Endpoint (No Auth)"
    
    try:
        response = requests.post(
            f"{BASE_URL}/generate-proof",
            json={
                "behavior_input": {"test": "data"},
                "session_id": "test_123"
            }
        )
        
        if response.status_code == 401:
            add_result(test_name, True, "Correctly required authentication")
            return True
        else:
            add_result(test_name, False, f"Expected 401, got {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def test_5_verify_proof():
    """Test proof verification"""
    test_name = "Proof Verification"
    
    try:
        response = requests.post(
            f"{BASE_URL}/verify",
            json={
                "proof_data": base64.b64encode(b"mock_proof").decode(),
                "verification_key": base64.b64encode(b"mock_key").decode(),
                "public_signals": {
                    "score": 85.5,
                    "score_range": [0, 100],
                    "behavior_hash": "abc123",
                    "model_version": "ai-scoring-v1.0",
                    "timestamp": datetime.now().timestamp()
                }
            }
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get("status") == 0:
                add_result(test_name, True, "Successfully verified proof")
                return True
            else:
                add_result(test_name, False, "Verification failed")
        else:
            add_result(test_name, False, f"Status code: {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def test_6_nonce_expiry():
    """Test nonce expiry (simulated)"""
    test_name = "Nonce Expiry Check"
    
    try:
        # Try to login without generating a nonce first
        response = requests.post(
            f"{BASE_URL}/auth/login",
            json={
                "wallet_address": "0x9999999999999999999999999999999999999999",
                "signature": "test_signature",
                "public_key": "test_key"
            }
        )
        
        if response.status_code in [401, 404]:
            add_result(test_name, True, "Correctly handled missing/expired nonce")
            return True
        else:
            add_result(test_name, False, f"Expected 401/404, got {response.status_code}")
    except Exception as e:
        add_result(test_name, False, str(e))
    return False

def print_summary():
    """Print test summary"""
    print("\n" + "="*60)
    print("ZKPersona Authentication API - Test Summary")
    print("="*60)
    print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"API Base URL: {BASE_URL}")
    print("="*60)
    
    passed = 0
    failed = 0
    
    for result in test_results:
        status = "✅ PASS" if result["passed"] else "❌ FAIL"
        print(f"{status} - {result['test']}")
        if result["details"]:
            print(f"         {result['details']}")
        
        if result["passed"]:
            passed += 1
        else:
            failed += 1
    
    print("="*60)
    print(f"Total Tests: {len(test_results)}")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")
    print(f"Success Rate: {(passed/len(test_results)*100):.1f}%")
    print("="*60)
    
    if failed > 0:
        print("\n⚠️  Some tests failed. This is expected for:")
        print("   - Login tests (mock signatures won't validate)")
        print("   - Any test requiring real wallet signatures")
    else:
        print("\n✅ All tests passed!")

def main():
    """Run all tests"""
    # Check if server is running
    try:
        requests.get(f"http://localhost:8080/health", timeout=1)
    except:
        print("❌ Server is not running on localhost:8080")
        print("   Please start the server with: cargo run --bin web_server")
        return
    
    print("Running ZKPersona Authentication API Tests...")
    
    # Run tests
    run_test("Nonce Generation", test_1_nonce_generation)
    run_test("Invalid Wallet Validation", test_2_invalid_wallet)
    run_test("Login with Invalid Signature", test_3_login_invalid_signature)
    run_test("Protected Endpoint (No Auth)", test_4_protected_endpoint_no_auth)
    run_test("Proof Verification", test_5_verify_proof)
    run_test("Nonce Expiry Check", test_6_nonce_expiry)
    
    # Print summary
    print_summary()

if __name__ == "__main__":
    main()