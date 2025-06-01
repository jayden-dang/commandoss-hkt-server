use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use jd_core::{AppState, base};

use crate::{
    DeveloperDmc,
    domain::{
        CodeReview, Developer, DeveloperActivity, DeveloperCollaborator, DeveloperContribution,
        DeveloperLeaderboard, DeveloperNetwork, DeveloperProfile, DeveloperRepository, DeveloperSearch,
        NetworkNode, Skill, SocialLinks, ZKProofVerification,
        DeveloperForCreate, DeveloperForUpdate, DeveloperFilter,
    },
    Error, Result,
};

pub struct DeveloperRepositoryImpl {
    state: AppState,
}

impl DeveloperRepositoryImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl DeveloperRepository for DeveloperRepositoryImpl {
    async fn create(&self, developer: &Developer) -> Result<Developer> {
        let create_req = DeveloperForCreate {
            username: developer.username.clone(),
            email: developer.email.clone(),
            github_username: developer.github_username.clone(),
            wallet_address: developer.wallet_address.clone(),
            reputation_score: Some(developer.reputation_score),
            is_verified: Some(developer.is_verified),
        };
        
        base::rest::create::<DeveloperDmc, _, Developer>(&self.state.mm(), create_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Handle profile insertion separately (requires separate table/model)
        Ok(developer.clone())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Developer> {
        match base::rest::get_by_id::<DeveloperDmc, Developer>(&self.state.mm(), id).await {
            Ok(mut developer) => {
                // Set default profile for now (TODO: Handle profiles separately)
                developer.profile = DeveloperProfile {
                    display_name: developer.username.clone(),
                    bio: None,
                    avatar_url: None,
                    location: None,
                    website: None,
                    skills: vec![],
                    social_links: SocialLinks {
                        github: developer.github_username.clone(),
                        twitter: None,
                        linkedin: None,
                        personal_site: None,
                    },
                    preferences: crate::domain::DeveloperPreferences {
                        email_notifications: true,
                        public_profile: true,
                        show_activity: true,
                        preferred_languages: vec![],
                    },
                };
                Ok(developer)
            },
            Err(_) => Err(Error::DeveloperNotFound(id.to_string())),
        }
    }

    async fn get_by_username(&self, username: &str) -> Result<Developer> {
        use modql::filter::{OpValsString, FilterValue}; 
        
        let filter = DeveloperFilter {
            username: Some(OpValsString::Single(FilterValue::from(username.to_string()))),
            email: None,
            github_username: None,
            wallet_address: None,
            is_verified: None,
            id: None,
        };

        match base::rest::first::<DeveloperDmc, _, Developer>(&self.state.mm(), Some(filter), None).await {
            Ok(Some(mut developer)) => {
                // Set default profile for now (TODO: Handle profiles separately)
                developer.profile = DeveloperProfile {
                    display_name: developer.username.clone(),
                    bio: None,
                    avatar_url: None,
                    location: None,
                    website: None,
                    skills: vec![],
                    social_links: SocialLinks {
                        github: developer.github_username.clone(),
                        twitter: None,
                        linkedin: None,
                        personal_site: None,
                    },
                    preferences: crate::domain::DeveloperPreferences {
                        email_notifications: true,
                        public_profile: true,
                        show_activity: true,
                        preferred_languages: vec![],
                    },
                };
                Ok(developer)
            },
            Ok(None) => Err(Error::DeveloperNotFound(username.to_string())),
            Err(_) => Err(Error::DeveloperNotFound(username.to_string())),
        }
    }

    async fn get_by_email(&self, email: &str) -> Result<Developer> {
        use modql::filter::{OpValsString, FilterValue}; 
        
        let filter = DeveloperFilter {
            email: Some(OpValsString::Single(FilterValue::from(email.to_string()))),
            username: None,
            github_username: None,
            wallet_address: None,
            is_verified: None,
            id: None,
        };

        match base::rest::first::<DeveloperDmc, _, Developer>(&self.state.mm(), Some(filter), None).await {
            Ok(Some(mut developer)) => {
                // Set default profile for now (TODO: Handle profiles separately)
                developer.profile = DeveloperProfile {
                    display_name: developer.username.clone(),
                    bio: None,
                    avatar_url: None,
                    location: None,
                    website: None,
                    skills: vec![],
                    social_links: SocialLinks {
                        github: developer.github_username.clone(),
                        twitter: None,
                        linkedin: None,
                        personal_site: None,
                    },
                    preferences: crate::domain::DeveloperPreferences {
                        email_notifications: true,
                        public_profile: true,
                        show_activity: true,
                        preferred_languages: vec![],
                    },
                };
                Ok(developer)
            },
            Ok(None) => Err(Error::DeveloperNotFound(email.to_string())),
            Err(_) => Err(Error::DeveloperNotFound(email.to_string())),
        }
    }

    async fn update(&self, developer: &Developer) -> Result<Developer> {
        let update_req = DeveloperForUpdate {
            username: Some(developer.username.clone()),
            email: Some(developer.email.clone()),
            github_username: developer.github_username.clone(),
            wallet_address: developer.wallet_address.clone(),
            reputation_score: Some(developer.reputation_score),
            is_verified: Some(developer.is_verified),
            verification_date: developer.verification_date,
        };
        
        base::rest::update::<DeveloperDmc, _>(&self.state.mm(), developer.id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Handle profile updates separately
        Ok(developer.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        base::rest::delete::<DeveloperDmc>(&self.state.mm(), id)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_profile(&self, id: Uuid, profile: &DeveloperProfile) -> Result<()> {
        // TODO: Implement profile updates with separate ProfileDmc
        Ok(())
    }

    async fn add_skill(&self, developer_id: Uuid, skill: &Skill) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO developer_skills (
                developer_id, skill_name, category, proficiency_level,
                years_experience, verified
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            developer_id,
            skill.name,
            serde_json::to_string(&skill.category)?,
            serde_json::to_string(&skill.proficiency_level)?,
            skill.years_experience,
            skill.verified
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn remove_skill(&self, developer_id: Uuid, skill_name: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM developer_skills
            WHERE developer_id = $1 AND skill_name = $2
            "#,
            developer_id,
            skill_name
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_social_links(&self, id: Uuid, links: &SocialLinks) -> Result<()> {
        // Update social links in profile
        let mut developer = self.get_by_id(id).await?;
        developer.profile.social_links = links.clone();
        self.update(&developer).await?;
        Ok(())
    }

    async fn search(&self, search: &DeveloperSearch) -> Result<Vec<Developer>> {
        let offset = ((search.page - 1) * search.limit) as i64;
        let limit = search.limit as i64;

        // Simple search implementation
        let developers = sqlx::query!(
            r#"
            SELECT id FROM developers
            WHERE ($1::text IS NULL OR username ILIKE '%' || $1 || '%')
            AND ($2::numeric IS NULL OR reputation_score >= $2)
            AND ($3::boolean IS NULL OR is_verified = $3)
            ORDER BY reputation_score DESC
            LIMIT $4 OFFSET $5
            "#,
            search.query,
            search.min_reputation,
            Some(search.verified_only),
            limit,
            offset
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut results = vec![];
        for dev in developers {
            if let Ok(developer) = self.get_by_id(dev.id).await {
                results.push(developer);
            }
        }

        Ok(results)
    }

    async fn count_search(&self, search: &DeveloperSearch) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM developers
            WHERE ($1::text IS NULL OR username ILIKE '%' || $1 || '%')
            AND ($2::numeric IS NULL OR reputation_score >= $2)
            AND ($3::boolean IS NULL OR is_verified = $3)
            "#,
            search.query,
            search.min_reputation,
            Some(search.verified_only)
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    async fn get_leaderboard(&self, limit: i64) -> Result<Vec<DeveloperLeaderboard>> {
        let leaderboard = sqlx::query!(
            r#"
            SELECT 
                id,
                username,
                reputation_score,
                ROW_NUMBER() OVER (ORDER BY reputation_score DESC) as rank
            FROM developers
            ORDER BY reputation_score DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(leaderboard
            .into_iter()
            .map(|r| DeveloperLeaderboard {
                developer_id: r.id,
                username: r.username,
                reputation_score: r.reputation_score,
                rank: r.rank.unwrap_or(0) as i32,
                contributions_count: 0, // TODO: Calculate from activities
                specialization: "Full Stack".to_string(), // TODO: Derive from skills
            })
            .collect())
    }

    async fn get_by_skill(&self, skill: &str, limit: i64) -> Result<Vec<Developer>> {
        let developers = sqlx::query!(
            r#"
            SELECT DISTINCT d.id
            FROM developers d
            JOIN developer_skills ds ON d.id = ds.developer_id
            WHERE ds.skill_name ILIKE '%' || $1 || '%'
            ORDER BY d.reputation_score DESC
            LIMIT $2
            "#,
            skill,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut results = vec![];
        for dev in developers {
            if let Ok(developer) = self.get_by_id(dev.id).await {
                results.push(developer);
            }
        }

        Ok(results)
    }

    async fn add_activity(&self, activity: &DeveloperActivity) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO activities (
                id, developer_id, activity_type, description,
                repository_id, related_id, impact_score, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            activity.id,
            activity.developer_id,
            serde_json::to_string(&activity.activity_type)?,
            activity.description,
            activity.repository_id,
            activity.related_id,
            activity.impact_score,
            activity.created_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_activities(
        &self,
        developer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<DeveloperActivity>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn get_contributions(
        &self,
        developer_id: Uuid,
    ) -> Result<Vec<DeveloperContribution>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn add_contribution(&self, contribution: &DeveloperContribution) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    async fn add_code_review(&self, review: &CodeReview) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    async fn get_reviews_given(
        &self,
        reviewer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CodeReview>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn get_reviews_received(
        &self,
        author_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CodeReview>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn get_collaborators(
        &self,
        developer_id: Uuid,
    ) -> Result<Vec<DeveloperCollaborator>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn add_collaboration(
        &self,
        developer_id: Uuid,
        collaborator_id: Uuid,
        score: Decimal,
    ) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    async fn get_network(&self, developer_id: Uuid) -> Result<DeveloperNetwork> {
        // Placeholder implementation
        Ok(DeveloperNetwork {
            developer_id,
            collaborators: vec![],
            total_connections: 0,
            collaboration_strength: dec!(0),
        })
    }

    async fn get_mentees(&self, mentor_id: Uuid) -> Result<Vec<Developer>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn verify_developer(
        &self,
        developer_id: Uuid,
        verification: &ZKProofVerification,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE developers
            SET is_verified = true, verification_date = $2
            WHERE id = $1
            "#,
            developer_id,
            verification.verified_at
        )
        .execute(&self.db_pool)
        .await?;

        // Store verification details
        sqlx::query!(
            r#"
            INSERT INTO developer_verifications (
                developer_id, proof_data, verification_type,
                verified_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            verification.developer_id,
            verification.proof_data,
            verification.verification_type,
            verification.verified_at,
            verification.expires_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_reputation(&self, developer_id: Uuid, new_score: Decimal) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE developers
            SET reputation_score = $2, updated_at = NOW()
            WHERE id = $1
            "#,
            developer_id,
            new_score
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_developer_count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM developers
            "#
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    async fn get_verified_developer_count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM developers WHERE is_verified = true
            "#
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    async fn get_active_developer_count(&self, days: i32) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT developer_id)
            FROM activities
            WHERE created_at > NOW() - INTERVAL '%d days'
            "#,
            days
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }
}