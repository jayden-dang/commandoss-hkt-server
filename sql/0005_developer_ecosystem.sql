-- Developer Ecosystem Database Schema
-- Extends ZK-Persona for Smart Contract Guardian system
-- Adds developers, github_repositories, code_analysis_results, security_vulnerabilities, and patch_proposals

-- Ensure UUID extension is available
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create custom enum types for better data consistency
DO $$
BEGIN
    -- Analysis types for code analysis
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'analysis_type_enum') THEN
        CREATE TYPE analysis_type_enum AS ENUM ('static_analysis', 'llm_review', 'dependency_check');
    END IF;
    
    -- Vulnerability types
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'vulnerability_type_enum') THEN
        CREATE TYPE vulnerability_type_enum AS ENUM ('reentrancy', 'overflow', 'access_control', 'other');
    END IF;
    
    -- Severity levels
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'severity_enum') THEN
        CREATE TYPE severity_enum AS ENUM ('critical', 'high', 'medium', 'low');
    END IF;
    
    -- Patch types
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'patch_type_enum') THEN
        CREATE TYPE patch_type_enum AS ENUM ('ai_generated', 'community', 'automated');
    END IF;
    
    -- Patch status
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'patch_status_enum') THEN
        CREATE TYPE patch_status_enum AS ENUM ('proposed', 'under_review', 'approved', 'rejected', 'applied');
    END IF;
END $$;

-- Table: developers
-- Core developer management for GitHub integration
CREATE TABLE IF NOT EXISTS developers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- GitHub identity
    github_username VARCHAR(100) UNIQUE NOT NULL,
    github_user_id BIGINT UNIQUE NOT NULL,
    display_name VARCHAR(255),
    email VARCHAR(255),
    
    -- Reputation scores (0-100 scale)
    coding_reputation_score DECIMAL(5,2) NOT NULL DEFAULT 0.0,
    security_awareness_score DECIMAL(5,2) NOT NULL DEFAULT 0.0,
    community_trust_score DECIMAL(5,2) NOT NULL DEFAULT 0.0,
    
    -- Activity metrics
    total_contributions INTEGER NOT NULL DEFAULT 0,
    account_created_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ,
    
    -- ZK integration
    zk_proof_hash VARCHAR(64), -- SHA-256 hash linking to zk_proofs table
    
    -- Timestamps following existing pattern
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT developers_github_username_check CHECK (
        LENGTH(github_username) >= 1 AND LENGTH(github_username) <= 39 AND
        github_username ~ '^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$'
    ),
    CONSTRAINT developers_github_user_id_check CHECK (github_user_id > 0),
    CONSTRAINT developers_coding_reputation_score_range CHECK (
        coding_reputation_score >= 0.0 AND coding_reputation_score <= 100.0
    ),
    CONSTRAINT developers_security_awareness_score_range CHECK (
        security_awareness_score >= 0.0 AND security_awareness_score <= 100.0
    ),
    CONSTRAINT developers_community_trust_score_range CHECK (
        community_trust_score >= 0.0 AND community_trust_score <= 100.0
    ),
    CONSTRAINT developers_total_contributions_check CHECK (total_contributions >= 0),
    CONSTRAINT developers_email_check CHECK (
        email IS NULL OR email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    ),
    CONSTRAINT developers_zk_proof_hash_check CHECK (
        zk_proof_hash IS NULL OR LENGTH(zk_proof_hash) = 64
    )
);

-- Indexes for developers table
CREATE INDEX IF NOT EXISTS idx_developers_github_username ON developers(github_username);
CREATE INDEX IF NOT EXISTS idx_developers_github_user_id ON developers(github_user_id);
CREATE INDEX IF NOT EXISTS idx_developers_display_name ON developers(display_name);
CREATE INDEX IF NOT EXISTS idx_developers_email ON developers(email);
CREATE INDEX IF NOT EXISTS idx_developers_coding_reputation_score ON developers(coding_reputation_score);
CREATE INDEX IF NOT EXISTS idx_developers_security_awareness_score ON developers(security_awareness_score);
CREATE INDEX IF NOT EXISTS idx_developers_community_trust_score ON developers(community_trust_score);
CREATE INDEX IF NOT EXISTS idx_developers_total_contributions ON developers(total_contributions);
CREATE INDEX IF NOT EXISTS idx_developers_last_activity_at ON developers(last_activity_at);
CREATE INDEX IF NOT EXISTS idx_developers_zk_proof_hash ON developers(zk_proof_hash);
CREATE INDEX IF NOT EXISTS idx_developers_ctime ON developers(ctime);

