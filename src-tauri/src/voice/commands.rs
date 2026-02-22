//! Voice Command Parser and Router
//!
//! This module provides voice command parsing and natural language routing.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parsed voice command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    /// Original transcript
    pub transcript: String,
    /// Detected language
    pub language: String,
    /// Parsed action
    pub action: VoiceAction,
    /// Extracted parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Confidence score
    pub confidence: f32,
}

/// Voice action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceAction {
    /// Execute a skill
    ExecuteSkill { skill_name: String },
    /// Run a recipe
    RunRecipe { recipe_name: String },
    /// Send a message
    SendMessage { content: String },
    /// Open a feature
    OpenFeature { feature: String },
    /// Search query
    Search { query: String },
    /// Unknown command
    Unknown,
}

/// Voice command parser
pub struct VoiceCommandParser {
    /// Command patterns
    patterns: Vec<CommandPattern>,
    /// Language-specific patterns
    language_patterns: HashMap<String, Vec<CommandPattern>>,
}

/// Command pattern for matching
#[derive(Debug, Clone)]
struct CommandPattern {
    /// Regex pattern (simplified)
    pattern: String,
    /// Action to execute
    action: VoiceAction,
    /// Parameter extractors
    param_names: Vec<String>,
}

impl VoiceCommandParser {
    /// Create a new voice command parser
    pub fn new() -> Self {
        // English patterns
        let patterns = vec![
            CommandPattern {
                pattern: "execute (.+) skill".to_string(),
                action: VoiceAction::ExecuteSkill { skill_name: "".to_string() },
                param_names: vec!["skill_name".to_string()],
            },
            CommandPattern {
                pattern: "run (.+) recipe".to_string(),
                action: VoiceAction::RunRecipe { recipe_name: "".to_string() },
                param_names: vec!["recipe_name".to_string()],
            },
            CommandPattern {
                pattern: "send (.+) message".to_string(),
                action: VoiceAction::SendMessage { content: "".to_string() },
                param_names: vec!["content".to_string()],
            },
            CommandPattern {
                pattern: "open (.+)".to_string(),
                action: VoiceAction::OpenFeature { feature: "".to_string() },
                param_names: vec!["feature".to_string()],
            },
            CommandPattern {
                pattern: "search for (.+)".to_string(),
                action: VoiceAction::Search { query: "".to_string() },
                param_names: vec!["query".to_string()],
            },
        ];

        // Korean patterns
        let ko_patterns = vec![
            CommandPattern {
                pattern: "(.+) 스킬 실행".to_string(),
                action: VoiceAction::ExecuteSkill { skill_name: "".to_string() },
                param_names: vec!["skill_name".to_string()],
            },
            CommandPattern {
                pattern: "(.+) 레시피 실행".to_string(),
                action: VoiceAction::RunRecipe { recipe_name: "".to_string() },
                param_names: vec!["recipe_name".to_string()],
            },
            CommandPattern {
                pattern: "(.+) 메시지 보내".to_string(),
                action: VoiceAction::SendMessage { content: "".to_string() },
                param_names: vec!["content".to_string()],
            },
            CommandPattern {
                pattern: "(.+) 열기".to_string(),
                action: VoiceAction::OpenFeature { feature: "".to_string() },
                param_names: vec!["feature".to_string()],
            },
        ];

        let mut language_patterns = HashMap::new();
        language_patterns.insert("en".to_string(), patterns.clone());
        language_patterns.insert("ko".to_string(), ko_patterns);

        Self {
            patterns,
            language_patterns,
        }
    }

