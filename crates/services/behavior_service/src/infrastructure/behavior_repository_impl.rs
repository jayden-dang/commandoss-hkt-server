use async_trait::async_trait;
use jd_core::AppState;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::BehaviorInput;
use jd_utils::time::now_utc;
use sqlx::Row;

use crate::domain::behavior_repository_trait::BehaviorRepository;
use crate::models::{
    requests::BehaviorQueryRequest,
    responses::{BehaviorInputResponse, BehaviorListResponse},
};
use crate::Result;

#[derive(Clone)]
pub struct BehaviorRepositoryImpl {
    app_state: AppState,
}

impl BehaviorRepositoryImpl {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl BehaviorRepository for BehaviorRepositoryImpl {
    async fn create_behavior_input(&self, input: BehaviorInput) -> Result<BehaviorInputResponse> {
        let id = Id::generate();
        let timestamp = now_utc();
        
        let query = r#"
            INSERT INTO behavior_inputs (id, session_id, input_data, timestamp, processed)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, session_id, input_data, timestamp, processed
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .bind(&input.session_id)
            .bind(&input.input_data)
            .bind(timestamp)
            .bind(false)
            .fetch_one(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(BehaviorInputResponse {
            id: Id::new(row.get::<uuid::Uuid, _>("id").to_string()),
            session_id: row.get("session_id"),
            input_data: row.get("input_data"),
            timestamp: row.get("timestamp"),
            processed: row.get("processed"),
        })
    }

    async fn get_behavior_input(&self, id: Id) -> Result<Option<BehaviorInputResponse>> {
        let query = r#"
            SELECT id, session_id, input_data, timestamp, processed
            FROM behavior_inputs
            WHERE id = $1
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .fetch_optional(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(row.map(|r| BehaviorInputResponse {
            id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
            session_id: r.get("session_id"),
            input_data: r.get("input_data"),
            timestamp: r.get("timestamp"),
            processed: r.get("processed"),
        }))
    }

    async fn list_behavior_inputs(&self, query_req: BehaviorQueryRequest) -> Result<BehaviorListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        // Build query and execute based on filters
        let (rows, total) = if let Some(ref session_id) = query_req.session_id {
            let query = r#"
                SELECT id, session_id, input_data, timestamp, processed
                FROM behavior_inputs
                WHERE session_id = $1
                ORDER BY timestamp DESC
                LIMIT $2 OFFSET $3
            "#;
            let rows = sqlx::query(query)
                .bind(session_id)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(self.app_state.mm.dbx().db())
                .await?;

            let count_row = sqlx::query("SELECT COUNT(*) as count FROM behavior_inputs WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(self.app_state.mm.dbx().db())
                .await?;
            let total: i64 = count_row.get("count");
            (rows, total)
        } else {
            let query = r#"
                SELECT id, session_id, input_data, timestamp, processed
                FROM behavior_inputs
                ORDER BY timestamp DESC
                LIMIT $1 OFFSET $2
            "#;
            let rows = sqlx::query(query)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(self.app_state.mm.dbx().db())
                .await?;

            let count_row = sqlx::query("SELECT COUNT(*) as count FROM behavior_inputs")
                .fetch_one(self.app_state.mm.dbx().db())
                .await?;
            let total: i64 = count_row.get("count");
            (rows, total)
        };
        
        let items: Vec<BehaviorInputResponse> = rows
            .into_iter()
            .map(|r| BehaviorInputResponse {
                id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
                session_id: r.get("session_id"),
                input_data: r.get("input_data"),
                timestamp: r.get("timestamp"),
                processed: r.get("processed"),
            })
            .collect();
        
        Ok(BehaviorListResponse {
            items,
            total: total as u64,
            limit,
            offset,
        })
    }

    async fn mark_as_processed(&self, id: Id) -> Result<()> {
        let query = r#"
            UPDATE behavior_inputs
            SET processed = true
            WHERE id = $1
        "#;
        
        sqlx::query(query)
            .bind(id.value())
            .execute(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(())
    }
}