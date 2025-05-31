-- Extended ZK-Persona Database Schema
-- Adds users, behavior_sessions, zk_proofs, and reputation_records tables
-- to complement the existing behavior_inputs, scoring_results, and zkml_proofs tables

-- Ensure UUID extension is available
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Table: users
-- Core user management for ZK-Persona system
-- Note: Uses different emails than profile.users to avoid conflicts
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(100) UNIQUE,
    email VARCHAR(255) UNIQUE,
    username VARCHAR(100) UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    privacy_settings JSONB NOT NULL DEFAULT '{"data_sharing": false, "public_profile": false}',
    
    -- Timestamps (following the existing pattern)
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT users_wallet_address_check CHECK (wallet_address IS NULL OR LENGTH(wallet_address) >= 10),
    CONSTRAINT users_email_check CHECK (email IS NULL OR email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT users_username_check CHECK (username IS NULL OR (LENGTH(username) >= 3 AND LENGTH(username) <= 100)),
    CONSTRAINT users_status_check CHECK (status IN ('active', 'inactive', 'suspended', 'deleted'))
);

CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(wallet_address);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_ctime ON users(ctime);

-- Table: behavior_sessions
-- Groups related behavior inputs under sessions for better organization
CREATE TABLE IF NOT EXISTS behavior_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    session_type VARCHAR(50) NOT NULL DEFAULT 'web',
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    duration_seconds INTEGER,
    metadata JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    
    -- Timestamps
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT behavior_sessions_session_token_check CHECK (LENGTH(session_token) >= 10),
    CONSTRAINT behavior_sessions_session_type_check CHECK (session_type IN ('web', 'mobile', 'api', 'blockchain')),
    CONSTRAINT behavior_sessions_duration_check CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
    CONSTRAINT behavior_sessions_status_check CHECK (status IN ('active', 'completed', 'expired', 'terminated')),
    CONSTRAINT behavior_sessions_time_check CHECK (end_time IS NULL OR end_time >= start_time)
);

CREATE INDEX IF NOT EXISTS idx_behavior_sessions_user_id ON behavior_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_behavior_sessions_session_token ON behavior_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_behavior_sessions_session_type ON behavior_sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_behavior_sessions_start_time ON behavior_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_behavior_sessions_status ON behavior_sessions(status);

-- Extend existing behavior_inputs table to reference users and sessions
-- Use IF NOT EXISTS-like approach with proper error handling
DO $$
BEGIN
    -- Add user_id column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'user_id') THEN
        ALTER TABLE behavior_inputs ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;
    END IF;
    
    -- Add behavior_session_id column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'behavior_session_id') THEN
        ALTER TABLE behavior_inputs ADD COLUMN behavior_session_id UUID REFERENCES behavior_sessions(id) ON DELETE SET NULL;
    END IF;
    
    -- Add input_type column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'input_type') THEN
        ALTER TABLE behavior_inputs ADD COLUMN input_type VARCHAR(50) NOT NULL DEFAULT 'general';
    END IF;
    
    -- Add source column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'source') THEN
        ALTER TABLE behavior_inputs ADD COLUMN source VARCHAR(50) NOT NULL DEFAULT 'web';
    END IF;
    
    -- Add timestamp columns if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'cid') THEN
        ALTER TABLE behavior_inputs ADD COLUMN cid UUID;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'ctime') THEN
        ALTER TABLE behavior_inputs ADD COLUMN ctime TIMESTAMPTZ NOT NULL DEFAULT NOW();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'mid') THEN
        ALTER TABLE behavior_inputs ADD COLUMN mid UUID;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'behavior_inputs' AND column_name = 'mtime') THEN
        ALTER TABLE behavior_inputs ADD COLUMN mtime TIMESTAMPTZ NOT NULL DEFAULT NOW();
    END IF;
END $$;

-- Add constraints to behavior_inputs if they don't exist
DO $$
BEGIN
    -- Add input_type constraint if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints 
                   WHERE table_name = 'behavior_inputs' AND constraint_name = 'behavior_inputs_input_type_check') THEN
        ALTER TABLE behavior_inputs 
        ADD CONSTRAINT behavior_inputs_input_type_check CHECK (input_type IN ('transaction', 'interaction', 'defi', 'nft', 'dao', 'social', 'general'));
    END IF;
    
    -- Add source constraint if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints 
                   WHERE table_name = 'behavior_inputs' AND constraint_name = 'behavior_inputs_source_check') THEN
        ALTER TABLE behavior_inputs 
        ADD CONSTRAINT behavior_inputs_source_check CHECK (source IN ('web', 'mobile', 'api', 'blockchain', 'oracle'));
    END IF;
END $$;

