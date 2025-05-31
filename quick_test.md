# Quick API Test Guide

## Start the Server
```bash
cargo run
# Server will start on http://localhost:8080
```

## Test Commands (copy and paste)

### 1. Health Check
```bash
curl http://localhost:8080/api/v1/sui/health
```

### 2. Generate ZK Proof (End-to-End Pipeline)
```bash
curl -X POST http://localhost:8080/api/v1/zkpersona/generate-proof \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "test-session-123",
    "behavior_input": {
      "clicks": 42,
      "scrolls": 15,
      "time_spent": 300,
      "pages_visited": 5
    }
  }'
```

### 3. Verify ZK Proof
```bash
curl -X POST http://localhost:8080/api/v1/zkpersona/verify-proof \
  -H "Content-Type: application/json" \
  -d '{
    "proof_data": "mock-proof-data",
    "verification_key": "mock-verification-key",
    "public_signals": {
      "score": 85.5,
      "model_version": "v1.0"
    }
  }'
```

### 4. RPC - Create Behavior Input
```bash
curl -X POST http://localhost:8080/api \
  -H "Content-Type: application/json" \
  -d '{
    "id": 1,
    "method": "behavior_input_create",
    "params": {
      "session_id": "session-456",
      "input_data": {
        "mouse_movements": 250,
        "keystrokes": 180,
        "session_duration": 450
      }
    }
  }'
```

### 5. RPC - List Behavior Inputs
```bash
curl -X POST http://localhost:8080/api \
  -H "Content-Type: application/json" \
  -d '{
    "id": 2,
    "method": "behavior_input_list",
    "params": {
      "limit": 10,
      "offset": 0
    }
  }'
```

## Available Scripts

### Bash Script
```bash
./test_api.sh
```

### Python Script
```bash
python3 test_api.py
```

## Expected Responses

The API endpoints return JSON responses. Since this is a mock implementation, you'll get placeholder data showing the structure of the ZK-Persona system:

- **Behavior Input**: Captures user interaction data
- **Scoring**: Calculates a score from behavior data
- **ZK Proof**: Generates cryptographic proof of the score
- **Verification**: Validates the proof

The system demonstrates the complete pipeline from behavior capture to zero-knowledge proof generation.
