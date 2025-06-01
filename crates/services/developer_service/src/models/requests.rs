use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::{Skill, SortBy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeveloperRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub github_username: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDeveloperRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSocialLinksRequest {
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub linkedin: Option<String>,
    pub personal_site: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSkillRequest {
    pub skill: Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSkillRequest {
    pub skill_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDevelopersRequest {
    pub query: Option<String>,
    pub skills: Option<Vec<String>>,
    pub min_reputation: Option<Decimal>,
    pub verified_only: Option<bool>,
    pub sort_by: Option<SortBy>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for SearchDevelopersRequest {
    fn default() -> Self {
        Self {
            query: None,
            skills: None,
            min_reputation: None,
            verified_only: Some(false),
            sort_by: Some(SortBy::ReputationDesc),
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyDeveloperRequest {
    pub proof_data: String,
    pub verification_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetActivitiesRequest {
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLeaderboardRequest {
    pub limit: Option<u32>,
    pub skill_filter: Option<String>,
}