-- Add indexes for new columns (conditional creation)
CREATE INDEX IF NOT EXISTS idx_behavior_inputs_user_id ON behavior_inputs(user_id);
CREATE INDEX IF NOT EXISTS idx_behavior_inputs_behavior_session_id ON behavior_inputs(behavior_session_id);
CREATE INDEX IF NOT EXISTS idx_behavior_inputs_input_type ON behavior_inputs(input_type);
CREATE INDEX IF NOT EXISTS idx_behavior_inputs_source ON behavior_inputs(source);

-- Table: zk_proofs (enhanced version of zkml_proofs)
-- Comprehensive ZK proof management with additional fields
CREATE TABLE IF NOT EXISTS zk_proofs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    behavior_input_id UUID REFERENCES behavior_inputs(id) ON DELETE CASCADE,
    scoring_result_id UUID REFERENCES scoring_results(id) ON DELETE CASCADE,
    
    -- Proof data
    proof_data JSONB NOT NULL, -- Store as JSON for flexibility
    verification_key JSONB NOT NULL,
    public_signals JSONB NOT NULL DEFAULT '{}',
    
    -- Proof metadata
    proof_type VARCHAR(50) NOT NULL DEFAULT 'zkml',
    protocol VARCHAR(50) NOT NULL DEFAULT 'groth16',
    circuit_version VARCHAR(50) NOT NULL,
    
    -- Verification status
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    verified_at TIMESTAMPTZ,
    verifier_id UUID,
    
    -- Blockchain integration
    blockchain_network VARCHAR(50),
    transaction_hash VARCHAR(100),
    block_number BIGINT,
    gas_used INTEGER,
    
    -- Additional metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    expires_at TIMESTAMPTZ,
    
    -- Timestamps
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT zk_proofs_proof_type_check CHECK (proof_type IN ('zkml', 'zk-snark', 'zk-stark', 'plonk', 'custom')),
    CONSTRAINT zk_proofs_protocol_check CHECK (protocol IN ('groth16', 'plonk', 'marlin', 'sonic', 'bulletproofs')),
    CONSTRAINT zk_proofs_verification_status_check CHECK (verification_status IN ('pending', 'verified', 'failed', 'expired')),
    CONSTRAINT zk_proofs_circuit_version_check CHECK (LENGTH(circuit_version) >= 1),
    CONSTRAINT zk_proofs_transaction_hash_check CHECK (transaction_hash IS NULL OR LENGTH(transaction_hash) >= 10),
    CONSTRAINT zk_proofs_verified_at_check CHECK (
        (verification_status = 'verified' AND verified_at IS NOT NULL) OR 
        (verification_status != 'verified' AND verified_at IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_zk_proofs_user_id ON zk_proofs(user_id);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_behavior_input_id ON zk_proofs(behavior_input_id);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_scoring_result_id ON zk_proofs(scoring_result_id);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_proof_type ON zk_proofs(proof_type);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_verification_status ON zk_proofs(verification_status);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_transaction_hash ON zk_proofs(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_blockchain_network ON zk_proofs(blockchain_network);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_expires_at ON zk_proofs(expires_at);

-- Table: reputation_records
-- Track reputation scores and changes over time
CREATE TABLE IF NOT EXISTS reputation_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    zk_proof_id UUID REFERENCES zk_proofs(id) ON DELETE SET NULL,
    scoring_result_id UUID REFERENCES scoring_results(id) ON DELETE SET NULL,
    
    -- Reputation data
    reputation_score DECIMAL(10,4) NOT NULL,
    previous_score DECIMAL(10,4),
    score_change DECIMAL(10,4),
    confidence_level DECIMAL(5,4) NOT NULL DEFAULT 0.0,
    
    -- Scoring context
    scoring_category VARCHAR(50) NOT NULL DEFAULT 'overall',
    scoring_period VARCHAR(50) NOT NULL DEFAULT 'current',
    weight DECIMAL(5,4) NOT NULL DEFAULT 1.0,
    
    -- Reputation metadata
    contributing_factors JSONB NOT NULL DEFAULT '[]',
    reputation_badges JSONB NOT NULL DEFAULT '[]',
    peer_validations INTEGER NOT NULL DEFAULT 0,
    
    -- Time-based data
    effective_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    effective_until TIMESTAMPTZ,
    calculation_method VARCHAR(100) NOT NULL DEFAULT 'zkml_score',
    
    -- Status and flags
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_public BOOLEAN NOT NULL DEFAULT false,
    
    -- Timestamps
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT reputation_records_reputation_score_range CHECK (reputation_score >= 0 AND reputation_score <= 100),
    CONSTRAINT reputation_records_previous_score_range CHECK (previous_score IS NULL OR (previous_score >= 0 AND previous_score <= 100)),
    CONSTRAINT reputation_records_confidence_range CHECK (confidence_level >= 0 AND confidence_level <= 1),
    CONSTRAINT reputation_records_weight_range CHECK (weight >= 0 AND weight <= 1),
    CONSTRAINT reputation_records_scoring_category_check CHECK (
        scoring_category IN ('overall', 'defi', 'nft', 'dao', 'social', 'trading', 'staking', 'lending')
    ),
    CONSTRAINT reputation_records_scoring_period_check CHECK (
        scoring_period IN ('current', 'daily', 'weekly', 'monthly', 'quarterly', 'yearly', 'all_time')
    ),
    CONSTRAINT reputation_records_status_check CHECK (status IN ('active', 'archived', 'disputed', 'invalidated')),
    CONSTRAINT reputation_records_effective_dates_check CHECK (effective_until IS NULL OR effective_until > effective_from),
    CONSTRAINT reputation_records_peer_validations_check CHECK (peer_validations >= 0)
);

CREATE INDEX IF NOT EXISTS idx_reputation_records_user_id ON reputation_records(user_id);
CREATE INDEX IF NOT EXISTS idx_reputation_records_zk_proof_id ON reputation_records(zk_proof_id);
CREATE INDEX IF NOT EXISTS idx_reputation_records_scoring_result_id ON reputation_records(scoring_result_id);
CREATE INDEX IF NOT EXISTS idx_reputation_records_reputation_score ON reputation_records(reputation_score);
CREATE INDEX IF NOT EXISTS idx_reputation_records_scoring_category ON reputation_records(scoring_category);
CREATE INDEX IF NOT EXISTS idx_reputation_records_scoring_period ON reputation_records(scoring_period);
CREATE INDEX IF NOT EXISTS idx_reputation_records_effective_from ON reputation_records(effective_from);
CREATE INDEX IF NOT EXISTS idx_reputation_records_status ON reputation_records(status);
CREATE INDEX IF NOT EXISTS idx_reputation_records_is_verified ON reputation_records(is_verified);
CREATE INDEX IF NOT EXISTS idx_reputation_records_is_public ON reputation_records(is_public);

-- Create composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_reputation_records_user_category_period ON reputation_records(user_id, scoring_category, scoring_period);
CREATE INDEX IF NOT EXISTS idx_reputation_records_user_effective ON reputation_records(user_id, effective_from DESC);
CREATE INDEX IF NOT EXISTS idx_behavior_inputs_user_session ON behavior_inputs(user_id, behavior_session_id);
CREATE INDEX IF NOT EXISTS idx_zk_proofs_user_status ON zk_proofs(user_id, verification_status);

-- Comments for documentation
COMMENT ON TABLE users IS 'Core user management for ZK-Persona system with privacy settings';
COMMENT ON TABLE behavior_sessions IS 'Groups related behavior inputs under sessions for better organization';
COMMENT ON TABLE zk_proofs IS 'Enhanced ZK proof management with comprehensive metadata and blockchain integration';
COMMENT ON TABLE reputation_records IS 'Time-series reputation tracking with detailed scoring context and verification';

-- Column comments
COMMENT ON COLUMN users.wallet_address IS 'Blockchain wallet address for decentralized identity';
COMMENT ON COLUMN users.privacy_settings IS 'User privacy preferences for data sharing and profile visibility';
COMMENT ON COLUMN behavior_sessions.session_token IS 'Unique session identifier for tracking user behavior';
COMMENT ON COLUMN behavior_sessions.duration_seconds IS 'Session duration calculated from start_time to end_time';
COMMENT ON COLUMN zk_proofs.public_signals IS 'Public inputs to the ZK proof for verification';
COMMENT ON COLUMN zk_proofs.circuit_version IS 'Version of the ZK circuit used to generate the proof';
COMMENT ON COLUMN reputation_records.contributing_factors IS 'Array of factors that contributed to the reputation score';
COMMENT ON COLUMN reputation_records.reputation_badges IS 'Earned badges and achievements based on reputation';
COMMENT ON COLUMN reputation_records.peer_validations IS 'Number of peer validations received for this reputation record';

-- Insert sample users for ZK-Persona (using unique emails to avoid conflicts)
INSERT INTO users (wallet_address, email, username, status, privacy_settings) VALUES
('0x1234567890abcdef1234567890abcdef12345678', 'zkuser1@zkpersona.dev', 'zkuser1', 'active', '{"data_sharing": true, "public_profile": true}'),
('0x2345678901bcdef12345678901bcdef123456789', 'zkuser2@zkpersona.dev', 'zkuser2', 'active', '{"data_sharing": false, "public_profile": true}'),
('0x3456789012cdef123456789012cdef12345678a', 'zkuser3@zkpersona.dev', 'zkuser3', 'active', '{"data_sharing": true, "public_profile": false}')
ON CONFLICT (email) DO NOTHING;