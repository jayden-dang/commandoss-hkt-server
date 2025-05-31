use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSession {
  pub id: Id,
  pub session_id: String,
  pub created_at: OffsetDateTime,
  pub completed: bool,
}
