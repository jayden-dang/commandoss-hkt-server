use async_trait::async_trait;
use jd_core::AppState;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ScoringResult;
use jd_utils::time::now_utc;
use sqlx::Row;

use crate::domain::scoring_repository_trait::ScoringRepository;
use crate::models::{
    requests::ScoringQueryRequest,
    responses::{ScoringResponse, ScoringListResponse},
};
use crate::Result;

#[derive(Clone)]
pub struct ScoringRepositoryImpl {
    app_state: AppState,
}

impl ScoringRepositoryImpl {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl ScoringRepository for ScoringRepositoryImpl {
    async fn create_scoring_result(&self, result: ScoringResult) -> Result<ScoringResponse> {
        let id = Id::generate();
        let timestamp = now_utc();
        
        let query = r#"
            INSERT INTO scoring_results (id, behavior_input_id, score, model_version, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, behavior_input_id, score, model_version, timestamp
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .bind(result.behavior_input_id.value())
            .bind(result.score)
            .bind(&result.model_version)
            .bind(timestamp)
            .fetch_one(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(ScoringResponse {
            id: Id::new(row.get::<uuid::Uuid, _>("id").to_string()),
            behavior_input_id: Id::new(row.get::<uuid::Uuid, _>("behavior_input_id").to_string()),
            score: row.get("score"),
            model_version: row.get("model_version"),
            timestamp: row.get("timestamp"),
            metadata: None,
        })
    }

    async fn get_scoring_result(&self, id: Id) -> Result<Option<ScoringResponse>> {
        let query = r#"
            SELECT id, behavior_input_id, score, model_version, timestamp
            FROM scoring_results
            WHERE id = $1
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .fetch_optional(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(row.map(|r| ScoringResponse {
            id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
            behavior_input_id: Id::new(r.get::<uuid::Uuid, _>("behavior_input_id").to_string()),
            score: r.get("score"),
            model_version: r.get("model_version"),
            timestamp: r.get("timestamp"),
            metadata: None,
        }))
    }

    async fn get_scoring_by_behavior_id(&self, behavior_input_id: Id) -> Result<Option<ScoringResponse>> {
        let query = r#"
            SELECT id, behavior_input_id, score, model_version, timestamp
            FROM scoring_results
            WHERE behavior_input_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
        "#;
        
        let row = sqlx::query(query)
            .bind(behavior_input_id.value())
            .fetch_optional(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(row.map(|r| ScoringResponse {
            id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
            behavior_input_id: Id::new(r.get::<uuid::Uuid, _>("behavior_input_id").to_string()),
            score: r.get("score"),
            model_version: r.get("model_version"),
            timestamp: r.get("timestamp"),
            metadata: None,
        }))
    }

    async fn list_scoring_results(&self, query_req: ScoringQueryRequest) -> Result<ScoringListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        // Build query and execute based on filters
        let (rows, total) = match (&query_req.behavior_input_id, &query_req.model_version) {
            (Some(behavior_input_id), Some(model_version)) => {
                // Both filters
                let query = r#"
                    SELECT id, behavior_input_id, score, model_version, timestamp
                    FROM scoring_results
                    WHERE behavior_input_id = $1 AND model_version = $2
                    ORDER BY timestamp DESC
                    LIMIT $3 OFFSET $4
                "#;
                let rows = sqlx::query(query)
                    .bind(behavior_input_id.value())
                    .bind(model_version)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query(
                    "SELECT COUNT(*) as count FROM scoring_results WHERE behavior_input_id = $1 AND model_version = $2"
                )
                .bind(behavior_input_id.value())
                .bind(model_version)
                .fetch_one(self.app_state.mm.dbx().db())
                .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (Some(behavior_input_id), None) => {
                // Only behavior_input_id filter
                let query = r#"
                    SELECT id, behavior_input_id, score, model_version, timestamp
                    FROM scoring_results
                    WHERE behavior_input_id = $1
                    ORDER BY timestamp DESC
                    LIMIT $2 OFFSET $3
                "#;
                let rows = sqlx::query(query)
                    .bind(behavior_input_id.value())
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM scoring_results WHERE behavior_input_id = $1")
                    .bind(behavior_input_id.value())
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (None, Some(model_version)) => {
                // Only model_version filter
                let query = r#"
                    SELECT id, behavior_input_id, score, model_version, timestamp
                    FROM scoring_results
                    WHERE model_version = $1
                    ORDER BY timestamp DESC
                    LIMIT $2 OFFSET $3
                "#;
                let rows = sqlx::query(query)
                    .bind(model_version)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM scoring_results WHERE model_version = $1")
                    .bind(model_version)
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (None, None) => {
                // No filters
                let query = r#"
                    SELECT id, behavior_input_id, score, model_version, timestamp
                    FROM scoring_results
                    ORDER BY timestamp DESC
                    LIMIT $1 OFFSET $2
                "#;
                let rows = sqlx::query(query)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM scoring_results")
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            }
        };
        
        let items: Vec<ScoringResponse> = rows
            .into_iter()
            .map(|r| ScoringResponse {
                id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
                behavior_input_id: Id::new(r.get::<uuid::Uuid, _>("behavior_input_id").to_string()),
                score: r.get("score"),
                model_version: r.get("model_version"),
                timestamp: r.get("timestamp"),
                metadata: None,
            })
            .collect();
        
        Ok(ScoringListResponse {
            items,
            total: total as u64,
            limit,
            offset,
        })
    }
}