use uuid::Uuid;

use crate::application::use_cases::PatchUseCases;
use crate::domain::PatchProposal;
use crate::Result;

pub struct PatchHandler {
    use_cases: PatchUseCases,
}

impl PatchHandler {
    pub fn new(use_cases: PatchUseCases) -> Self {
        Self { use_cases }
    }

    pub async fn get_patch(&self, id: Uuid) -> Result<PatchProposal> {
        self.use_cases.get_patch(id).await
    }

    pub async fn create_patch(&self, developer_id: i64, patch: &PatchProposal) -> Result<PatchProposal> {
        self.use_cases.create_patch(developer_id, patch).await
    }

    pub async fn update_patch(&self, patch: &PatchProposal) -> Result<PatchProposal> {
        self.use_cases.update_patch(patch).await
    }

    pub async fn delete_patch(&self, id: Uuid) -> Result<()> {
        self.use_cases.delete_patch(id).await
    }

    // TODO: Add more handler methods as needed
}