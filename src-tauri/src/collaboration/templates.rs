// Template Management

use crate::collaboration::{Template, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template manager
pub struct TemplateManager {
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Create a new template
    pub fn create(&mut self, template: Template) -> Result<(), String> {
        if template.id.is_empty() {
            return Err("Template ID is required".to_string());
        }
        if template.name.is_empty() {
            return Err("Template name is required".to_string());
        }

        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Get template by ID
    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.get(id)
    }

    /// Update template
    pub fn update(&mut self, id: &str, updates: TemplateUpdates) -> Result<(), String> {
        let template = self.templates.get_mut(id)
            .ok_or("Template not found")?;

        if let Some(name) = updates.name {
            template.name = name;
        }
        if let Some(category) = updates.category {
            template.category = category;
        }
        if let Some(content) = updates.content {
            template.content = content;
        }
        if let Some(visibility) = updates.visibility {
            template.visibility = visibility;
        }

        template.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Delete template
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        self.templates.remove(id)
            .map(|_| ())
            .ok_or("Template not found".to_string())
    }

    /// List templates by category
    pub fn list_by_category(&self, category: &str) -> Vec<&Template> {
        self.templates.values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// List public templates
    pub fn list_public(&self) -> Vec<&Template> {
        self.templates.values()
            .filter(|t| t.visibility == "public")
            .collect()
    }

    /// Search templates
    pub fn search(&self, query: &str) -> Vec<&Template> {
        let query_lower = query.to_lowercase();
        self.templates.values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower) ||
                t.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Template update fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUpdates {
    pub name: Option<String>,
    pub category: Option<String>,
    pub content: Option<String>,
    pub visibility: Option<String>,
}

/// Default templates
pub fn get_default_templates() -> Vec<Template> {
    let now = chrono::Utc::now().to_rfc3339();

    vec![
        Template {
            id: "tpl-code-review".to_string(),
            name: "Code Review".to_string(),
            category: "development".to_string(),
            content: "Review the following code for:\n- Bugs and issues\n- Security vulnerabilities\n- Performance concerns\n- Code style".to_string(),
            visibility: "public".to_string(),
            version: "1.0.0".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        Template {
            id: "tpl-meeting-summary".to_string(),
            name: "Meeting Summary".to_string(),
            category: "productivity".to_string(),
            content: "Summarize this meeting:\n- Key decisions\n- Action items\n- Attendees".to_string(),
            visibility: "public".to_string(),
            version: "1.0.0".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
        Template {
            id: "tpl-email-draft".to_string(),
            name: "Email Draft".to_string(),
            category: "communication".to_string(),
            content: "Draft a professional email about:\n{{topic}}\n\nTone: {{tone}}".to_string(),
            visibility: "public".to_string(),
            version: "1.0.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
        },
    ]
}
