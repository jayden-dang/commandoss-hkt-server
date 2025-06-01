use uuid::Uuid;

use crate::application::use_cases::DeveloperUseCases;
use crate::domain::Developer;
use crate::Result;

pub struct DeveloperHandler {
    use_cases: DeveloperUseCases,
}

impl DeveloperHandler {
    pub fn new(use_cases: DeveloperUseCases) -> Self {
        Self { use_cases }
    }

    pub async fn get_developer(&self, id: Uuid) -> Result<Developer> {
        self.use_cases.get_developer(id).await
    }

    pub async fn get_developer_by_username(&self, username: &str) -> Result<Developer> {
        self.use_cases.get_developer_by_username(username).await
    }

    pub async fn get_developer_by_email(&self, email: &str) -> Result<Developer> {
        self.use_cases.get_developer_by_email(email).await
    }

    // TODO: Add more handler methods as needed
}