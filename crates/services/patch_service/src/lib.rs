pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;

pub use error::{Error, Result};

use jd_core::base::DMC;

pub struct PatchDmc;

impl DMC for PatchDmc {
  const SCHEMA: &'static str = "public";
  const TABLE: &'static str = "patch_proposals";
  const ID: &'static str = "id";
  const ENUM_COLUMNS: &'static [&'static str] = &["status"];
}