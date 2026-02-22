//! Template JSON Export/Import functionality
//!
//! This module provides serialization and deserialization for templates.

use crate::collaboration::Template;
use serde::{Deserialize, Serialize};

/// Export data containing multiple templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateExportData {
    pub version: String,
    pub exported_at: String,
    pub templates: Vec<Template>,
}

/// Import result statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip importing templates with conflicting IDs
    Skip,
    /// Overwrite existing templates
    Overwrite,
    /// Rename imported template (append suffix)
    Rename,
    /// Create a new version
    Version,
}

/// Export templates to JSON format
pub fn export_templates_to_json(templates: &[Template]) -> Result<Vec<u8>, String> {
    let export_data = TemplateExportData {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        templates: templates.to_vec(),
    };

    serde_json::to_vec_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize templates: {}", e))
}

/// Import templates from JSON format
pub fn import_templates_from_json(
    data: &[u8],
    resolution: ConflictResolution,
) -> Result<(Vec<Template>, ImportResult), String> {
    // Parse JSON
    let export_data: TemplateExportData = serde_json::from_slice(data)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut imported = Vec::new();
    let skipped = 0;
    let mut errors = Vec::new();

    for template in export_data.templates {
        // Validate template
        if let Err(e) = validate_template(&template) {
            errors.push(e);
            continue;
        }

        // Handle conflicts based on resolution strategy
        match resolution {
            ConflictResolution::Skip => {
                // Would check against existing templates here
                imported.push(template);
            }
            ConflictResolution::Overwrite => {
                imported.push(template);
            }
            ConflictResolution::Rename => {
                let mut renamed = template;
                renamed.id = format!("{}-imported", renamed.id);
                imported.push(renamed);
            }
            ConflictResolution::Version => {
                let mut versioned = template;
                versioned.id = format!("{}@v2", versioned.id);
                imported.push(versioned);
            }
        }
    }

    let result = ImportResult {
        success: errors.is_empty(),
        imported_count: imported.len(),
        skipped_count: skipped,
        error_count: errors.len(),
        errors,
    };

    Ok((imported, result))
}

/// Validate a template structure
pub fn validate_template(template: &Template) -> Result<(), String> {
    if template.id.is_empty() {
        return Err("Template ID cannot be empty".to_string());
    }
    if template.name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }
    if template.content.is_empty() {
        return Err("Template content cannot be empty".to_string());
    }

    // Validate visibility
    match template.visibility.as_str() {
        "private" | "public" | "team" => {}
        _ => return Err(format!("Invalid visibility: {}", template.visibility)),
    }

    // Validate version format (simple check for now)
    if !template.version.chars().all(|c| c.is_numeric() || c == '.') {
        return Err(format!("Invalid version format: {}", template.version));
    }

    Ok(())
}

/// Export a single template to JSON
pub fn export_template_to_json(template: &Template) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(template)
        .map_err(|e| format!("Failed to serialize template: {}", e))
}

/// Import a single template from JSON
pub fn import_template_from_json(data: &[u8]) -> Result<Template, String> {
    let template: Template = serde_json::from_slice(data)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    validate_template(&template)?;
    Ok(template)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let template = Template {
            id: "test".to_string(),
            name: "Test Template".to_string(),
            category: "test".to_string(),
            content: "Test content".to_string(),
            visibility: "private".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let json = export_template_to_json(&template).unwrap();
        let imported = import_template_from_json(&json).unwrap();

        assert_eq!(imported.id, template.id);
        assert_eq!(imported.name, template.name);
    }

    #[test]
    fn test_validate_template_valid() {
        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: "test".to_string(),
            content: "Content".to_string(),
            visibility: "private".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        assert!(validate_template(&template).is_ok());
    }

    #[test]
    fn test_validate_template_invalid_visibility() {
        let mut template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: "test".to_string(),
            content: "Content".to_string(),
            visibility: "invalid".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        assert!(validate_template(&template).is_err());
    }
}
