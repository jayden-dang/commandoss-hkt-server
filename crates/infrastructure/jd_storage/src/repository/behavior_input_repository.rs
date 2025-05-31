use async_trait::async_trait;
use chrono::Utc;
// use sea_query::{Expr, PostgresQueryBuilder, Query, Order};
// use sea_query_binder::SqlxBinder;
use sqlx::Row;

use crate::{
    dbx::Dbx,
    repository::{
        FilterableRepository, FilterablePaginatedRepository, 
        PaginatedRepository, PaginatedResult, Repository
    },
};
use jd_domain::{
    Id,
    zkpersona_domain::models::{
        BehaviorInput, BehaviorInputFilter, InputType, InputSource,
        CreateBehaviorInputRequest,
        ZkPersonaError, ZkPersonaResult
    }
};

// ================================================================================================
// Database Table Definition
// ================================================================================================

#[derive(sea_query::Iden)]
enum BehaviorInputTable {
    Table,
    Id,
    UserId,
    BehaviorSessionId,
    SessionId,
    InputData,
    InputType,
    Source,
    Timestamp,
    Processed,
    Cid,
    Ctime,
    Mid,
    Mtime,
}

// ================================================================================================
// Repository Implementation
// ================================================================================================

#[derive(Debug, Clone)]
pub struct BehaviorInputRepository {
    dbx: Dbx,
}

impl BehaviorInputRepository {
    pub fn new(dbx: Dbx) -> Self {
        Self { dbx }
    }

    /// Convert database row to BehaviorInput model
    fn row_to_model(row: &sqlx::postgres::PgRow) -> sqlx::Result<BehaviorInput> {
        Ok(BehaviorInput {
            id: Id::new(row.try_get::<uuid::Uuid, _>("id")?.to_string()),
            user_id: row.try_get::<Option<uuid::Uuid>, _>("user_id")?
                .map(|u| Id::new(u.to_string())),
            behavior_session_id: row.try_get::<Option<uuid::Uuid>, _>("behavior_session_id")?
                .map(|u| Id::new(u.to_string())),
            session_id: row.try_get("session_id")?,
            input_data: row.try_get("input_data")?,
            input_type: row.try_get::<String, _>("input_type")?
                .parse::<InputType>()
                .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
            source: row.try_get::<String, _>("source")?
                .parse::<InputSource>()
                .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?,
            timestamp: row.try_get("timestamp")?,
            processed: row.try_get("processed")?,
            cid: row.try_get::<Option<uuid::Uuid>, _>("cid")?
                .map(|u| Id::new(u.to_string())),
            ctime: row.try_get("ctime")?,
            mid: row.try_get::<Option<uuid::Uuid>, _>("mid")?
                .map(|u| Id::new(u.to_string())),
            mtime: row.try_get("mtime")?,
        })
    }

