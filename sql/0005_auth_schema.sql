-- Auth Schema Migration
-- Creates the auth schema and tables expected by the auth_service

-- Create auth schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Table: auth.nonces
-- Stores authentication nonces for wallet signature verification
CREATE TABLE IF NOT EXISTS auth.nonces (
    address VARCHAR(100) PRIMARY KEY,
    nonce VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '5 minutes'),
    
    -- Constraints
    CONSTRAINT auth_nonces_address_check CHECK (LENGTH(address) = 66),
    CONSTRAINT auth_nonces_nonce_check CHECK (LENGTH(nonce) >= 32),
    CONSTRAINT auth_nonces_expires_check CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_auth_nonces_expires_at ON auth.nonces(expires_at);
CREATE INDEX IF NOT EXISTS idx_auth_nonces_created_at ON auth.nonces(created_at);

-- Table: auth.users  
-- Core user management for auth service with wallet authentication
CREATE TABLE IF NOT EXISTS auth.users (
    address VARCHAR(100) PRIMARY KEY,
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    login_count INTEGER NOT NULL DEFAULT 1,
    
    -- Timestamps (following the existing pattern)
    cid UUID,
    ctime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    mid UUID,
    mtime TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT auth_users_address_check CHECK (LENGTH(address) = 66),
    CONSTRAINT auth_users_public_key_check CHECK (LENGTH(public_key) >= 10),
    CONSTRAINT auth_users_login_count_check CHECK (login_count >= 0)
);

CREATE INDEX IF NOT EXISTS idx_auth_users_created_at ON auth.users(created_at);
CREATE INDEX IF NOT EXISTS idx_auth_users_last_login ON auth.users(last_login);
CREATE INDEX IF NOT EXISTS idx_auth_users_login_count ON auth.users(login_count);

-- Comments for documentation
COMMENT ON SCHEMA auth IS 'Authentication schema for wallet-based user authentication';
COMMENT ON TABLE auth.nonces IS 'Temporary nonces for wallet signature verification';
COMMENT ON TABLE auth.users IS 'Authenticated users with wallet-based login';

COMMENT ON COLUMN auth.nonces.address IS 'Sui wallet address (66 characters: 0x + 64 hex)';
COMMENT ON COLUMN auth.nonces.nonce IS 'Random nonce string for signature verification';
COMMENT ON COLUMN auth.nonces.expires_at IS 'Nonce expiration time (5 minutes from creation)';

COMMENT ON COLUMN auth.users.address IS 'Sui wallet address serving as primary key';
COMMENT ON COLUMN auth.users.public_key IS 'Base64-encoded public key for signature verification';
COMMENT ON COLUMN auth.users.login_count IS 'Number of successful logins';