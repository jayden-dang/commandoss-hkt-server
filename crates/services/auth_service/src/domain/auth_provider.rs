use serde::{Deserialize, Serialize};
use std::fmt::Display;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_provider", rename_all = "lowercase")]
pub enum AuthProviderType {
    Wallet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "provider_status", rename_all = "lowercase")]
pub enum ProviderStatus {
    Active,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuthProvider {
    pub provider_id: Uuid,
    pub user_id: Uuid,
    pub provider_type: AuthProviderType,
    pub provider_user_id: String,
    pub wallet_address: String,
    pub public_key: String,
    pub provider_metadata: Option<serde_json::Value>,
    pub status: ProviderStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthProviderForCreate {
    pub user_id: Uuid,
    pub provider_type: AuthProviderType,
    pub provider_user_id: String,
    pub wallet_address: String,
    pub public_key: String,
    pub provider_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthProviderForUpdate {
    pub provider_metadata: Option<serde_json::Value>,
    pub status: Option<ProviderStatus>,
    pub last_used_at: Option<OffsetDateTime>,
}

impl AuthProviderType {
    pub fn all() -> Vec<AuthProviderType> {
        vec![AuthProviderType::Wallet]
    }

    pub fn requires_wallet(&self) -> bool {
        matches!(self, AuthProviderType::Wallet)
    }
}

impl Display for AuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let provider_str = match self {
            AuthProviderType::Wallet => "wallet",
        };
        write!(f, "{}", provider_str)
    }
}

impl Display for ProviderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            ProviderStatus::Active => "active",
            ProviderStatus::Suspended => "suspended",
            ProviderStatus::Revoked => "revoked",
        };
        write!(f, "{}", status_str)
    }
}

impl UserAuthProvider {
    pub fn new_wallet_provider(
        user_id: Uuid,
        wallet_address: String,
        public_key: String,
    ) -> UserAuthProviderForCreate {
        UserAuthProviderForCreate {
            user_id,
            provider_type: AuthProviderType::Wallet,
            provider_user_id: wallet_address.clone(),
            wallet_address,
            public_key,
            provider_metadata: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ProviderStatus::Active)
    }

    pub fn validate_wallet_address(address: &str) -> bool {
        address.starts_with("0x") && address.len() == 66 && 
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}