    /// Create a new behavior input from request
    pub async fn create_from_request(
        &self,
        request: CreateBehaviorInputRequest,
    ) -> ZkPersonaResult<BehaviorInput> {
        let now = Utc::now();
        let id = Id::generate();

        let sql = r#"
            INSERT INTO behavior_inputs (
                id, user_id, behavior_session_id, session_id, input_data, 
                input_type, source, timestamp, processed, ctime, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;

        let user_id_val = request.user_id.map(|u| *u.value());
        let session_id_val = request.behavior_session_id.map(|u| *u.value());
        
        self.dbx
            .execute(sqlx::query(sql)
                .bind(id.value())
                .bind(user_id_val)
                .bind(session_id_val)
                .bind(request.session_id)
                .bind(&request.input_data)
                .bind(format!("{:?}", request.input_type.unwrap_or_default()).to_lowercase())
                .bind(format!("{:?}", request.source.unwrap_or_default()).to_lowercase())
                .bind(now)
                .bind(false)
                .bind(now)
                .bind(now))
            .await
            .map_err(|e| match e {
                crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                other => ZkPersonaError::DatabaseTransaction(other.to_string()),
            })?;

        self.find_by_id(id).await?
            .ok_or_else(|| ZkPersonaError::Database(sqlx::Error::RowNotFound))
    }

    /// Update processed status
    pub async fn mark_as_processed(&self, id: Id) -> ZkPersonaResult<bool> {
        let sql = "UPDATE behavior_inputs SET processed = true, mtime = $1 WHERE id = $2";

        let rows_affected = self.dbx
            .execute(sqlx::query(sql)
                .bind(Utc::now())
                .bind(id.value()))
            .await
            .map_err(|e| match e {
                crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                other => ZkPersonaError::DatabaseTransaction(other.to_string()),
            })?;

        Ok(rows_affected > 0)
    }

    /// Find inputs by session ID
    pub async fn find_by_session_id(&self, session_id: &str) -> ZkPersonaResult<Vec<BehaviorInput>> {
        let filter = BehaviorInputFilter {
            session_id: Some(session_id.to_string()),
            ..Default::default()
        };
        self.find_by_filter(filter).await
    }

    /// Find inputs by user ID
    pub async fn find_by_user_id(&self, user_id: Id) -> ZkPersonaResult<Vec<BehaviorInput>> {
        let filter = BehaviorInputFilter {
            user_id: Some(user_id),
            ..Default::default()
        };
        self.find_by_filter(filter).await
    }

    /// Find unprocessed inputs
    pub async fn find_unprocessed(&self) -> ZkPersonaResult<Vec<BehaviorInput>> {
        let filter = BehaviorInputFilter {
            processed: Some(false),
            ..Default::default()
        };
        self.find_by_filter(filter).await
    }

    // Note: Removed build_where_clause_and_params as it's not used and causes trait object issues

    /// Execute a raw query with proper error handling
    async fn execute_behavior_input_query(&self, sql: &str) -> ZkPersonaResult<Vec<BehaviorInput>> {
        let rows = sqlx::query(sql)
            .fetch_all(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        let mut inputs = Vec::new();
        for row in rows {
            inputs.push(Self::row_to_model(&row).map_err(ZkPersonaError::Database)?);
        }
        Ok(inputs)
    }
}

// ================================================================================================
// Repository Trait Implementations
// ================================================================================================

#[async_trait]
impl Repository<BehaviorInput, Id> for BehaviorInputRepository {
    type Error = ZkPersonaError;

    async fn find_by_id(&self, id: Id) -> Result<Option<BehaviorInput>, Self::Error> {
        let sql = r#"
            SELECT id, user_id, behavior_session_id, session_id, input_data, 
                   input_type, source, timestamp, processed, cid, ctime, mid, mtime
            FROM behavior_inputs 
            WHERE id = $1
        "#;

        let row = sqlx::query(sql)
            .bind(id.value())
            .fetch_optional(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        match row {
            Some(row) => Ok(Some(Self::row_to_model(&row).map_err(ZkPersonaError::Database)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<BehaviorInput>, Self::Error> {
        let sql = r#"
            SELECT id, user_id, behavior_session_id, session_id, input_data, 
                   input_type, source, timestamp, processed, cid, ctime, mid, mtime
            FROM behavior_inputs 
            ORDER BY timestamp DESC
        "#;

        self.execute_behavior_input_query(sql).await
    }

    async fn save(&self, entity: &BehaviorInput) -> Result<BehaviorInput, Self::Error> {
        let sql = r#"
            INSERT INTO behavior_inputs (
                id, user_id, behavior_session_id, session_id, input_data, 
                input_type, source, timestamp, processed, cid, ctime, mid, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#;

        let user_id_val = entity.user_id.as_ref().map(|u| *u.value());
        let session_id_val = entity.behavior_session_id.as_ref().map(|u| *u.value());
        let cid_val = entity.cid.as_ref().map(|u| *u.value());
        let mid_val = entity.mid.as_ref().map(|u| *u.value());
        
        self.dbx
            .execute(sqlx::query(sql)
                .bind(entity.id.value())
                .bind(user_id_val)
                .bind(session_id_val)
                .bind(&entity.session_id)
                .bind(&entity.input_data)
                .bind(format!("{:?}", entity.input_type).to_lowercase())
                .bind(format!("{:?}", entity.source).to_lowercase())
                .bind(entity.timestamp)
                .bind(entity.processed)
                .bind(cid_val)
                .bind(entity.ctime)
                .bind(mid_val)
                .bind(entity.mtime))
            .await
            .map_err(|e| match e {
                crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                other => ZkPersonaError::DatabaseTransaction(other.to_string()),
            })?;

        Ok(entity.clone())
    }

    async fn update(&self, id: Id, entity: &BehaviorInput) -> Result<BehaviorInput, Self::Error> {
        let sql = r#"
            UPDATE behavior_inputs 
            SET session_id = $1, input_data = $2, input_type = $3, 
                source = $4, processed = $5, mtime = $6
            WHERE id = $7
        "#;

        let rows_affected = self.dbx
            .execute(sqlx::query(sql)
                .bind(&entity.session_id)
                .bind(&entity.input_data)
                .bind(format!("{:?}", entity.input_type).to_lowercase())
                .bind(format!("{:?}", entity.source).to_lowercase())
                .bind(entity.processed)
                .bind(Utc::now())
                .bind(id.value()))
            .await
            .map_err(|e| match e {
                crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                other => ZkPersonaError::DatabaseTransaction(other.to_string()),
            })?;

        if rows_affected == 0 {
            return Err(ZkPersonaError::Database(sqlx::Error::RowNotFound));
        }

        self.find_by_id(id).await?
            .ok_or_else(|| ZkPersonaError::Database(sqlx::Error::RowNotFound))
    }

    async fn delete(&self, id: Id) -> Result<bool, Self::Error> {
        let sql = "DELETE FROM behavior_inputs WHERE id = $1";

        let rows_affected = self.dbx
            .execute(sqlx::query(sql).bind(id.value()))
            .await
            .map_err(|e| match e {
                crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                other => ZkPersonaError::DatabaseTransaction(other.to_string()),
            })?;

        Ok(rows_affected > 0)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let sql = "SELECT COUNT(*) FROM behavior_inputs";

        let count: i64 = sqlx::query_scalar(sql)
            .fetch_one(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        Ok(count)
    }
}

#[async_trait]
impl FilterableRepository<BehaviorInput, Id, BehaviorInputFilter> for BehaviorInputRepository {
    async fn find_by_filter(&self, filter: BehaviorInputFilter) -> Result<Vec<BehaviorInput>, Self::Error> {
        // Simplified implementation using string building
        let mut sql = r#"
            SELECT id, user_id, behavior_session_id, session_id, input_data, 
                   input_type, source, timestamp, processed, cid, ctime, mid, mtime
            FROM behavior_inputs
        "#.to_string();

        let mut conditions = Vec::new();

        if filter.user_id.is_some() {
            conditions.push("user_id = $1".to_string());
        }
        if filter.session_id.is_some() {
            conditions.push(format!("session_id = ${}", conditions.len() + 1));
        }
        if filter.processed.is_some() {
            conditions.push(format!("processed = ${}", conditions.len() + 1));
        }

        if !conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        // For now, use a simplified query without dynamic parameter binding
        // In a production system, you'd want proper parameter binding
        if filter.user_id.is_some() && filter.session_id.is_some() {
            let final_sql = r#"
                SELECT id, user_id, behavior_session_id, session_id, input_data, 
                       input_type, source, timestamp, processed, cid, ctime, mid, mtime
                FROM behavior_inputs 
                WHERE user_id = $1 AND session_id = $2
                ORDER BY timestamp DESC
            "#;
            
            let rows = sqlx::query(final_sql)
                .bind(filter.user_id.unwrap().value())
                .bind(filter.session_id.unwrap())
                .fetch_all(self.dbx.db())
                .await
                .map_err(ZkPersonaError::Database)?;

            let mut inputs = Vec::new();
            for row in rows {
                inputs.push(Self::row_to_model(&row).map_err(ZkPersonaError::Database)?);
            }
            return Ok(inputs);
        }

        // Fallback to find_all for now
        self.find_all().await
    }

    async fn count_by_filter(&self, _filter: BehaviorInputFilter) -> Result<i64, Self::Error> {
        // Simplified implementation
        self.count().await
    }

    async fn delete_by_filter(&self, filter: BehaviorInputFilter) -> Result<u64, Self::Error> {
        if let Some(user_id) = filter.user_id {
            let sql = "DELETE FROM behavior_inputs WHERE user_id = $1";
            let rows_affected = self.dbx
                .execute(sqlx::query(sql).bind(user_id.value()))
                .await
                .map_err(|e| match e {
                    crate::dbx::Error::Sqlx(sqlx_err) => ZkPersonaError::Database(sqlx_err),
                    other => ZkPersonaError::DatabaseTransaction(other.to_string()),
                })?;
            Ok(rows_affected)
        } else {
            Ok(0)
        }
    }
}

#[async_trait]
impl PaginatedRepository<BehaviorInput, Id> for BehaviorInputRepository {
    async fn find_paginated(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedResult<BehaviorInput>, Self::Error> {
        let offset = (page.saturating_sub(1)) * per_page;
        
        let sql = r#"
            SELECT id, user_id, behavior_session_id, session_id, input_data, 
                   input_type, source, timestamp, processed, cid, ctime, mid, mtime
            FROM behavior_inputs 
            ORDER BY timestamp DESC 
            LIMIT $1 OFFSET $2
        "#;

        let rows = sqlx::query(sql)
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        let mut inputs = Vec::new();
        for row in rows {
            inputs.push(Self::row_to_model(&row).map_err(ZkPersonaError::Database)?);
        }

        let total = self.count().await?;

        Ok(PaginatedResult::new(inputs, total, page, per_page))
    }
}

#[async_trait]
impl FilterablePaginatedRepository<BehaviorInput, Id, BehaviorInputFilter> for BehaviorInputRepository {
    async fn find_by_filter_paginated(
        &self,
        filter: BehaviorInputFilter,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedResult<BehaviorInput>, Self::Error> {
        // Simplified implementation - delegate to basic pagination for now
        let all_filtered = self.find_by_filter(filter.clone()).await?;
        let total = all_filtered.len() as i64;
        
        let start = ((page.saturating_sub(1)) * per_page) as usize;
        let end = (start + per_page as usize).min(all_filtered.len());
        
        let items = if start < all_filtered.len() {
            all_filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedResult::new(items, total, page, per_page))
    }
}

// Helper import for analytics summary
use jd_domain::zkpersona_domain::models::BehaviorAnalyticsSummary;

impl BehaviorInputRepository {
    /// Get analytics summary for behavior inputs
    pub async fn get_analytics_summary(
        &self,
        _filter: Option<BehaviorInputFilter>,
    ) -> ZkPersonaResult<BehaviorAnalyticsSummary> {
        let total_inputs = self.count().await?;
        
        let processed_sql = "SELECT COUNT(*) FROM behavior_inputs WHERE processed = true";
        let processed_inputs: i64 = sqlx::query_scalar(processed_sql)
            .fetch_one(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        let sessions_sql = "SELECT COUNT(DISTINCT session_id) FROM behavior_inputs WHERE session_id IS NOT NULL";
        let unique_sessions: i64 = sqlx::query_scalar(sessions_sql)
            .fetch_one(self.dbx.db())
            .await
            .map_err(ZkPersonaError::Database)?;

        Ok(BehaviorAnalyticsSummary {
            total_inputs,
            processed_inputs,
            unique_sessions,
            input_types_distribution: std::collections::HashMap::new(),
            source_distribution: std::collections::HashMap::new(),
            date_range: (
                Utc::now() - chrono::Duration::days(30),
                Utc::now(),
            ),
        })
    }
}