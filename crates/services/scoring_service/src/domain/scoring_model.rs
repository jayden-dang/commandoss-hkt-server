use rand::Rng;
use serde_json::Value;
use crate::{Error, Result};

pub struct HardcodedScoringModel {
    version: String,
}

impl HardcodedScoringModel {
    pub fn new() -> Self {
        Self {
            version: "hardcoded-v1.0".to_string(),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub async fn calculate_score(&self, behavior_data: &Value) -> Result<f64> {
        // Hardcoded scoring logic - replace with actual AI model later
        
        // Extract some mock features from behavior data
        let feature_count = self.extract_feature_count(behavior_data)?;
        let complexity_score = self.calculate_complexity(behavior_data)?;
        
        // Simple scoring formula - normalize to 0-100 range
        let base_score = (feature_count as f64 * 10.0 + complexity_score * 50.0).min(100.0);
        
        // Add some randomness to simulate model variance
        let mut rng = rand::thread_rng();
        let noise = rng.gen_range(-5.0..5.0);
        
        let final_score = (base_score + noise).max(0.0).min(100.0);
        
        Ok(final_score)
    }

    fn extract_feature_count(&self, data: &Value) -> Result<usize> {
        match data {
            Value::Object(obj) => Ok(obj.len()),
            Value::Array(arr) => Ok(arr.len()),
            _ => Ok(1), // Single value counts as 1 feature
        }
    }

    fn calculate_complexity(&self, data: &Value) -> Result<f64> {
        let complexity = match data {
            Value::Object(obj) => {
                let nested_objects = obj.values()
                    .filter(|v| matches!(v, Value::Object(_) | Value::Array(_)))
                    .count();
                (nested_objects as f64 * 0.2).min(1.0)
            },
            Value::Array(arr) => {
                if arr.len() > 10 {
                    0.8
                } else if arr.len() > 5 {
                    0.5
                } else {
                    0.2
                }
            },
            Value::String(s) => {
                if s.len() > 100 {
                    0.6
                } else {
                    0.3
                }
            },
            _ => 0.1,
        };
        
        Ok(complexity)
    }

    pub fn get_model_metadata(&self) -> Value {
        serde_json::json!({
            "model_type": "hardcoded",
            "version": self.version,
            "features": [
                "feature_count",
                "complexity_score",
                "randomness"
            ],
            "score_range": "0-100",
            "description": "Hardcoded scoring model for ZK-Persona proof of concept"
        })
    }
}