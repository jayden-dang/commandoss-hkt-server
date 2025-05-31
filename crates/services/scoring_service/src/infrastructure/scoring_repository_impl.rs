use async_trait::async_trait;
use jd_core::{AppState, base};
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ScoringResult;

use crate::{
    ScoringResultDmc,
    domain::scoring_repository_trait::ScoringRepository,
    models::{
        requests::ScoringQueryRequest,
        responses::{ScoringResponse, ScoringListResponse},
        ScoringResultRecord, ScoringResultForCreate, ScoringResultFilter,
    },
    Result,
};

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
        let behavior_input_uuid = result.behavior_input_id.to_uuid();
            
        let create_req = ScoringResultForCreate {
            behavior_input_id: behavior_input_uuid,
            score: result.score,
            model_version: result.model_version,
        };
        
        let record = base::rest::create::<ScoringResultDmc, _, ScoringResultRecord>(
            &self.app_state.mm, 
            create_req
        ).await?;
            
        Ok(ScoringResponse::from(record))
    }

    async fn get_scoring_result(&self, id: Id) -> Result<Option<ScoringResponse>> {
        let id_uuid = id.to_uuid();
            
        match base::rest::get_by_id::<ScoringResultDmc, ScoringResultRecord>(&self.app_state.mm, id_uuid).await {
            Ok(record) => Ok(Some(ScoringResponse::from(record))),
            Err(_) => Ok(None),
        }
    }

    async fn get_scoring_by_behavior_id(&self, behavior_input_id: Id) -> Result<Option<ScoringResponse>> {
        let behavior_input_uuid = behavior_input_id.to_uuid();
            
        let filter = ScoringResultFilter {
            behavior_input_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(behavior_input_uuid.to_string()))].into()),
            model_version: None,
        };
        
        // Use first to get the most recent result (ordered by timestamp DESC by default)
        let list_options = modql::filter::ListOptions {
            limit: Some(1),
            offset: None,
            order_bys: Some("!timestamp".into()), // ! prefix for descending
        };
        
        match base::rest::first::<ScoringResultDmc, _, ScoringResultRecord>(&self.app_state.mm, Some(filter), Some(list_options)).await {
            Ok(Some(record)) => Ok(Some(ScoringResponse::from(record))),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    async fn list_scoring_results(&self, query_req: ScoringQueryRequest) -> Result<ScoringListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        let filter = match (&query_req.behavior_input_id, &query_req.model_version) {
            (Some(behavior_input_id), Some(model_version)) => {
                let behavior_input_uuid = behavior_input_id.to_uuid();
                Some(ScoringResultFilter {
                    behavior_input_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(behavior_input_uuid.to_string()))].into()),
                    model_version: Some(model_version.clone().into()),
                })
            },
            (Some(behavior_input_id), None) => {
                let behavior_input_uuid = behavior_input_id.to_uuid();
                Some(ScoringResultFilter {
                    behavior_input_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(behavior_input_uuid.to_string()))].into()),
                    model_version: None,
                })
            },
            (None, Some(model_version)) => {
                Some(ScoringResultFilter {
                    behavior_input_id: None,
                    model_version: Some(model_version.clone().into()),
                })
            },
            (None, None) => None,
        };
        
        let list_options = modql::filter::ListOptions {
            limit: Some(limit as i64),
            offset: Some(offset as i64),
            order_bys: Some("!timestamp".into()), // ! prefix for descending
        };
        
        let (records, meta) = base::rest::list::<ScoringResultDmc, _, ScoringResultRecord>(
            &self.app_state.mm, 
            filter, 
            Some(list_options)
        ).await?;
        
        let items: Vec<ScoringResponse> = records
            .into_iter()
            .map(ScoringResponse::from)
            .collect();
        
        Ok(ScoringListResponse {
            items,
            total: meta.total_items(),
            limit,
            offset,
        })
    }
}