    /// Parse a transcript into a voice command
    pub fn parse(&self, transcript: &str, language: &str) -> VoiceCommand {
        let transcript_lower = transcript.to_lowercase();

        // Try to match patterns
        let patterns = self.language_patterns.get(language).unwrap_or(&self.patterns);

        for pattern in patterns {
            if let Some(captures) = self.match_pattern(&pattern.pattern, &transcript_lower) {
                let action = match &pattern.action {
                    VoiceAction::ExecuteSkill { .. } => {
                        let skill_name = captures.first().map(|s| s.to_string()).unwrap_or_default();
                        VoiceAction::ExecuteSkill { skill_name }
                    }
                    VoiceAction::RunRecipe { .. } => {
                        let recipe_name = captures.first().map(|s| s.to_string()).unwrap_or_default();
                        VoiceAction::RunRecipe { recipe_name }
                    }
                    VoiceAction::SendMessage { .. } => {
                        let content = captures.first().map(|s| s.to_string()).unwrap_or_default();
                        VoiceAction::SendMessage { content }
                    }
                    VoiceAction::OpenFeature { .. } => {
                        let feature = captures.first().map(|s| s.to_string()).unwrap_or_default();
                        VoiceAction::OpenFeature { feature }
                    }
                    VoiceAction::Search { .. } => {
                        let query = captures.first().map(|s| s.to_string()).unwrap_or_default();
                        VoiceAction::Search { query }
                    }
                    _ => VoiceAction::Unknown,
                };

                let mut parameters = HashMap::new();
                for (i, param_name) in pattern.param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i) {
                        parameters.insert(param_name.clone(), serde_json::json!(value));
                    }
                }

                return VoiceCommand {
                    transcript: transcript.to_string(),
                    language: language.to_string(),
                    action,
                    parameters,
                    confidence: 0.8, // Fixed confidence for now
                };
            }
        }

        // No match found
        VoiceCommand {
            transcript: transcript.to_string(),
            language: language.to_string(),
            action: VoiceAction::Unknown,
            parameters: HashMap::new(),
            confidence: 0.2,
        }
    }

    /// Simple pattern matching (placeholder for regex)
    fn match_pattern(&self, pattern: &str, text: &str) -> Option<Vec<String>> {
        // Very simplified pattern matching for "execute (.+) skill" type patterns
        // Format: "prefix capture_group suffix"

        // Find the position of (.+) in the pattern
        let wildcard_pos = pattern.find("(.+)")?;
        let prefix = &pattern[..wildcard_pos];
        let suffix = &pattern[wildcard_pos + 4..]; // Skip "(.+)" which is 4 chars

        // Check if text starts with prefix
        if !text.starts_with(prefix) {
            return None;
        }

        // Check if text ends with suffix
        if !text.ends_with(suffix) {
            return None;
        }

        // Extract the middle part
        let captured = &text[prefix.len()..text.len() - suffix.len()];

        if !captured.is_empty() {
            Some(vec![captured.trim().to_string()])
        } else {
            None
        }
    }
}

impl Default for VoiceCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Natural language router for voice commands
pub struct VoiceRouter {
    /// Skill registry
    skills: HashMap<String, SkillInfo>,
    /// Recipe registry
    recipes: HashMap<String, RecipeInfo>,
}

/// Skill information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

/// Recipe information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}

impl VoiceRouter {
    /// Create a new voice router
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            recipes: HashMap::new(),
        }
    }

    /// Register a skill
    pub fn register_skill(&mut self, skill: SkillInfo) {
        self.skills.insert(skill.id.clone(), skill);
    }

    /// Register a recipe
    pub fn register_recipe(&mut self, recipe: RecipeInfo) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Route a command to the appropriate handler
    pub fn route(&self, command: &VoiceCommand) -> RoutingResult {
        match &command.action {
            VoiceAction::ExecuteSkill { skill_name } => {
                // Find skill by name or keywords
                let skill = self.find_skill(skill_name);
                RoutingResult {
                    handler_type: HandlerType::Skill,
                    target_id: skill.map(|s| s.id.clone()),
                    target_name: skill.map(|s| s.name.clone()).unwrap_or_default(),
                    confidence: command.confidence,
                }
            }
            VoiceAction::RunRecipe { recipe_name } => {
                let recipe = self.find_recipe(recipe_name);
                RoutingResult {
                    handler_type: HandlerType::Recipe,
                    target_id: recipe.map(|r| r.id.clone()),
                    target_name: recipe.map(|r| r.name.clone()).unwrap_or_default(),
                    confidence: command.confidence,
                }
            }
            VoiceAction::SendMessage { content } => {
                RoutingResult {
                    handler_type: HandlerType::Message,
                    target_id: None,
                    target_name: content.clone(),
                    confidence: command.confidence,
                }
            }
            VoiceAction::OpenFeature { feature } => {
                RoutingResult {
                    handler_type: HandlerType::Feature,
                    target_id: None,
                    target_name: feature.clone(),
                    confidence: command.confidence,
                }
            }
            VoiceAction::Search { query } => {
                RoutingResult {
                    handler_type: HandlerType::Search,
                    target_id: None,
                    target_name: query.clone(),
                    confidence: command.confidence,
                }
            }
            VoiceAction::Unknown => {
                RoutingResult {
                    handler_type: HandlerType::Unknown,
                    target_id: None,
                    target_name: "Unknown".to_string(),
                    confidence: command.confidence,
                }
            }
        }
    }

    /// Find a skill by name or keyword
    fn find_skill(&self, name: &str) -> Option<&SkillInfo> {
        // Try exact match first
        if let Some(skill) = self.skills.values().find(|s| s.id == name || s.name == name) {
            return Some(skill);
        }

        // Try keyword match
        self.skills.values().find(|skill| {
            skill.keywords.iter().any(|k| k.to_lowercase() == name.to_lowercase())
        })
    }

    /// Find a recipe by name
    fn find_recipe(&self, name: &str) -> Option<&RecipeInfo> {
        self.recipes.values().find(|r| r.id == name || r.name == name)
    }
}

