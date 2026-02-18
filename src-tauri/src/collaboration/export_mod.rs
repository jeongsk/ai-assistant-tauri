// Export and Import functionality

use crate::collaboration::{ExportFormat, ExportOptions};
use serde::{Deserialize, Serialize};

/// Conversation export data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationExport {
    pub version: String,
    pub exported_at: String,
    pub conversations: Vec<ExportedConversation>,
}

/// Exported conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub messages: Vec<ExportedMessage>,
}

/// Exported message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMessage {
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// Export conversations to format
pub fn export_conversations(
    conversations: Vec<ExportedConversation>,
    options: &ExportOptions,
) -> Result<Vec<u8>, String> {
    match options.format {
        ExportFormat::Json => export_to_json(&conversations, options),
        ExportFormat::Markdown => export_to_markdown(&conversations, options),
        ExportFormat::Html => export_to_html(&conversations, options),
    }
}

fn export_to_json(
    conversations: &[ExportedConversation],
    options: &ExportOptions,
) -> Result<Vec<u8>, String> {
    let export = ConversationExport {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        conversations: conversations.to_vec(),
    };

    if options.pretty_print {
        serde_json::to_string_pretty(&export)
            .map(|s| s.into_bytes())
            .map_err(|e| e.to_string())
    } else {
        serde_json::to_vec(&export)
            .map_err(|e| e.to_string())
    }
}

fn export_to_markdown(
    conversations: &[ExportedConversation],
    options: &ExportOptions,
) -> Result<Vec<u8>, String> {
    let mut output = String::new();

    output.push_str("# Conversation Export\n\n");

    for conv in conversations {
        output.push_str(&format!("## {}\n\n", conv.title));

        if options.include_timestamps {
            output.push_str(&format!("*Created: {}*\n\n", conv.created_at));
        }

        for msg in &conv.messages {
            let role = match msg.role.as_str() {
                "user" => "👤 **You**",
                "assistant" => "🤖 **Assistant**",
                _ => "**System**",
            };

            output.push_str(&format!("{}\n\n{}\n\n---\n\n", role, msg.content));
        }

        output.push_str("\n");
    }

    Ok(output.into_bytes())
}

fn export_to_html(
    conversations: &[ExportedConversation],
    options: &ExportOptions,
) -> Result<Vec<u8>, String> {
    let mut output = String::new();

    output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    output.push_str("<meta charset=\"UTF-8\">\n");
    output.push_str("<title>Conversation Export</title>\n");
    output.push_str("<style>\n");
    output.push_str("body { font-family: -apple-system, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
    output.push_str(".message { margin: 16px 0; padding: 12px; border-radius: 8px; }\n");
    output.push_str(".user { background: #e3f2fd; }\n");
    output.push_str(".assistant { background: #f5f5f5; }\n");
    output.push_str(".conversation { border: 1px solid #ddd; margin: 20px 0; padding: 20px; border-radius: 8px; }\n");
    output.push_str("</style>\n</head>\n<body>\n");

    for conv in conversations {
        output.push_str("<div class=\"conversation\">\n");
        output.push_str(&format!("<h2>{}</h2>\n", conv.title));

        if options.include_timestamps {
            output.push_str(&format!("<p><em>Created: {}</em></p>\n", conv.created_at));
        }

        for msg in &conv.messages {
            let class = msg.role.as_str();
            output.push_str(&format!(
                "<div class=\"message {}\"><strong>{}</strong><p>{}</p></div>\n",
                class, msg.role, msg.content
            ));
        }

        output.push_str("</div>\n");
    }

    output.push_str("</body>\n</html>");

    Ok(output.into_bytes())
}

/// Import conversations from JSON
pub fn import_from_json(data: &[u8]) -> Result<Vec<ExportedConversation>, String> {
    let export: ConversationExport = serde_json::from_slice(data)
        .map_err(|e| format!("Failed to parse import: {}", e))?;

    Ok(export.conversations)
}
