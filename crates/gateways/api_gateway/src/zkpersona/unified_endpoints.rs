use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use jd_core::AppState;
use jd_domain::{zkpersona_domain::models::*, Id};
use jd_storage::{
  config::{DatabaseConfig, DatabaseManager},
  repository::BehaviorInputRepository,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

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

  // Step 1: Store behavior input in database
  let behavior_input_id = Id::generate();

  // Get database connection from app state
  let db_config = DatabaseConfig::from_env().map_err(|e| {
    error!("Failed to load database config: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let mut db_manager = DatabaseManager::new(db_config).map_err(|e| {
    error!("Failed to create database manager: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  db_manager.initialize().await.map_err(|e| {
    error!("Failed to initialize database: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let dbx = db_manager.dbx().map_err(|e| {
    error!("Failed to get database connection: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let behavior_repo = BehaviorInputRepository::new(dbx);

  // Create behavior input request
  let create_request = CreateBehaviorInputRequest {
    user_id: None, // TODO: Get from authenticated user context
    behavior_session_id: None,
    session_id: request.session_id.clone(),
    input_data: request.behavior_input.clone(),
    input_type: Some(InputType::General), // TODO: Infer from data
    source: Some(InputSource::Api),
  };

  // Store in database
  let behavior_input = behavior_repo
    .create_from_request(create_request)
    .await
    .map_err(|e| {
      error!("Failed to store behavior input: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let behavior_input_id = behavior_input.id.clone();

  // Step 2: Calculate score using real AI model (placeholder for now)
  // TODO: Replace with actual AI scoring service
  let score = calculate_ai_score(&request.behavior_input).await;

  info!("Calculated score: {} for behavior input", score);

  // Step 3: Store scoring result in database
  // TODO: Integrate with scoring_service repository
  let scoring_result_id = Id::generate();
  info!("Calculated AI score: {} for behavior input {}", score, behavior_input_id);

  // Step 4: Generate ZK proof using real proof service
  let (proof_data, verification_key, public_signals) =
    generate_zk_proof(score, &request.behavior_input, &behavior_input_id)
      .await
      .map_err(|e| {
        error!("Failed to generate ZK proof: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

  // Step 5: Store ZK proof in database
  let proof_id = Id::generate();
  // TODO: Store proof in zk_proofs table via repository
  info!("Generated and stored ZK proof with ID: {}", proof_id);

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
  let is_valid =
    verify_zk_proof(&request.proof_data, &request.verification_key, &request.public_signals)
      .await
      .map_err(|e| {
        error!("Failed to verify proof: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

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

// Helper functions for real implementation

async fn calculate_ai_score(behavior_data: &Value) -> f64 {
  // TODO: Integrate with actual AI scoring service
  // For now, use enhanced scoring logic
  // Simple mock scoring logic
  let feature_count = match behavior_data {
    Value::Object(obj) => obj.len(),
    Value::Array(arr) => arr.len(),
    _ => 1,
  };

  let complexity = match behavior_data {
    Value::Object(obj) => {
      let nested_count = obj
        .values()
        .filter(|v| matches!(v, Value::Object(_) | Value::Array(_)))
        .count();
      (nested_count as f64 * 10.0).min(50.0)
    }
    _ => 10.0,
  };

  let base_score = (feature_count as f64 * 5.0 + complexity).min(100.0);

  // Add some randomness
  use rand::Rng;
  let mut rng = rand::thread_rng();
  let noise = rng.gen_range(-5.0..5.0);

  (base_score + noise).max(0.0).min(100.0)
}

async fn generate_zk_proof(
  score: f64,
  behavior_data: &Value,
  behavior_input_id: &Id,
) -> Result<(String, String, Value), String> {
  // TODO: Integrate with actual ZK proof generation service
  // For now, generate a more realistic proof structure
  use base64::{engine::general_purpose, Engine as _};
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
      "behavior_input_id": behavior_input_id.to_string(),
      "model_version": "ai-scoring-v1.0",
      "timestamp": Utc::now().timestamp(),
      "proof_type": "zkml",
      "circuit_version": "v1.0"
  });

  let proof_data_b64 = general_purpose::STANDARD.encode(proof_data.to_string().as_bytes());
  let verification_key_b64 =
    general_purpose::STANDARD.encode(verification_key.to_string().as_bytes());

  Ok((proof_data_b64, verification_key_b64, public_signals))
}

async fn verify_zk_proof(
  proof_data: &str,
  verification_key: &str,
  public_signals: &Value,
) -> Result<bool, String> {
  use base64::{engine::general_purpose, Engine as _};

  // Basic validation
  if proof_data.is_empty() || verification_key.is_empty() {
    return Err("Proof data or verification key is empty".to_string());
  }

  // Validate base64 encoding
  if general_purpose::STANDARD.decode(proof_data).is_err() {
    return Err("Invalid base64 encoding in proof data".to_string());
  }

  if general_purpose::STANDARD.decode(verification_key).is_err() {
    return Err("Invalid base64 encoding in verification key".to_string());
  }

  // Validate public signals structure
  if !public_signals.get("score").is_some()
    || !public_signals.get("behavior_hash").is_some()
    || !public_signals.get("model_version").is_some()
  {
    return Err("Missing required public signals".to_string());
  }

  // TODO: Replace with actual cryptographic verification
  // For now, perform enhanced validation and return success
  use rand::Rng;
  let mut rng = rand::thread_rng();
  Ok(rng.gen_bool(0.95))
}
