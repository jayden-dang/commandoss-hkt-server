use modql::SIden;
use sea_query::{Iden, SeaRc, TableRef};

// ================================================================================================
// DMC Trait Definition (Local to avoid circular dependencies)
// ================================================================================================

pub trait DMC {
    const SCHEMA: &'static str;
    const TABLE: &'static str;
    const ID: &'static str;
    const ENUM_COLUMNS: &'static [&'static str];

    fn table_ref() -> TableRef {
        TableRef::SchemaTable(SeaRc::new(SIden(Self::SCHEMA)), SeaRc::new(SIden(Self::TABLE)))
    }

    fn has_timestamps() -> bool {
        true
    }

    fn has_owner_id() -> bool {
        false
    }
}

// ================================================================================================
// Model Controllers (DMC) for ZK-Persona Tables
// ================================================================================================

/// Model controller for behavior_inputs table
pub struct BehaviorInputMC;

impl DMC for BehaviorInputMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "behavior_inputs";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &["input_type", "source"];
}

/// Model controller for scoring_results table
pub struct ScoringResultMC;

impl DMC for ScoringResultMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "scoring_results";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &[];
}

/// Model controller for zk_proofs table
pub struct ZkProofMC;

impl DMC for ZkProofMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "zk_proofs";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &["proof_type", "protocol", "verification_status"];
}

/// Model controller for users table (ZK-Persona specific)
pub struct UserMC;

impl DMC for UserMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "users";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &["status"];
}

/// Model controller for behavior_sessions table
pub struct BehaviorSessionMC;

impl DMC for BehaviorSessionMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "behavior_sessions";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &["session_type", "status"];
}

/// Model controller for reputation_records table
pub struct ReputationRecordMC;

impl DMC for ReputationRecordMC {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "reputation_records";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &["scoring_category", "scoring_period", "status"];
}