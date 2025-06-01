use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{PatchProposal, PatchRepository};
use crate::Result;

pub struct PatchUseCases {
    repository: Arc<dyn PatchRepository>,
}

impl PatchUseCases {
    pub fn new(repository: Arc<dyn PatchRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_patch(&self, id: Uuid) -> Result<PatchProposal> {
        self.repository.get_by_id(id).await
    }

    pub async fn create_patch(&self, developer_id: i64, patch: &PatchProposal) -> Result<PatchProposal> {
        self.repository.create(patch).await
    }

    pub async fn update_patch(&self, patch: &PatchProposal) -> Result<PatchProposal> {
        self.repository.update(patch).await
    }

    pub async fn delete_patch(&self, id: Uuid) -> Result<()> {
        self.repository.delete(id).await
    }

    // TODO: Add more use case methods as needed
}