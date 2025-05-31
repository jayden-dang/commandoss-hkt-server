use async_trait::async_trait;
use jd_core::{AppState, base};
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::BehaviorInput;

use crate::{
    BehaviorInputDmc,
    domain::behavior_repository_trait::BehaviorRepository,
    models::{
        requests::BehaviorQueryRequest,
        responses::{BehaviorInputResponse, BehaviorListResponse},
        BehaviorInputRecord, BehaviorInputForCreate, BehaviorInputForUpdate, BehaviorInputFilter,
    },
    Result,
};

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
        let create_req = BehaviorInputForCreate {
            session_id: input.session_id,
            input_data: serde_json::to_string(&input.input_data)
                .map_err(|e| crate::Error::Serialization(e))?,
            processed: Some(false),
        };
        
        let record = base::rest::create::<BehaviorInputDmc, _, BehaviorInputRecord>(
            &self.app_state.mm, 
            create_req
        ).await?;
            
        Ok(BehaviorInputResponse::from(record))
    }

    async fn get_behavior_input(&self, id: Id) -> Result<Option<BehaviorInputResponse>> {
        let id_uuid = id.to_uuid();
            
        match base::rest::get_by_id::<BehaviorInputDmc, BehaviorInputRecord>(&self.app_state.mm, id_uuid).await {
            Ok(record) => Ok(Some(BehaviorInputResponse::from(record))),
            Err(_) => Ok(None),
        }
    }

    async fn list_behavior_inputs(&self, query_req: BehaviorQueryRequest) -> Result<BehaviorListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        let filter = if let Some(session_id) = query_req.session_id {
            Some(BehaviorInputFilter {
                session_id: Some(session_id.into()),
                processed: None,
            })
        } else {
            None
        };
        
        let list_options = modql::filter::ListOptions {
            limit: Some(limit as i64),
            offset: Some(offset as i64),
            order_bys: Some("!timestamp".into()), // ! prefix for descending
        };
        
        let (records, meta) = base::rest::list::<BehaviorInputDmc, _, BehaviorInputRecord>(
            &self.app_state.mm, 
            filter, 
            Some(list_options)
        ).await?;
        
        let items: Vec<BehaviorInputResponse> = records
            .into_iter()
            .map(BehaviorInputResponse::from)
            .collect();
        
        Ok(BehaviorListResponse {
            items,
            total: meta.total_items(),
            limit,
            offset,
        })
    }

    async fn mark_as_processed(&self, id: Id) -> Result<()> {
        let id_uuid = id.to_uuid();
            
        let update_req = BehaviorInputForUpdate {
            processed: Some(true),
        };
        
        base::rest::update::<BehaviorInputDmc, _>(&self.app_state.mm, id_uuid, update_req)
            .await?;
            
        Ok(())
    }
}