-- Table: github_repositories
-- GitHub repositories being monitored
CREATE TABLE IF NOT EXISTS github_repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- GitHub identity
    github_repo_id BIGINT UNIQUE NOT NULL,
    owner_username VARCHAR(100) NOT NULL,
    repo_name VARCHAR(100) NOT NULL,
    full_name VARCHAR(201) NOT NULL, -- owner/repo format
    description TEXT,
    
    -- Repository metadata
    primary_language VARCHAR(50),
    is_private BOOLEAN NOT NULL DEFAULT false,
    star_count INTEGER NOT NULL DEFAULT 0,
    fork_count INTEGER NOT NULL DEFAULT 0,
    
    -- Security analysis
    security_score DECIMAL(5,2), -- nullable until first analysis
    last_analyzed_at TIMESTAMPTZ,
    
    -- Monitoring configuration
    webhook_secret VARCHAR(255), -- encrypted webhook secret
    monitoring_enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Timestamps
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT github_repositories_github_repo_id_check CHECK (github_repo_id > 0),
    CONSTRAINT github_repositories_owner_username_check CHECK (
        LENGTH(owner_username) >= 1 AND LENGTH(owner_username) <= 39 AND
        owner_username ~ '^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$'
    ),
    CONSTRAINT github_repositories_repo_name_check CHECK (
        LENGTH(repo_name) >= 1 AND LENGTH(repo_name) <= 100 AND
        repo_name ~ '^[a-zA-Z0-9._-]+$'
    ),
    CONSTRAINT github_repositories_full_name_check CHECK (
        full_name = owner_username || '/' || repo_name
    ),
    CONSTRAINT github_repositories_star_count_check CHECK (star_count >= 0),
    CONSTRAINT github_repositories_fork_count_check CHECK (fork_count >= 0),
    CONSTRAINT github_repositories_security_score_range CHECK (
        security_score IS NULL OR (security_score >= 0.0 AND security_score <= 100.0)
    )
);

-- Indexes for github_repositories table
CREATE INDEX IF NOT EXISTS idx_github_repositories_github_repo_id ON github_repositories(github_repo_id);
CREATE INDEX IF NOT EXISTS idx_github_repositories_owner_username ON github_repositories(owner_username);
CREATE INDEX IF NOT EXISTS idx_github_repositories_repo_name ON github_repositories(repo_name);
CREATE INDEX IF NOT EXISTS idx_github_repositories_full_name ON github_repositories(full_name);
CREATE INDEX IF NOT EXISTS idx_github_repositories_primary_language ON github_repositories(primary_language);
CREATE INDEX IF NOT EXISTS idx_github_repositories_is_private ON github_repositories(is_private);
CREATE INDEX IF NOT EXISTS idx_github_repositories_star_count ON github_repositories(star_count);
CREATE INDEX IF NOT EXISTS idx_github_repositories_security_score ON github_repositories(security_score);
CREATE INDEX IF NOT EXISTS idx_github_repositories_last_analyzed_at ON github_repositories(last_analyzed_at);
CREATE INDEX IF NOT EXISTS idx_github_repositories_monitoring_enabled ON github_repositories(monitoring_enabled);
CREATE INDEX IF NOT EXISTS idx_github_repositories_ctime ON github_repositories(ctime);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_github_repositories_owner_repo ON github_repositories(owner_username, repo_name);
CREATE INDEX IF NOT EXISTS idx_github_repositories_language_stars ON github_repositories(primary_language, star_count DESC);

-- Table: code_analysis_results
-- Results from security and quality analysis
CREATE TABLE IF NOT EXISTS code_analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Relationships
    repository_id UUID NOT NULL REFERENCES github_repositories(id) ON DELETE CASCADE,
    
    -- Git information
    commit_sha VARCHAR(40) NOT NULL,
    
    -- Analysis metadata
    analysis_type analysis_type_enum NOT NULL,
    security_score DECIMAL(5,2) NOT NULL,
    quality_score DECIMAL(5,2) NOT NULL,
    
    -- Issue counts
    issues_found INTEGER NOT NULL DEFAULT 0,
    critical_issues INTEGER NOT NULL DEFAULT 0,
    
    -- Performance metrics
    analysis_duration_ms INTEGER NOT NULL,
    analyzer_version VARCHAR(50) NOT NULL,
    
    -- Raw analysis data
    raw_results JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT code_analysis_results_commit_sha_check CHECK (
        LENGTH(commit_sha) = 40 AND commit_sha ~ '^[a-f0-9]+$'
    ),
    CONSTRAINT code_analysis_results_security_score_range CHECK (
        security_score >= 0.0 AND security_score <= 100.0
    ),
    CONSTRAINT code_analysis_results_quality_score_range CHECK (
        quality_score >= 0.0 AND quality_score <= 100.0
    ),
    CONSTRAINT code_analysis_results_issues_found_check CHECK (issues_found >= 0),
    CONSTRAINT code_analysis_results_critical_issues_check CHECK (
        critical_issues >= 0 AND critical_issues <= issues_found
    ),
    CONSTRAINT code_analysis_results_analysis_duration_check CHECK (analysis_duration_ms > 0),
    CONSTRAINT code_analysis_results_analyzer_version_check CHECK (LENGTH(analyzer_version) >= 1)
);

