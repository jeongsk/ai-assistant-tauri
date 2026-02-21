// Integration Module - External service integrations

pub mod database;
pub mod git;
pub mod cloud;

pub use database::*;
pub use git::*;
pub use cloud::*;

use serde::{Deserialize, Serialize};

/// Integration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    Database,
    Git,
    Cloud,
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub name: String,
    pub integration_type: IntegrationType,
    pub connected: bool,
    pub last_sync: Option<String>,
    pub error: Option<String>,
}

/// Integration configuration base trait
pub trait IntegrationConfig {
    fn validate(&self) -> Result<(), String>;
    fn connection_string(&self) -> String;
}
