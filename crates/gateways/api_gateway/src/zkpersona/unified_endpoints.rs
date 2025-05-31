use axum::{extract::State, response::Json, http::StatusCode};
use jd_core::AppState;
use jd_domain::{Id, zkpersona_domain::profile::{BehaviorInput, ScoringResult, ZkProof}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, error};

// Unified request/response types for the endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofRequest {
    pub behavior_input: Value,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofResponse {
    pub proof_id: Id,
    pub behavior_input_id: Id,
    pub scoring_result_id: Id,
    pub score: f64,
    pub proof_data: String,
    pub verification_key: String,
    pub public_signals: Value,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    pub proof_data: String,
    pub verification_key: String,
    pub public_signals: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofResponse {
    pub valid: bool,
    pub proof_id: Option<Id>,
    pub success: bool,
    pub message: String,
}

/// POST /generate-proof
/// Main endpoint that:
/// 1. Accepts behavior input
/// 2. Calculates AI score  
/// 3. Generates ZK proof
/// 4. Returns proof + public signals
pub async fn generate_proof(
    State(app_state): State<AppState>,
    Json(request): Json<GenerateProofRequest>,
) -> Result<Json<GenerateProofResponse>, StatusCode> {
    info!("Generate proof request received for session: {:?}", request.session_id);
    
    // Step 1: Store behavior input (mock implementation for now)
    let behavior_input_id = Id::generate();
    let behavior_input = BehaviorInput {
        session_id: request.session_id.clone(),
        input_data: request.behavior_input.clone(),
    };
    
    // For this demo, we'll simulate the full pipeline
    // In production, you'd call the actual services
    
    // Step 2: Calculate score using hardcoded model
    let score = calculate_mock_score(&request.behavior_input);
    
    info!("Calculated score: {} for behavior input", score);
    
    // Step 3: Create scoring result
    let scoring_result_id = Id::generate();
    let scoring_result = ScoringResult {
        behavior_input_id: behavior_input_id.clone(),
        score,
        model_version: "hardcoded-v1.0".to_string(),
    };
    
    // Step 4: Generate ZK proof (mock implementation)
    let (proof_data, verification_key, public_signals) = generate_mock_proof(score, &request.behavior_input).await;
    
    let proof_id = Id::generate();
    let zk_proof = ZkProof {
        scoring_result_id: scoring_result_id.clone(),
        proof_data: proof_data.as_bytes().to_vec(),
        verification_key: verification_key.as_bytes().to_vec(),
        verified: false,
        blockchain_tx_hash: None,
    };
    
    info!("Generated ZK proof with ID: {}", proof_id);
    
    let response = GenerateProofResponse {
        proof_id,
        behavior_input_id,
        scoring_result_id,
        score,
        proof_data,
        verification_key,
        public_signals,
        success: true,
        message: "Proof generated successfully".to_string(),
    };
    
    Ok(Json(response))
}

/// POST /verify
/// Verifies a ZK proof
pub async fn verify_proof(
    State(app_state): State<AppState>,
    Json(request): Json<VerifyProofRequest>,
) -> Result<Json<VerifyProofResponse>, StatusCode> {
    info!("Verify proof request received");
    
    // Mock verification logic
    let is_valid = verify_mock_proof(&request.proof_data, &request.verification_key, &request.public_signals).await;
    
    let response = VerifyProofResponse {
        valid: is_valid,
        proof_id: if is_valid { Some(Id::generate()) } else { None },
        success: true,
        message: if is_valid { 
            "Proof verified successfully".to_string() 
        } else { 
            "Proof verification failed".to_string() 
        },
    };
    
    info!("Proof verification result: {}", is_valid);
    
    Ok(Json(response))
}

// Helper functions for mock implementation

fn calculate_mock_score(behavior_data: &Value) -> f64 {
    // Simple mock scoring logic
    let feature_count = match behavior_data {
        Value::Object(obj) => obj.len(),
        Value::Array(arr) => arr.len(),
        _ => 1,
    };
    
    let complexity = match behavior_data {
        Value::Object(obj) => {
            let nested_count = obj.values()
                .filter(|v| matches!(v, Value::Object(_) | Value::Array(_)))
                .count();
            (nested_count as f64 * 10.0).min(50.0)
        },
        _ => 10.0,
    };
    
    let base_score = (feature_count as f64 * 5.0 + complexity).min(100.0);
    
    // Add some randomness
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let noise = rng.gen_range(-5.0..5.0);
    
    (base_score + noise).max(0.0).min(100.0)
}

async fn generate_mock_proof(score: f64, behavior_data: &Value) -> (String, String, Value) {
    use base64::{Engine as _, engine::general_purpose};
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    
    // Generate mock proof data
    let proof_data = json!({
        "pi_a": [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>()), "1"],
        "pi_b": [[format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                 [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                 ["1", "0"]],
        "pi_c": [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>()), "1"],
        "protocol": "groth16"
    });
    
    let verification_key = json!({
        "alpha": [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>()), "1"],
        "beta": [[format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                 [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                 ["1", "0"]],
        "gamma": [[format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                  [format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>())], 
                  ["1", "0"]],
        "ic": [[format!("{:016x}", rng.r#gen::<u64>()), format!("{:016x}", rng.r#gen::<u64>()), "1"]]
    });
    
    let public_signals = json!({
        "score": score,
        "score_range": [0, 100],
        "behavior_hash": format!("{:016x}", behavior_data.to_string().chars().map(|c| c as u64).sum::<u64>()),
        "model_version": "hardcoded-v1.0",
        "timestamp": jd_utils::time::now_utc().unix_timestamp()
    });
    
    let proof_data_b64 = general_purpose::STANDARD.encode(proof_data.to_string().as_bytes());
    let verification_key_b64 = general_purpose::STANDARD.encode(verification_key.to_string().as_bytes());
    
    (proof_data_b64, verification_key_b64, public_signals)
}

async fn verify_mock_proof(proof_data: &str, verification_key: &str, public_signals: &Value) -> bool {
    use base64::{Engine as _, engine::general_purpose};
    
    // Basic validation
    if proof_data.is_empty() || verification_key.is_empty() {
        return false;
    }
    
    // Validate base64 encoding
    if general_purpose::STANDARD.decode(proof_data).is_err() {
        return false;
    }
    
    if general_purpose::STANDARD.decode(verification_key).is_err() {
        return false;
    }
    
    // Validate public signals structure
    if !public_signals.get("score").is_some() ||
       !public_signals.get("behavior_hash").is_some() ||
       !public_signals.get("model_version").is_some() {
        return false;
    }
    
    // Mock verification passes with 95% probability
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.95)
}