-- Indexes for code_analysis_results table
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_repository_id ON code_analysis_results(repository_id);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_commit_sha ON code_analysis_results(commit_sha);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_analysis_type ON code_analysis_results(analysis_type);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_security_score ON code_analysis_results(security_score);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_quality_score ON code_analysis_results(quality_score);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_issues_found ON code_analysis_results(issues_found);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_critical_issues ON code_analysis_results(critical_issues);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_ctime ON code_analysis_results(ctime);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_repo_commit ON code_analysis_results(repository_id, commit_sha);
CREATE INDEX IF NOT EXISTS idx_code_analysis_results_repo_type_time ON code_analysis_results(repository_id, analysis_type, ctime DESC);

-- Table: security_vulnerabilities
-- Detected security vulnerabilities
CREATE TABLE IF NOT EXISTS security_vulnerabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Relationships
    repository_id UUID NOT NULL REFERENCES github_repositories(id) ON DELETE CASCADE,
    analysis_result_id UUID NOT NULL REFERENCES code_analysis_results(id) ON DELETE CASCADE,
    
    -- Vulnerability classification
    vulnerability_type vulnerability_type_enum NOT NULL,
    severity severity_enum NOT NULL,
    confidence_score DECIMAL(5,2) NOT NULL,
    
    -- Location information
    file_path TEXT NOT NULL,
    line_number INTEGER,
    code_snippet TEXT,
    
    -- Vulnerability details
    description TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    cve_id VARCHAR(20), -- CVE-YYYY-NNNN format
    
    -- Status tracking
    is_false_positive BOOLEAN NOT NULL DEFAULT false,
    fixed_at TIMESTAMPTZ,
    
    -- Timestamps
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT security_vulnerabilities_confidence_score_range CHECK (
        confidence_score >= 0.0 AND confidence_score <= 100.0
    ),
    CONSTRAINT security_vulnerabilities_line_number_check CHECK (
        line_number IS NULL OR line_number > 0
    ),
    CONSTRAINT security_vulnerabilities_file_path_check CHECK (LENGTH(file_path) >= 1),
    CONSTRAINT security_vulnerabilities_description_check CHECK (LENGTH(description) >= 10),
    CONSTRAINT security_vulnerabilities_recommendation_check CHECK (LENGTH(recommendation) >= 10),
    CONSTRAINT security_vulnerabilities_cve_id_check CHECK (
        cve_id IS NULL OR cve_id ~ '^CVE-[0-9]{4}-[0-9]+$'
    )
);

