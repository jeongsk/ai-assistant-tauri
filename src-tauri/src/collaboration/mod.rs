// Collaboration Module

pub mod templates;
pub mod export_mod;

use serde::{Deserialize, Serialize};

/// Template visibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Team,
}

/// Template model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub category: String,
    pub content: String,
    pub visibility: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Shared workflow model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorkflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: String,
    pub owner_id: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Markdown,
    Html,
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_timestamps: bool,
    pub pretty_print: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_timestamps: true,
            pretty_print: true,
        }
    }
}
