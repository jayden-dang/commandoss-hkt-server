use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Postgres};
use rust_decimal::Decimal;

use crate::{
    dbx::Dbx,
    repository::{
        FilterableRepository, Repository
    },
};
use jd_domain::{
    Id,
    zkpersona_domain::developer_models::{
        Developer, DeveloperForCreate, DeveloperForUpdate, DeveloperFilter,
        GitHubRepository, GitHubRepositoryForCreate, GitHubRepositoryFilter,
        SecurityVulnerability, SecurityVulnerabilityForCreate, SecurityVulnerabilityForUpdate,
    }
};

// ================================================================================================
// Custom Error Type
// ================================================================================================

#[derive(Debug, thiserror::Error)]
pub enum DeveloperRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Dbx error: {0}")]
    Dbx(#[from] crate::dbx::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

type DeveloperResult<T> = Result<T, DeveloperRepositoryError>;

// ================================================================================================
// Developer Repository
// ================================================================================================

#[derive(Debug, Clone)]
pub struct DeveloperRepository {
    pub dbx: Dbx,
}

impl DeveloperRepository {
    pub fn new(dbx: Dbx) -> Self {
        Self { dbx }
    }

    pub async fn find_by_github_username(&self, github_username: &str) -> DeveloperResult<Option<Developer>> {
        let query = "SELECT * FROM developers WHERE github_username = $1";
        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(github_username);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    pub async fn find_by_github_user_id(&self, github_user_id: i64) -> DeveloperResult<Option<Developer>> {
        let query = "SELECT * FROM developers WHERE github_user_id = $1";
        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(github_user_id);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    pub async fn create(&self, create_req: DeveloperForCreate) -> DeveloperResult<Developer> {
        let now = Utc::now();
        let id = Id::generate();
        
        let query = r#"
            INSERT INTO developers (
                id, github_username, github_user_id, display_name, email,
                coding_reputation_score, security_awareness_score, community_trust_score,
                total_contributions, account_created_at, zk_proof_hash, ctime, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(id)
            .bind(&create_req.github_username)
            .bind(create_req.github_user_id)
            .bind(&create_req.display_name)
            .bind(&create_req.email)
            .bind(create_req.coding_reputation_score.unwrap_or_else(|| Decimal::new(0, 0)))
            .bind(create_req.security_awareness_score.unwrap_or_else(|| Decimal::new(0, 0)))
            .bind(create_req.community_trust_score.unwrap_or_else(|| Decimal::new(0, 0)))
            .bind(create_req.total_contributions.unwrap_or(0))
            .bind(create_req.account_created_at)
            .bind(&create_req.zk_proof_hash)
            .bind(now)
            .bind(now);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    pub async fn update_scores(&self, id: Id, update_req: DeveloperForUpdate) -> DeveloperResult<Developer> {
        let now = Utc::now();
        
        let query = r#"
            UPDATE developers SET
                display_name = COALESCE($2, display_name),
                email = COALESCE($3, email),
                coding_reputation_score = COALESCE($4, coding_reputation_score),
                security_awareness_score = COALESCE($5, security_awareness_score),
                community_trust_score = COALESCE($6, community_trust_score),
                total_contributions = COALESCE($7, total_contributions),
                last_activity_at = COALESCE($8, last_activity_at),
                zk_proof_hash = COALESCE($9, zk_proof_hash),
                mtime = $10
            WHERE id = $1
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(id)
            .bind(&update_req.display_name)
            .bind(&update_req.email)
            .bind(update_req.coding_reputation_score)
            .bind(update_req.security_awareness_score)
            .bind(update_req.community_trust_score)
            .bind(update_req.total_contributions)
            .bind(update_req.last_activity_at)
            .bind(&update_req.zk_proof_hash)
            .bind(now);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }
}

#[async_trait]
impl Repository<Developer, Id> for DeveloperRepository {
    type Error = DeveloperRepositoryError;

    async fn find_by_id(&self, id: Id) -> DeveloperResult<Option<Developer>> {
        let query = "SELECT * FROM developers WHERE id = $1";
        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(id);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    async fn find_all(&self) -> DeveloperResult<Vec<Developer>> {
        let query = "SELECT * FROM developers ORDER BY ctime DESC";
        let query_as = sqlx::query_as::<_, Developer>(query);
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    async fn save(&self, entity: &Developer) -> DeveloperResult<Developer> {
        let query = r#"
            INSERT INTO developers (
                id, github_username, github_user_id, display_name, email,
                coding_reputation_score, security_awareness_score, community_trust_score,
                total_contributions, account_created_at, last_activity_at, zk_proof_hash,
                cid, ctime, mid, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(&entity.id)
            .bind(&entity.github_username)
            .bind(entity.github_user_id)
            .bind(&entity.display_name)
            .bind(&entity.email)
            .bind(entity.coding_reputation_score)
            .bind(entity.security_awareness_score)
            .bind(entity.community_trust_score)
            .bind(entity.total_contributions)
            .bind(entity.account_created_at)
            .bind(entity.last_activity_at)
            .bind(&entity.zk_proof_hash)
            .bind(&entity.cid)
            .bind(entity.ctime)
            .bind(&entity.mid)
            .bind(entity.mtime);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn update(&self, id: Id, entity: &Developer) -> DeveloperResult<Developer> {
        let query = r#"
            UPDATE developers SET
                github_username = $2,
                github_user_id = $3,
                display_name = $4,
                email = $5,
                coding_reputation_score = $6,
                security_awareness_score = $7,
                community_trust_score = $8,
                total_contributions = $9,
                account_created_at = $10,
                last_activity_at = $11,
                zk_proof_hash = $12,
                mid = $13,
                mtime = $14
            WHERE id = $1
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, Developer>(query)
            .bind(id)
            .bind(&entity.github_username)
            .bind(entity.github_user_id)
            .bind(&entity.display_name)
            .bind(&entity.email)
            .bind(entity.coding_reputation_score)
            .bind(entity.security_awareness_score)
            .bind(entity.community_trust_score)
            .bind(entity.total_contributions)
            .bind(entity.account_created_at)
            .bind(entity.last_activity_at)
            .bind(&entity.zk_proof_hash)
            .bind(&entity.mid)
            .bind(entity.mtime);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn delete(&self, id: Id) -> DeveloperResult<bool> {
        let query = "DELETE FROM developers WHERE id = $1";
        let query_cmd = sqlx::query(query)
            .bind(id);
        let rows_affected = self.dbx.execute(query_cmd).await?;
        Ok(rows_affected > 0)
    }

    async fn count(&self) -> DeveloperResult<i64> {
        let query = "SELECT COUNT(*) FROM developers";
        let query_as = sqlx::query_as::<_, (i64,)>(query);
        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result.0)
    }
}

#[async_trait]
impl FilterableRepository<Developer, Id, DeveloperFilter> for DeveloperRepository {
    async fn find_by_filter(&self, filter: DeveloperFilter) -> DeveloperResult<Vec<Developer>> {
        let mut query_builder = QueryBuilder::<Postgres>::new("SELECT * FROM developers WHERE 1=1");
        
        if let Some(github_username) = &filter.github_username {
            query_builder.push(" AND github_username ILIKE ");
            query_builder.push_bind(format!("%{}%", github_username));
        }
        
        if let Some(email) = &filter.email {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        
        if let Some(min_score) = filter.min_coding_reputation_score {
            query_builder.push(" AND coding_reputation_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_score) = filter.min_security_awareness_score {
            query_builder.push(" AND security_awareness_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_score) = filter.min_community_trust_score {
            query_builder.push(" AND community_trust_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_contributions) = filter.min_total_contributions {
            query_builder.push(" AND total_contributions >= ");
            query_builder.push_bind(min_contributions);
        }
        
        if let Some(has_zk_proof) = filter.has_zk_proof {
            if has_zk_proof {
                query_builder.push(" AND zk_proof_hash IS NOT NULL");
            } else {
                query_builder.push(" AND zk_proof_hash IS NULL");
            }
        }
        
        query_builder.push(" ORDER BY ctime DESC");
        
        let query_as = query_builder.build_query_as::<Developer>();
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    async fn count_by_filter(&self, filter: DeveloperFilter) -> DeveloperResult<i64> {
        let mut query_builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM developers WHERE 1=1");
        
        if let Some(github_username) = &filter.github_username {
            query_builder.push(" AND github_username ILIKE ");
            query_builder.push_bind(format!("%{}%", github_username));
        }
        
        if let Some(email) = &filter.email {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        
        if let Some(min_score) = filter.min_coding_reputation_score {
            query_builder.push(" AND coding_reputation_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_score) = filter.min_security_awareness_score {
            query_builder.push(" AND security_awareness_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_score) = filter.min_community_trust_score {
            query_builder.push(" AND community_trust_score >= ");
            query_builder.push_bind(min_score);
        }
        
        if let Some(min_contributions) = filter.min_total_contributions {
            query_builder.push(" AND total_contributions >= ");
            query_builder.push_bind(min_contributions);
        }
        
        if let Some(has_zk_proof) = filter.has_zk_proof {
            if has_zk_proof {
                query_builder.push(" AND zk_proof_hash IS NOT NULL");
            } else {
                query_builder.push(" AND zk_proof_hash IS NULL");
            }
        }
        
        let query_as = query_builder.build_query_as::<(i64,)>();
        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result.0)
    }

    async fn delete_by_filter(&self, filter: DeveloperFilter) -> DeveloperResult<u64> {
        let mut query_builder = QueryBuilder::<Postgres>::new("DELETE FROM developers WHERE 1=1");
        
        if let Some(github_username) = &filter.github_username {
            query_builder.push(" AND github_username ILIKE ");
            query_builder.push_bind(format!("%{}%", github_username));
        }
        
        if let Some(email) = &filter.email {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        
        let query_cmd = query_builder.build();
        let rows_affected = self.dbx.execute(query_cmd).await?;
        Ok(rows_affected)
    }
}

// ================================================================================================
// GitHub Repository Repository
// ================================================================================================

#[derive(Debug, Clone)]
pub struct GitHubRepositoryRepository {
    pub dbx: Dbx,
}

impl GitHubRepositoryRepository {
    pub fn new(dbx: Dbx) -> Self {
        Self { dbx }
    }

    pub async fn find_by_full_name(&self, full_name: &str) -> DeveloperResult<Option<GitHubRepository>> {
        let query = "SELECT * FROM github_repositories WHERE full_name = $1";
        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(full_name);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    pub async fn find_by_github_repo_id(&self, github_repo_id: i64) -> DeveloperResult<Option<GitHubRepository>> {
        let query = "SELECT * FROM github_repositories WHERE github_repo_id = $1";
        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(github_repo_id);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    pub async fn find_monitored(&self) -> DeveloperResult<Vec<GitHubRepository>> {
        let query = "SELECT * FROM github_repositories WHERE monitoring_enabled = true ORDER BY ctime DESC";
        let query_as = sqlx::query_as::<_, GitHubRepository>(query);
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    pub async fn create(&self, create_req: GitHubRepositoryForCreate) -> DeveloperResult<GitHubRepository> {
        let now = Utc::now();
        let id = Id::generate();
        let full_name = format!("{}/{}", create_req.owner_username, create_req.repo_name);
        
        let query = r#"
            INSERT INTO github_repositories (
                id, github_repo_id, owner_username, repo_name, full_name, description,
                primary_language, is_private, star_count, fork_count,
                webhook_secret, monitoring_enabled, ctime, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(id)
            .bind(create_req.github_repo_id)
            .bind(&create_req.owner_username)
            .bind(&create_req.repo_name)
            .bind(&full_name)
            .bind(&create_req.description)
            .bind(&create_req.primary_language)
            .bind(create_req.is_private)
            .bind(create_req.star_count.unwrap_or(0))
            .bind(create_req.fork_count.unwrap_or(0))
            .bind(&create_req.webhook_secret)
            .bind(create_req.monitoring_enabled.unwrap_or(true))
            .bind(now)
            .bind(now);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }
}

#[async_trait]
impl Repository<GitHubRepository, Id> for GitHubRepositoryRepository {
    type Error = DeveloperRepositoryError;

    async fn find_by_id(&self, id: Id) -> DeveloperResult<Option<GitHubRepository>> {
        let query = "SELECT * FROM github_repositories WHERE id = $1";
        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(id);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    async fn find_all(&self) -> DeveloperResult<Vec<GitHubRepository>> {
        let query = "SELECT * FROM github_repositories ORDER BY ctime DESC";
        let query_as = sqlx::query_as::<_, GitHubRepository>(query);
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    async fn save(&self, entity: &GitHubRepository) -> DeveloperResult<GitHubRepository> {
        let query = r#"
            INSERT INTO github_repositories (
                id, github_repo_id, owner_username, repo_name, full_name, description,
                primary_language, is_private, star_count, fork_count,
                security_score, last_analyzed_at, webhook_secret, monitoring_enabled,
                cid, ctime, mid, mtime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(&entity.id)
            .bind(entity.github_repo_id)
            .bind(&entity.owner_username)
            .bind(&entity.repo_name)
            .bind(&entity.full_name)
            .bind(&entity.description)
            .bind(&entity.primary_language)
            .bind(entity.is_private)
            .bind(entity.star_count)
            .bind(entity.fork_count)
            .bind(entity.security_score)
            .bind(entity.last_analyzed_at)
            .bind(&entity.webhook_secret)
            .bind(entity.monitoring_enabled)
            .bind(&entity.cid)
            .bind(entity.ctime)
            .bind(&entity.mid)
            .bind(entity.mtime);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn update(&self, id: Id, entity: &GitHubRepository) -> DeveloperResult<GitHubRepository> {
        let query = r#"
            UPDATE github_repositories SET
                github_repo_id = $2,
                owner_username = $3,
                repo_name = $4,
                full_name = $5,
                description = $6,
                primary_language = $7,
                is_private = $8,
                star_count = $9,
                fork_count = $10,
                security_score = $11,
                last_analyzed_at = $12,
                webhook_secret = $13,
                monitoring_enabled = $14,
                mid = $15,
                mtime = $16
            WHERE id = $1
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, GitHubRepository>(query)
            .bind(id)
            .bind(entity.github_repo_id)
            .bind(&entity.owner_username)
            .bind(&entity.repo_name)
            .bind(&entity.full_name)
            .bind(&entity.description)
            .bind(&entity.primary_language)
            .bind(entity.is_private)
            .bind(entity.star_count)
            .bind(entity.fork_count)
            .bind(entity.security_score)
            .bind(entity.last_analyzed_at)
            .bind(&entity.webhook_secret)
            .bind(entity.monitoring_enabled)
            .bind(&entity.mid)
            .bind(entity.mtime);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn delete(&self, id: Id) -> DeveloperResult<bool> {
        let query = "DELETE FROM github_repositories WHERE id = $1";
        let query_cmd = sqlx::query(query)
            .bind(id);
        let rows_affected = self.dbx.execute(query_cmd).await?;
        Ok(rows_affected > 0)
    }

    async fn count(&self) -> DeveloperResult<i64> {
        let query = "SELECT COUNT(*) FROM github_repositories";
        let query_as = sqlx::query_as::<_, (i64,)>(query);
        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result.0)
    }
}

// ================================================================================================
// Security Vulnerability Repository
// ================================================================================================

#[derive(Debug, Clone)]
pub struct SecurityVulnerabilityRepository {
    pub dbx: Dbx,
}

impl SecurityVulnerabilityRepository {
    pub fn new(dbx: Dbx) -> Self {
        Self { dbx }
    }

    pub async fn find_by_repository(&self, repository_id: Id) -> DeveloperResult<Vec<SecurityVulnerability>> {
        let query = "SELECT * FROM security_vulnerabilities WHERE repository_id = $1 ORDER BY ctime DESC";
        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(repository_id);
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    pub async fn find_unfixed(&self, repository_id: Option<Id>) -> DeveloperResult<Vec<SecurityVulnerability>> {
        let (query_str, query_as) = if let Some(repo_id) = repository_id {
            let query = "SELECT * FROM security_vulnerabilities WHERE repository_id = $1 AND fixed_at IS NULL AND is_false_positive = false ORDER BY severity::text, ctime DESC";
            (query, sqlx::query_as::<_, SecurityVulnerability>(query).bind(repo_id))
        } else {
            let query = "SELECT * FROM security_vulnerabilities WHERE fixed_at IS NULL AND is_false_positive = false ORDER BY severity::text, ctime DESC";
            (query, sqlx::query_as::<_, SecurityVulnerability>(query))
        };
        
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    pub async fn create(&self, create_req: SecurityVulnerabilityForCreate) -> DeveloperResult<SecurityVulnerability> {
        let now = Utc::now();
        let id = Id::generate();
        
        let query = r#"
            INSERT INTO security_vulnerabilities (
                id, repository_id, analysis_result_id, vulnerability_type, severity,
                confidence_score, file_path, line_number, code_snippet,
                description, recommendation, cve_id, is_false_positive, ctime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(id)
            .bind(create_req.repository_id)
            .bind(create_req.analysis_result_id)
            .bind(create_req.vulnerability_type)
            .bind(create_req.severity)
            .bind(create_req.confidence_score)
            .bind(&create_req.file_path)
            .bind(create_req.line_number)
            .bind(&create_req.code_snippet)
            .bind(&create_req.description)
            .bind(&create_req.recommendation)
            .bind(&create_req.cve_id)
            .bind(create_req.is_false_positive.unwrap_or(false))
            .bind(now);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    pub async fn mark_as_fixed(&self, id: Id) -> DeveloperResult<SecurityVulnerability> {
        let now = Utc::now();
        let query = "UPDATE security_vulnerabilities SET fixed_at = $2 WHERE id = $1 RETURNING *";
        
        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(id)
            .bind(now);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    pub async fn mark_as_false_positive(&self, id: Id, is_false_positive: bool) -> DeveloperResult<SecurityVulnerability> {
        let query = "UPDATE security_vulnerabilities SET is_false_positive = $2 WHERE id = $1 RETURNING *";
        
        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(id)
            .bind(is_false_positive);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }
}

#[async_trait]
impl Repository<SecurityVulnerability, Id> for SecurityVulnerabilityRepository {
    type Error = DeveloperRepositoryError;

    async fn find_by_id(&self, id: Id) -> DeveloperResult<Option<SecurityVulnerability>> {
        let query = "SELECT * FROM security_vulnerabilities WHERE id = $1";
        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(id);
        let result = self.dbx.fetch_optional(query_as).await?;
        Ok(result)
    }

    async fn find_all(&self) -> DeveloperResult<Vec<SecurityVulnerability>> {
        let query = "SELECT * FROM security_vulnerabilities ORDER BY ctime DESC";
        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query);
        let result = self.dbx.fetch_all(query_as).await?;
        Ok(result)
    }

    async fn save(&self, entity: &SecurityVulnerability) -> DeveloperResult<SecurityVulnerability> {
        let query = r#"
            INSERT INTO security_vulnerabilities (
                id, repository_id, analysis_result_id, vulnerability_type, severity,
                confidence_score, file_path, line_number, code_snippet,
                description, recommendation, cve_id, is_false_positive, fixed_at, ctime
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(&entity.id)
            .bind(&entity.repository_id)
            .bind(&entity.analysis_result_id)
            .bind(&entity.vulnerability_type)
            .bind(&entity.severity)
            .bind(entity.confidence_score)
            .bind(&entity.file_path)
            .bind(entity.line_number)
            .bind(&entity.code_snippet)
            .bind(&entity.description)
            .bind(&entity.recommendation)
            .bind(&entity.cve_id)
            .bind(entity.is_false_positive)
            .bind(entity.fixed_at)
            .bind(entity.ctime);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn update(&self, id: Id, entity: &SecurityVulnerability) -> DeveloperResult<SecurityVulnerability> {
        let query = r#"
            UPDATE security_vulnerabilities SET
                repository_id = $2,
                analysis_result_id = $3,
                vulnerability_type = $4,
                severity = $5,
                confidence_score = $6,
                file_path = $7,
                line_number = $8,
                code_snippet = $9,
                description = $10,
                recommendation = $11,
                cve_id = $12,
                is_false_positive = $13,
                fixed_at = $14
            WHERE id = $1
            RETURNING *
        "#;

        let query_as = sqlx::query_as::<_, SecurityVulnerability>(query)
            .bind(id)
            .bind(&entity.repository_id)
            .bind(&entity.analysis_result_id)
            .bind(&entity.vulnerability_type)
            .bind(&entity.severity)
            .bind(entity.confidence_score)
            .bind(&entity.file_path)
            .bind(entity.line_number)
            .bind(&entity.code_snippet)
            .bind(&entity.description)
            .bind(&entity.recommendation)
            .bind(&entity.cve_id)
            .bind(entity.is_false_positive)
            .bind(entity.fixed_at);

        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result)
    }

    async fn delete(&self, id: Id) -> DeveloperResult<bool> {
        let query = "DELETE FROM security_vulnerabilities WHERE id = $1";
        let query_cmd = sqlx::query(query)
            .bind(id);
        let rows_affected = self.dbx.execute(query_cmd).await?;
        Ok(rows_affected > 0)
    }

    async fn count(&self) -> DeveloperResult<i64> {
        let query = "SELECT COUNT(*) FROM security_vulnerabilities";
        let query_as = sqlx::query_as::<_, (i64,)>(query);
        let result = self.dbx.fetch_one(query_as).await?;
        Ok(result.0)
    }
}