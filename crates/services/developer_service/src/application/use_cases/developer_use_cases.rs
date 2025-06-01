use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{Developer, DeveloperRepository};
use crate::Result;

pub struct DeveloperUseCases {
    repository: Arc<dyn DeveloperRepository>,
}

impl DeveloperUseCases {
    pub fn new(repository: Arc<dyn DeveloperRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_developer(&self, id: Uuid) -> Result<Developer> {
        self.repository.get_by_id(id).await
    }

    pub async fn get_developer_by_username(&self, username: &str) -> Result<Developer> {
        self.repository.get_by_username(username).await
    }

    pub async fn get_developer_by_email(&self, email: &str) -> Result<Developer> {
        self.repository.get_by_email(email).await
    }

    pub async fn create_developer(&self, developer: &Developer) -> Result<Developer> {
        self.repository.create(developer).await
    }

    pub async fn update_developer(&self, developer: &Developer) -> Result<Developer> {
        self.repository.update(developer).await
    }

    pub async fn delete_developer(&self, id: Uuid) -> Result<()> {
        self.repository.delete(id).await
    }

    // TODO: Add more use case methods as needed
}