use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::AnalyticsRepository;
use crate::models::*;
use crate::Result;

pub struct AnalyticsUseCases {
    repository: Arc<dyn AnalyticsRepository>,
}

impl AnalyticsUseCases {
    pub fn new(repository: Arc<dyn AnalyticsRepository>) -> Self {
        Self { repository }
    }

    // Developer Analytics
    pub async fn get_developer_analytics(&self, developer_id: Uuid) -> Result<DeveloperAnalyticsResponse> {
        let analytics = self.repository.get_developer_analytics(developer_id).await?;
        
        Ok(DeveloperAnalyticsResponse {
            analytics,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_reputation_history(
        &self,
        developer_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ReputationHistoryResponse> {
        let history = self
            .repository
            .get_reputation_history(developer_id, start_date, end_date)
            .await?;
        
        let history_points = history
            .into_iter()
            .map(|(date, score)| ReputationPoint { date, score })
            .collect();
        
        Ok(ReputationHistoryResponse {
            developer_id,
            history: history_points,
        })
    }

    pub async fn refresh_reputation_score(
        &self,
        developer_id: Uuid,
        force_recalculation: bool,
    ) -> Result<RefreshReputationResponse> {
        // Get current score first
        let current_analytics = self.repository.get_developer_analytics(developer_id).await?;
        let previous_score = current_analytics.reputation_score;
        
        // Refresh the score
        let new_score = self.repository.refresh_reputation_score(developer_id).await?;
        
        Ok(RefreshReputationResponse {
            developer_id,
            new_score,
            previous_score,
            updated_at: Utc::now(),
        })
    }

    // Repository Analytics
    pub async fn get_repository_analytics(&self, repository_id: Uuid) -> Result<RepositoryAnalyticsResponse> {
        let analytics = self.repository.get_repository_analytics(repository_id).await?;
        
        Ok(RepositoryAnalyticsResponse {
            analytics,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_repository_trends(
        &self,
        repository_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<RepositoryTrendsResponse> {
        let trends = self
            .repository
            .get_repository_trends(repository_id, start_date, end_date)
            .await?;
        
        Ok(RepositoryTrendsResponse {
            repository_id,
            trends,
            period_start: start_date,
            period_end: end_date,
        })
    }

    pub async fn get_repository_contributors(
        &self,
        repository_id: Uuid,
    ) -> Result<RepositoryContributorsResponse> {
        let contributors = self
            .repository
            .get_repository_contributors(repository_id)
            .await?;
        
        let contributor_infos = contributors
            .into_iter()
            .map(|(developer_id, username, contribution_score)| ContributorInfo {
                developer_id,
                username,
                contribution_score,
                commits_count: 0, // TODO: Fetch from repository
                issues_resolved: 0, // TODO: Fetch from repository
            })
            .collect();
        
        Ok(RepositoryContributorsResponse {
            repository_id,
            contributors: contributor_infos,
        })
    }

    // Team Analytics
    pub async fn get_team_performance(&self, team_id: Uuid) -> Result<TeamPerformanceResponse> {
        let analytics = self.repository.get_team_performance(team_id).await?;
        
        Ok(TeamPerformanceResponse {
            analytics,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_team_skills(&self, team_id: Uuid) -> Result<TeamSkillsResponse> {
        let skills = self.repository.get_team_skills(team_id).await?;
        let total_members = skills.iter().map(|s| s.member_count).sum();
        
        Ok(TeamSkillsResponse {
            team_id,
            skills,
            total_members,
        })
    }

    pub async fn get_team_collaboration(&self, team_id: Uuid) -> Result<TeamCollaborationResponse> {
        let metrics = self.repository.get_team_collaboration_metrics(team_id).await?;
        
        // TODO: Build collaboration graph from team interactions
        let collaboration_graph = None;
        
        Ok(TeamCollaborationResponse {
            team_id,
            metrics,
            collaboration_graph,
        })
    }

    // Platform Overview
    pub async fn get_platform_overview(&self) -> Result<PlatformOverviewResponse> {
        let overview = self.repository.get_platform_overview().await?;
        
        Ok(PlatformOverviewResponse {
            overview,
            generated_at: Utc::now(),
        })
    }

    pub async fn get_platform_trends(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<PlatformTrendsResponse> {
        let trends = self
            .repository
            .get_platform_trends(start_date, end_date)
            .await?;
        
        Ok(PlatformTrendsResponse {
            trends,
            period_start: start_date,
            period_end: end_date,
        })
    }
}