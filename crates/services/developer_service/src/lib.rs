pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;

pub use error::{Error, Result};

use jd_core::base::DMC;

pub struct DeveloperDmc;

impl DMC for DeveloperDmc {
  const SCHEMA: &'static str = "public";
  const TABLE: &'static str = "developers";
  const ID: &'static str = "id";
  const ENUM_COLUMNS: &'static [&'static str] = &[];
}