impl Default for VoiceRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Routing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    pub handler_type: HandlerType,
    pub target_id: Option<String>,
    pub target_name: String,
    pub confidence: f32,
}

/// Handler type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerType {
    Skill,
    Recipe,
    Message,
    Feature,
    Search,
    Unknown,
}

/// Integration with scheduler for voice-triggered actions
pub struct VoiceSchedulerIntegration {
    /// Scheduler reference
    scheduler_running: bool,
}

impl VoiceSchedulerIntegration {
    pub fn new() -> Self {
        Self {
            scheduler_running: false,
        }
    }

    /// Schedule a voice-triggered action
    pub fn schedule_voice_action(
        &self,
        command: &VoiceCommand,
        _router: &VoiceRouter,
    ) -> Result<String, String> {
        // In production, this would schedule the action with the scheduler
        let result = match &command.action {
            VoiceAction::ExecuteSkill { skill_name } => {
                format!("Scheduled skill execution: {}", skill_name)
            }
            VoiceAction::RunRecipe { recipe_name } => {
                format!("Scheduled recipe execution: {}", recipe_name)
            }
            _ => "Command processed".to_string(),
        };

        Ok(result)
    }
}

impl Default for VoiceSchedulerIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Multilingual voice conversation manager
pub struct VoiceConversationManager {
    /// Current conversation language
    current_language: String,
    /// Conversation history
    history: Vec<VoiceMessage>,
}

/// Voice message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMessage {
    pub role: String,  // "user" or "assistant"
    pub content: String,
    pub language: String,
    pub timestamp: String,
}

impl VoiceConversationManager {
    pub fn new() -> Self {
        Self {
            current_language: "en".to_string(),
            history: Vec::new(),
        }
    }

    /// Set conversation language
    pub fn set_language(&mut self, language: String) {
        self.current_language = language;
    }

    /// Detect language from transcript (simplified)
    pub fn detect_language(&self, transcript: &str) -> String {
        // Simple heuristic: Korean characters detection
        if transcript.chars().any(is_hangul) {
            "ko".to_string()
        } else {
            "en".to_string()
        }
    }

