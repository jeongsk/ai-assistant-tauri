// Voice Module - STT and TTS integration

#![allow(dead_code)]

pub mod stt;
pub mod tts;
pub mod commands;


use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Voice settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub enabled: bool,
    pub stt_model: String,
    pub tts_voice: String,
    pub language: String,
    pub wake_word: Option<String>,
    pub vad_sensitivity: f32,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            stt_model: "base".to_string(),
            tts_voice: "default".to_string(),
            language: "en".to_string(),
            wake_word: None,
            vad_sensitivity: 0.5,
        }
    }
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
    pub duration_ms: u64,
}

/// TTS synthesis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

// ============================================================================
// Voice Command Types (v0.5 - exported for Tauri commands)
// ============================================================================

/// Voice action types (re-export from commands)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceAction {
    ExecuteSkill { skill_name: String },
    RunRecipe { recipe_name: String },
    SendMessage { content: String },
    OpenFeature { feature: String },
    Search { query: String },
    Unknown,
}

/// Parsed voice command (v0.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedVoiceCommand {
    pub transcript: String,
    pub language: String,
    pub action: VoiceAction,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: f32,
}