-- Indexes for security_vulnerabilities table
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_repository_id ON security_vulnerabilities(repository_id);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_analysis_result_id ON security_vulnerabilities(analysis_result_id);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_vulnerability_type ON security_vulnerabilities(vulnerability_type);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_severity ON security_vulnerabilities(severity);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_confidence_score ON security_vulnerabilities(confidence_score);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_file_path ON security_vulnerabilities USING gin(file_path gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_is_false_positive ON security_vulnerabilities(is_false_positive);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_fixed_at ON security_vulnerabilities(fixed_at);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_cve_id ON security_vulnerabilities(cve_id);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_ctime ON security_vulnerabilities(ctime);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_repo_severity ON security_vulnerabilities(repository_id, severity);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_repo_type_severity ON security_vulnerabilities(repository_id, vulnerability_type, severity);
CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_unfixed ON security_vulnerabilities(repository_id, fixed_at) WHERE fixed_at IS NULL;

-- Table: patch_proposals
-- AI-generated and community security patches
CREATE TABLE IF NOT EXISTS patch_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Relationships
    vulnerability_id UUID NOT NULL REFERENCES security_vulnerabilities(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES github_repositories(id) ON DELETE CASCADE,
    proposed_by_developer_id UUID REFERENCES developers(id) ON DELETE SET NULL,
    
    -- Patch metadata
    patch_type patch_type_enum NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    
    -- Patch content
    diff_content TEXT NOT NULL,
    
    -- GitHub integration
    github_pr_number INTEGER,
    
    -- Status and approval
    status patch_status_enum NOT NULL DEFAULT 'proposed',
    community_votes_for INTEGER NOT NULL DEFAULT 0,
    community_votes_against INTEGER NOT NULL DEFAULT 0,
    approval_threshold_met BOOLEAN NOT NULL DEFAULT false,
    
    -- Application tracking
    applied_at TIMESTAMPTZ,
    
    -- Timestamps
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT patch_proposals_title_check CHECK (LENGTH(title) >= 5),
    CONSTRAINT patch_proposals_description_check CHECK (LENGTH(description) >= 20),
    CONSTRAINT patch_proposals_diff_content_check CHECK (LENGTH(diff_content) >= 10),
    CONSTRAINT patch_proposals_github_pr_number_check CHECK (
        github_pr_number IS NULL OR github_pr_number > 0
    ),
    CONSTRAINT patch_proposals_community_votes_for_check CHECK (community_votes_for >= 0),
    CONSTRAINT patch_proposals_community_votes_against_check CHECK (community_votes_against >= 0),
    CONSTRAINT patch_proposals_applied_at_check CHECK (
        (status = 'applied' AND applied_at IS NOT NULL) OR
        (status != 'applied' AND applied_at IS NULL)
    )
);

-- Indexes for patch_proposals table
CREATE INDEX IF NOT EXISTS idx_patch_proposals_vulnerability_id ON patch_proposals(vulnerability_id);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_repository_id ON patch_proposals(repository_id);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_proposed_by_developer_id ON patch_proposals(proposed_by_developer_id);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_patch_type ON patch_proposals(patch_type);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_status ON patch_proposals(status);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_github_pr_number ON patch_proposals(github_pr_number);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_approval_threshold_met ON patch_proposals(approval_threshold_met);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_applied_at ON patch_proposals(applied_at);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_ctime ON patch_proposals(ctime);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_patch_proposals_repo_status ON patch_proposals(repository_id, status);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_vuln_status ON patch_proposals(vulnerability_id, status);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_developer_status ON patch_proposals(proposed_by_developer_id, status);
CREATE INDEX IF NOT EXISTS idx_patch_proposals_votes ON patch_proposals(community_votes_for DESC, community_votes_against ASC);

-- Comments for documentation
COMMENT ON TABLE developers IS 'Developer profiles with GitHub integration and reputation scoring';
COMMENT ON TABLE github_repositories IS 'GitHub repositories being monitored for security analysis';
COMMENT ON TABLE code_analysis_results IS 'Results from automated security and quality analysis';
COMMENT ON TABLE security_vulnerabilities IS 'Detected security vulnerabilities with classification and location';
COMMENT ON TABLE patch_proposals IS 'AI-generated and community security patches with voting system';

-- Column comments for key fields
COMMENT ON COLUMN developers.github_username IS 'GitHub username following GitHub naming conventions';
COMMENT ON COLUMN developers.github_user_id IS 'GitHub numeric user ID for API integration';
COMMENT ON COLUMN developers.coding_reputation_score IS 'Coding skill reputation score (0-100)';
COMMENT ON COLUMN developers.security_awareness_score IS 'Security awareness reputation score (0-100)';
COMMENT ON COLUMN developers.community_trust_score IS 'Community trust reputation score (0-100)';
COMMENT ON COLUMN developers.zk_proof_hash IS 'SHA-256 hash linking to ZK proof for reputation verification';

COMMENT ON COLUMN github_repositories.github_repo_id IS 'GitHub numeric repository ID for API integration';
COMMENT ON COLUMN github_repositories.full_name IS 'Complete repository name in owner/repo format';
COMMENT ON COLUMN github_repositories.webhook_secret IS 'Encrypted secret for GitHub webhook verification';
COMMENT ON COLUMN github_repositories.security_score IS 'Overall security score based on analysis results';

COMMENT ON COLUMN code_analysis_results.commit_sha IS 'Git commit SHA (40-character hex string)';
COMMENT ON COLUMN code_analysis_results.raw_results IS 'Complete analysis results in JSON format';
COMMENT ON COLUMN code_analysis_results.analysis_duration_ms IS 'Analysis execution time in milliseconds';

COMMENT ON COLUMN security_vulnerabilities.confidence_score IS 'Confidence level of vulnerability detection (0-100)';
COMMENT ON COLUMN security_vulnerabilities.file_path IS 'Relative path to file containing vulnerability';
COMMENT ON COLUMN security_vulnerabilities.code_snippet IS 'Vulnerable code snippet for context';
COMMENT ON COLUMN security_vulnerabilities.cve_id IS 'CVE identifier if vulnerability is publicly known';

COMMENT ON COLUMN patch_proposals.diff_content IS 'Git diff content showing proposed changes';
COMMENT ON COLUMN patch_proposals.github_pr_number IS 'GitHub pull request number if patch was submitted';
COMMENT ON COLUMN patch_proposals.approval_threshold_met IS 'Whether patch has met community approval requirements';