    /// Add message to conversation
    pub fn add_message(&mut self, role: String, content: String, language: String) {
        let message = VoiceMessage {
            role,
            content,
            language,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.history.push(message);
    }

    /// Get conversation history
    pub fn get_history(&self) -> &[VoiceMessage] {
        &self.history
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for VoiceConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a character is Hangul (Korean)
fn is_hangul(c: char) -> bool {
    match c {
        '\u{AC00}'..='\u{D7A3}' => true,  // Hangul syllables
        '\u{1100}'..='\u{11FF}' => true,  // Hangul Jamo
        '\u{3131}'..='\u{318E}' => true,  // Hangul compatibility Jamo
        _ => false,
    }
}

// ============================================================================
// Tauri Commands - Voice Command Parsing (v0.5)
// ============================================================================

use crate::voice::ParsedVoiceCommand;

/// Parse voice command transcript
#[tauri::command]
pub fn parse_voice_command(
    transcript: String,
    language: Option<String>,
) -> Result<ParsedVoiceCommand, String> {
    let parser = VoiceCommandParser::new();
    let lang = language.unwrap_or_else(|| {
        // Auto-detect language
        if transcript.chars().any(is_hangul) {
            "ko".to_string()
        } else {
            "en".to_string()
        }
    });

    let command = parser.parse(&transcript, &lang);

    // Convert VoiceAction to serializable format (use mod.rs VoiceAction)
    use crate::voice::VoiceAction as ModVoiceAction;
    let action = match &command.action {
        VoiceAction::ExecuteSkill { skill_name } => ModVoiceAction::ExecuteSkill {
            skill_name: skill_name.clone(),
        },
        VoiceAction::RunRecipe { recipe_name } => ModVoiceAction::RunRecipe {
            recipe_name: recipe_name.clone(),
        },
        VoiceAction::SendMessage { content } => ModVoiceAction::SendMessage {
            content: content.clone(),
        },
        VoiceAction::OpenFeature { feature } => ModVoiceAction::OpenFeature {
            feature: feature.clone(),
        },
        VoiceAction::Search { query } => ModVoiceAction::Search {
            query: query.clone(),
        },
        VoiceAction::Unknown => ModVoiceAction::Unknown,
    };

    Ok(ParsedVoiceCommand {
        transcript: command.transcript,
        language: command.language,
        action,
        parameters: command.parameters,
        confidence: command.confidence,
    })
}

/// Detect language from transcript
#[tauri::command]
pub fn detect_voice_language(transcript: String) -> String {
    if transcript.chars().any(is_hangul) {
        "ko".to_string()
    } else {
        "en".to_string()
    }
}

/// Validate voice command format
#[tauri::command]
pub fn validate_voice_command(transcript: String) -> Result<bool, String> {
    if transcript.trim().is_empty() {
        return Ok(false);
    }

    // Check if transcript contains valid alphanumeric characters and common punctuation
    for c in transcript.chars() {
        if !(c.is_alphabetic() || c.is_whitespace() || c.is_ascii_punctuation() || c.is_numeric()) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Get supported voice command patterns
#[tauri::command]
pub fn get_voice_command_patterns(language: String) -> Vec<VoiceCommandPattern> {
    let parser = VoiceCommandParser::new();

    // Extract patterns from parser
    let patterns = parser.language_patterns.get(&language).unwrap_or(&parser.patterns);

    patterns.iter().map(|p| {
        let action_type = match &p.action {
            VoiceAction::ExecuteSkill { .. } => "execute_skill",
            VoiceAction::RunRecipe { .. } => "run_recipe",
            VoiceAction::SendMessage { .. } => "send_message",
            VoiceAction::OpenFeature { .. } => "open_feature",
            VoiceAction::Search { .. } => "search",
            VoiceAction::Unknown => "unknown",
        }.to_string();

        VoiceCommandPattern {
            pattern: p.pattern.clone(),
            action_type,
            param_names: p.param_names.clone(),
        }
    }).collect()
}

/// Voice command pattern description
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceCommandPattern {
    pub pattern: String,
    pub action_type: String,
    pub param_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = VoiceCommandParser::new();
        assert!(!parser.patterns.is_empty());
    }

    #[test]
    fn test_parse_english_command() {
        let parser = VoiceCommandParser::new();
        let command = parser.parse("execute data processing skill", "en");

        match command.action {
            VoiceAction::ExecuteSkill { skill_name } => {
                assert_eq!(skill_name, "data processing");
            }
            _ => panic!("Expected ExecuteSkill action"),
        }
    }

    #[test]
    fn test_router_creation() {
        let router = VoiceRouter::new();
        assert_eq!(router.skills.len(), 0);
        assert_eq!(router.recipes.len(), 0);
    }

    #[test]
    fn test_language_detection() {
        let manager = VoiceConversationManager::new();
        assert_eq!(manager.detect_language("Hello"), "en");
        assert_eq!(manager.detect_language("안녕"), "ko");
    }
}
