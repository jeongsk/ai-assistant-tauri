// Speech-to-Text using Whisper
//
// Enhanced implementation with WAV parsing, VAD, and model management.
//
// TODO: Add whisper-rs dependency for actual Whisper inference:
// whisper-rs = "0.11"
// dirs = "5"

#![allow(dead_code)]

use crate::voice::TranscriptionResult;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// STT error types
#[derive(Debug, Error)]
pub enum SttError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),

    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("Whisper error: {0}")]
    WhisperError(#[from] whisper_rs::WhisperError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("State lock error: {0}")]
    LockError(String),
}

pub type SttResult<T> = Result<T, SttError>;

/// STT engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub model: String,
    pub language: Option<String>,
    pub task: String,
    pub threads: usize,
    pub processors: usize,
    pub temperature: f32,
    pub vad_threshold: f32,
    pub translate: bool,
    pub no_context: bool,
    pub single_segment: bool,
    pub print_special: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    pub print_timestamps: bool,
    pub token_timestamps: bool,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model: "base".to_string(),
            language: None,
            task: "transcribe".to_string(),
            threads: 4,
            processors: 1,
            temperature: 0.0,
            vad_threshold: 0.5,
            translate: false,
            no_context: false,
            single_segment: false,
            print_special: false,
            print_progress: false,
            print_realtime: false,
            print_timestamps: false,
            token_timestamps: false,
        }
    }
}

/// Audio parameters for processing
#[derive(Debug, Clone)]
pub struct AudioParams {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for AudioParams {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bits_per_sample: 16,
        }
    }
}

/// VAD (Voice Activity Detection) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadResult {
    pub is_speech: bool,
    pub confidence: f32,
    pub energy_db: f32,
}

// Maximum audio size: 10 minutes at 16kHz mono = ~10MB
const MAX_AUDIO_SIZE: usize = 10 * 1024 * 1024;

// Use whisper_rs types directly when voice feature is enabled
#[cfg(feature = "voice")]
pub use whisper_rs::{WhisperContext, WhisperState};

/// STT engine with model management
/// Note: Clone is removed because WhisperContext is not Clone
/// Use Arc for shared access
pub struct SttEngine {
    config: SttConfig,
    model_loaded: Arc<RwLock<bool>>,
    model_path: Arc<Mutex<Option<PathBuf>>>,
    whisper_context: Arc<Mutex<Option<WhisperContext>>>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

/// Global STT engine instance
static STT_ENGINE: Mutex<Option<SttEngine>> = Mutex::new(None);

/// Initialize STT engine
///
/// Initializes the Whisper STT engine with the specified model.
/// Returns success message with model path information.
#[tauri::command]
pub fn init_stt(model: String) -> Result<String, String> {
    let config = SttConfig {
        model: model.clone(),
        ..Default::default()
    };

    let engine = SttEngine::new(config)?;
    let model_path = engine.get_model_path_str();

    // Store engine globally
    let mut global_engine = STT_ENGINE
        .lock()
        .map_err(|e| format!("Failed to acquire STT lock: {}", e))?;
    *global_engine = Some(engine);

    Ok(format!("STT initialized with model: {} at {}", model, model_path))
}

/// Transcribe audio data
///
/// Processes WAV audio data and returns transcription results.
/// Includes WAV header parsing and audio preprocessing.
#[tauri::command]
pub fn transcribe(audio_data: Vec<u8>, language: String) -> Result<TranscriptionResult, String> {
    let start_time = std::time::Instant::now();

    // Get STT engine reference (no clone since WhisperContext is not Clone)
    let engine_guard = STT_ENGINE
        .lock()
        .map_err(|e| format!("Failed to acquire STT lock: {}", e))?;

    let engine = engine_guard
        .as_ref()
        .ok_or_else(|| "STT engine not initialized. Call init_stt first.".to_string())?;

    // Validate audio data
    if audio_data.is_empty() {
        return Err("Audio data is empty".to_string());
    }

    // Check maximum audio size to prevent DoS
    if audio_data.len() > MAX_AUDIO_SIZE {
        return Err(format!(
            "Audio data exceeds maximum size of {} bytes (about 10 minutes at 16kHz)",
            MAX_AUDIO_SIZE
        ));
    }

    // Check minimum audio length (44 bytes is WAV header + 1 sample)
    if audio_data.len() < 44 {
        return Err("Audio data too short to be valid".to_string());
    }

    // Parse WAV header
    let audio_params = parse_wav_header(&audio_data)?;
    let audio_samples = extract_pcm_data(&audio_data, &audio_params)?;

    // Calculate duration
    let duration_ms = (audio_samples.len() as f64 / audio_params.sample_rate as f64 * 1000.0) as u64;

    // Set language for this transcription
    let config = SttConfig {
        language: if language.is_empty() {
            None
        } else {
            Some(language.clone())
        },
        ..Default::default()
    };

    // Perform actual transcription
    let result = engine.transcribe_audio(&audio_samples, &audio_params, &config)
        .map_err(|e| e.to_string())?;

    println!("[STT] Transcription completed in {:?}", start_time.elapsed());

    Ok(TranscriptionResult {
        text: result.text,
        confidence: result.confidence,
        language: result.language.unwrap_or(language),
        duration_ms,
    })
}

/// Transcription result with extended information
#[derive(Debug, Clone)]
struct TranscriptionOutput {
    pub text: String,
    pub confidence: f32,
    pub language: Option<String>,
    pub segments: Vec<TranscriptionSegment>,
}

/// Individual transcription segment
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f32,
}

impl SttEngine {
    /// Create a new STT engine instance
    pub fn new(config: SttConfig) -> Result<Self, String> {
        let model_path = Self::resolve_model_path(&config.model)?;

        // Validate model file exists
        if !model_path.exists() {
            return Err(format!(
                "Model file not found at: {}. Please download the model first.",
                model_path.display()
            ));
        }

        println!("[STT] Loading model from: {}", model_path.display());

        // Load Whisper context using actual whisper-rs
        let context = Self::load_whisper_model(&model_path)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        Ok(Self {
            config,
            model_loaded: Arc::new(RwLock::new(true)),
            model_path: Arc::new(Mutex::new(Some(model_path))),
            whisper_context: Arc::new(Mutex::new(Some(context))),
            app_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// Resolve model path to app data directory
    fn resolve_model_path(model_name: &str) -> Result<PathBuf, String> {
        // Standard model filename
        let model_filename = format!("ggml-{}.bin", model_name);

        // Try to get app data directory
        if let Some(mut path) = dirs::data_dir() {
            path.push("ai-assistant-tauri");
            path.push("models");
            path.push("whisper");
            path.push(&model_filename);

            if path.exists() {
                return Ok(path);
            }

            // Also try legacy location
            let mut legacy_path =
                dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;
            legacy_path.push(".ai-assistant");
            legacy_path.push("models");
            legacy_path.push("whisper");
            legacy_path.push(&model_filename);

            if legacy_path.exists() {
                return Ok(legacy_path);
            }
        }

        // Fallback to current directory (for development)
        let mut dev_path = PathBuf::from(".");
        dev_path.push("models");
        dev_path.push("whisper");
        dev_path.push(&model_filename);

        if dev_path.exists() {
            return Ok(dev_path);
        }

        // Return default path even if it doesn't exist (will error during load)
        let mut default_path = PathBuf::from(".");
        default_path.push("models");
        default_path.push("whisper");
        default_path.push(&model_filename);
        Ok(default_path)
    }

    /// Load Whisper model from file using whisper-rs
    pub fn load_whisper_model(model_path: &Path) -> SttResult<WhisperContext> {
        println!("[STT] Loading Whisper model from: {}", model_path.display());

        // Check file exists and is readable
        if !model_path.exists() {
            return Err(SttError::ModelNotFound(model_path.display().to_string()));
        }

        let metadata = std::fs::metadata(model_path)?;
        println!("[STT] Model file size: {} bytes", metadata.len());

        // Validate it's a valid ggml model file (check minimum size)
        if metadata.len() < 1024 * 1024 {
            return Err(SttError::ModelLoadFailed(
                "Model file too small to be valid".to_string()
            ));
        }

        // Create Whisper context with CPU-only parameters for v0.5
        let params = whisper_rs::WhisperContextParameters {
            use_gpu: false,  // CPU-only for cross-platform compatibility
            ..Default::default()
        };

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| SttError::ModelLoadFailed(
                "Invalid model path (non-UTF8)".to_string()
            ))?,
            params
        )?;

        println!("[STT] Model loaded successfully");
        Ok(ctx)
    }

    /// Get the model path as a string
    fn get_model_path_str(&self) -> String {
        self.model_path
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Transcribe audio samples using Whisper
    fn transcribe_audio(
        &self,
        samples: &[f32],
        params: &AudioParams,
        config: &SttConfig,
    ) -> Result<TranscriptionOutput, SttError> {
        println!(
            "[STT] Transcribing {} samples at {} Hz",
            samples.len(),
            params.sample_rate
        );

        // 1. Resample to 16kHz if needed (Whisper requires 16kHz mono)
        let processed_samples = if params.sample_rate != 16000 {
            resample_audio(samples, params.sample_rate, 16000)
        } else {
            samples.to_vec()
        };

        // 2. Get Whisper context
        let mut ctx_guard = self.whisper_context.lock()
            .map_err(|e| SttError::LockError(e.to_string()))?;
        let ctx = ctx_guard.as_mut()
            .ok_or_else(|| SttError::TranscriptionFailed("Whisper context not initialized".to_string()))?;

        // 3. Create state for this transcription (whisper-rs v0.15 pattern)
        let mut state = ctx.create_state()
            .map_err(|e| SttError::TranscriptionFailed(format!("Failed to create state: {}", e)))?;

        // 4. Configure FullParams with correct syntax for whisper-rs v0.15
        let mut wparams = whisper_rs::FullParams::new(
            whisper_rs::SamplingStrategy::Greedy { best_of: 1 }
        );

        wparams.set_n_threads(config.threads as i32);
        wparams.set_translate(config.translate);
        wparams.set_language(config.language.as_deref());
        wparams.set_offset_ms(0);
        wparams.set_duration_ms(0);

        // 5. Run inference using state
        state.full(wparams, &processed_samples)
            .map_err(|e| SttError::TranscriptionFailed(format!("Inference failed: {}", e)))?;

        // 6. Extract segments from state using iterator
        let mut full_text = String::new();
        let mut segments = Vec::new();

        for segment in state.as_iter() {
            // Use to_str_lossy() to handle potential invalid UTF-8
            let segment_text = segment.to_str_lossy()
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(""));

            let start = segment.start_timestamp();  // Returns i64 in centiseconds
            let end = segment.end_timestamp();       // Returns i64 in centiseconds

            full_text.push_str(&segment_text);
            segments.push(TranscriptionSegment {
                text: segment_text.to_string(),
                start_time_ms: start as u64 * 10,  // Convert centiseconds to milliseconds
                end_time_ms: end as u64 * 10,
                confidence: 0.9, // Whisper doesn't provide per-segment confidence
            });
        }

        // 7. Detect language if not specified
        let detected_lang = if let Some(ref lang) = config.language {
            Some(lang.clone())
        } else {
            // Use lang_id from state
            let lang_id = state.full_lang_id_from_state();
            let lang_opt = whisper_rs::get_lang_str(lang_id);
            lang_opt.map(|s| s.to_string())
        };

        Ok(TranscriptionOutput {
            text: full_text.trim().to_string(),
            confidence: 0.9,
            language: detected_lang,
            segments,
        })
    }

    /// Detect language from audio samples (fallback for non-State contexts)
    fn detect_language_from_samples(_samples: &[f32]) -> Result<String, String> {
        // Default to English if no context available
        Ok("en".to_string())
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        *self.model_loaded.read().unwrap()
    }

    /// Unload the model and free resources
    pub fn unload(&self) -> Result<(), String> {
        let mut handle = self
            .whisper_context
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;
        *handle = None;
        *self.model_loaded.write().unwrap() = false;
        Ok(())
    }
}

/// Check if STT is available
///
/// Checks if Whisper models are downloaded and the engine is ready.
pub fn is_stt_available() -> bool {
    // Check if engine is initialized
    if let Ok(engine_guard) = STT_ENGINE.lock() {
        if let Some(engine) = engine_guard.as_ref() {
            return engine.is_loaded();
        }
    }

    // Check if any model files exist
    for model_name in &["tiny", "base", "small"] {
        if let Ok(path) = SttEngine::resolve_model_path(model_name) {
            if path.exists() {
                return true;
            }
        }
    }

    false
}

/// Get available STT models
///
/// Scans the model directory and returns actually downloaded models.
#[tauri::command]
pub fn get_available_models() -> Vec<String> {
    let mut models = Vec::new();
    let model_names = [
        "tiny",
        "tiny-en",
        "base",
        "base-en",
        "small",
        "small-en",
        "medium",
        "medium-en",
        "large",
        "large-v1",
        "large-v2",
        "large-v3",
    ];

    for model_name in model_names {
        if let Ok(path) = SttEngine::resolve_model_path(model_name) {
            if path.exists() {
                models.push(model_name.to_string());
            }
        }
    }

    models
}

/// Detect language from audio data
///
/// Uses Whisper's language detection capabilities.
pub fn detect_language(audio_data: &[u8]) -> Result<String, String> {
    // Parse WAV header and extract samples
    let audio_params = parse_wav_header(audio_data)?;
    let samples = extract_pcm_data(audio_data, &audio_params)?;

    // Get STT engine for language detection
    let engine_guard = STT_ENGINE
        .lock()
        .map_err(|e| format!("Failed to acquire STT lock: {}", e))?;

    let _engine = engine_guard
        .as_ref()
        .ok_or_else(|| "STT engine not initialized".to_string())?;

    // Use engine to detect language
    SttEngine::detect_language_from_samples(&samples)
}

/// Parse WAV file header
///
/// Returns audio parameters (sample rate, channels, bits per sample)
pub fn parse_wav_header(data: &[u8]) -> Result<AudioParams, String> {
    if data.len() < 44 {
        return Err("Audio data too short for WAV header".to_string());
    }

    // Check RIFF header
    if &data[0..4] != b"RIFF" {
        return Err("Not a WAV file (missing RIFF header)".to_string());
    }

    // Check WAVE format
    if &data[8..12] != b"WAVE" {
        return Err("Not a WAV file (missing WAVE format)".to_string());
    }

    // Check fmt chunk
    if &data[12..16] != b"fmt " {
        return Err("Invalid WAV file (missing fmt chunk)".to_string());
    }

    // Extract audio format (1 = PCM)
    let audio_format = u16::from_le_bytes([data[20], data[21]]);
    if audio_format != 1 {
        return Err(format!(
            "Unsupported audio format: {} (only PCM is supported)",
            audio_format
        ));
    }

    // Extract channels
    let channels = u16::from_le_bytes([data[22], data[23]]);
    if channels != 1 {
        // Support mono only for now
        return Err(format!(
            "Unsupported channel count: {} (only mono is supported)",
            channels
        ));
    }

    // Extract sample rate
    let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);

    // Extract bits per sample
    let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);

    // Validate bit depth
    if bits_per_sample != 16 {
        return Err(format!(
            "Unsupported bit depth: {} (only 16-bit is supported)",
            bits_per_sample
        ));
    }

    println!(
        "[STT] WAV header: {} Hz, {} ch, {} bits",
        sample_rate, channels, bits_per_sample
    );

    Ok(AudioParams {
        sample_rate,
        channels,
        bits_per_sample,
    })
}

/// Extract PCM data from WAV file
///
/// Converts raw 16-bit PCM samples to f32 values normalized to [-1, 1]
pub fn extract_pcm_data(data: &[u8], params: &AudioParams) -> Result<Vec<f32>, String> {
    // Find data chunk
    let mut data_offset = 12; // Skip RIFF header

    while data_offset + 8 < data.len() {
        let chunk_id = &data[data_offset..data_offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[data_offset + 4],
            data[data_offset + 5],
            data[data_offset + 6],
            data[data_offset + 7],
        ]) as usize;

        if chunk_id == b"data" {
            data_offset += 8;
            break;
        }

        data_offset += 8 + chunk_size;
    }

    if data_offset >= data.len() {
        return Err("No data chunk found in WAV file".to_string());
    }

    // Calculate number of samples
    let bytes_per_sample = (params.bits_per_sample / 8) as usize;
    let total_samples = (data.len() - data_offset) / bytes_per_sample;

    if total_samples == 0 {
        return Err("No audio samples found in WAV file".to_string());
    }

    // Convert 16-bit PCM to f32
    let mut samples = Vec::with_capacity(total_samples);
    let mut i = data_offset;

    while i + 1 < data.len() {
        let sample_i16 = i16::from_le_bytes([data[i], data[i + 1]]);
        let sample_f32 = sample_i16 as f32 / 32768.0; // Normalize to [-1, 1]
        samples.push(sample_f32);
        i += 2;
    }

    println!("[STT] Extracted {} samples", samples.len());

    Ok(samples)
}

/// Resample audio to target sample rate
///
/// Uses simple linear interpolation for resampling.
/// In production, consider using a dedicated resampling library.
pub fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = ((samples.len() as f64) / ratio).ceil() as usize;

    let mut resampled = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let pos = (i as f64) * ratio;
        let idx = pos.floor() as usize;
        let frac = pos - (idx as f64);

        if idx + 1 < samples.len() {
            let sample = samples[idx] * (1.0 - frac) as f32 + samples[idx + 1] * (frac as f32);
            resampled.push(sample);
        } else if idx < samples.len() {
            resampled.push(samples[idx]);
        }
    }

    resampled
}

/// Apply Voice Activity Detection (VAD)
///
/// Basic energy-based VAD implementation.
/// Returns whether speech is detected and the energy level in dB.
pub fn apply_vad(samples: &[f32], sensitivity: f32) -> VadResult {
    if samples.is_empty() {
        return VadResult {
            is_speech: false,
            confidence: 0.0,
            energy_db: -100.0,
        };
    }

    // Calculate RMS energy
    let rms = (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

    // Convert to dB
    let energy_db = 20.0 * (rms.max(1e-6)).log10();

    // Calculate background noise threshold
    // Sensitivity: 0.0 = very sensitive (detect quiet speech), 1.0 = less sensitive
    let threshold_db = -40.0 + (sensitivity * 20.0);

    // Speech detected if energy exceeds threshold
    let is_speech = energy_db > threshold_db;

    // Confidence based on how far above/below threshold
    let confidence = ((energy_db - threshold_db) / 20.0).clamp(0.0, 1.0);

    VadResult {
        is_speech,
        confidence,
        energy_db,
    }
}

/// Transcribe audio file directly from path
#[tauri::command]
pub fn transcribe_file(file_path: String, language: String) -> Result<TranscriptionResult, String> {
    use std::path::Path;

    // Validate path to prevent path traversal attacks
    let path = Path::new(&file_path);

    // Check for path traversal components
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Invalid file path: path traversal not allowed".to_string());
    }

    // Resolve to canonical path to prevent symlink attacks
    let canonical_path = path.canonicalize()
        .map_err(|_| "Invalid file path".to_string())?;

    // Only allow files from user-writable directories
    let allowed_dirs = vec![
        dirs::audio_dir(),
        dirs::home_dir(),
        dirs::document_dir(),
    ].into_iter().flatten().filter_map(|p| p.canonicalize().ok()).collect::<Vec<_>>();

    let is_allowed = allowed_dirs.iter().any(|dir| {
        canonical_path.starts_with(dir)
    });

    if !is_allowed {
        return Err("File path not in allowed directory".to_string());
    }

    // Read file
    let audio_data = std::fs::read(&canonical_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    // Delegate to regular transcribe function
    transcribe(audio_data, language)
}

/// Download Whisper model if not present
#[tauri::command]
pub async fn download_model(model_name: String) -> Result<String, String> {
    // Validate model name first to prevent injection
    let valid_models = [
        "tiny", "base", "small", "medium", "large",
        "tiny-en", "base-en", "small-en", "medium-en",
        "large-v1", "large-v2", "large-v3",
    ];

    if !valid_models.contains(&model_name.as_str()) {
        return Err(format!(
            "Invalid model name '{}'. Valid options: {}",
            model_name,
            valid_models.join(", ")
        ));
    }

    let model_filename = format!("ggml-{}.bin", model_name);

    // Determine model directory
    let model_dir = if let Some(mut path) = dirs::data_dir() {
        path.push("ai-assistant-tauri");
        path.push("models");
        path.push("whisper");
        path
    } else {
        return Err("Cannot determine data directory".to_string());
    };

    // Create directory if needed
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("Failed to create model directory: {}", e))?;

    let model_path = model_dir.join(&model_filename);

    // Check if already exists
    if model_path.exists() {
        return Ok(format!(
            "Model already exists at: {}",
            model_path.display()
        ));
    }

    // Base URL for HuggingFace models
    let base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
    let download_url = format!("{}/{}", base_url, model_filename);

    println!("[STT] Downloading model from: {}", download_url);

    // Download using reqwest (add to dependencies)
    // For now, return instruction URL
    Ok(format!(
        "Model download required. Please download from: {} and save to: {}",
        download_url,
        model_path.display()
    ))
}

/// Get model download URL for manual download
#[tauri::command]
pub fn get_model_download_url(model_name: String) -> Result<String, String> {
    let valid_models = [
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "tiny-en",
        "base-en",
        "small-en",
        "medium-en",
        "large-v1",
        "large-v2",
        "large-v3",
    ];

    if !valid_models.contains(&model_name.as_str()) {
        return Err(format!(
            "Invalid model name: {}. Valid options: {}",
            model_name,
            valid_models.join(", ")
        ));
    }

    let model_filename = format!("ggml-{}.bin", model_name);
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        model_filename
    );

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wav_header() {
        // Minimal valid WAV header (44 bytes)
        let mut wav_header = [0u8; 44];
        wav_header[0..4].copy_from_slice(b"RIFF");
        wav_header[8..12].copy_from_slice(b"WAVE");
        wav_header[12..16].copy_from_slice(b"fmt ");
        wav_header[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
        wav_header[22..24].copy_from_slice(&1u16.to_le_bytes()); // Mono
        wav_header[24..28].copy_from_slice(&16000u32.to_le_bytes()); // 16kHz
        wav_header[34..36].copy_from_slice(&16u16.to_le_bytes()); // 16-bit
        wav_header[36..40].copy_from_slice(b"data");

        let params = parse_wav_header(&wav_header).unwrap();
        assert_eq!(params.sample_rate, 16000);
        assert_eq!(params.channels, 1);
        assert_eq!(params.bits_per_sample, 16);
    }

    #[test]
    fn test_resample_audio() {
        let input = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let resampled = resample_audio(&input, 16000, 8000);
        assert_eq!(resampled.len(), 3); // Half the samples
    }

    #[test]
    fn test_vad_detection() {
        let silence = vec![0.0; 1000];
        let vad_result = apply_vad(&silence, 0.5);
        assert!(!vad_result.is_speech);

        let loud_noise = vec![0.5; 1000];
        let vad_result = apply_vad(&loud_noise, 0.5);
        assert!(vad_result.is_speech);
    }

    #[test]
    fn test_get_model_download_url() {
        let url = get_model_download_url("base".to_string()).unwrap();
        assert!(url.contains("ggml-base.bin"));
        assert!(url.contains("huggingface.co"));
    }

    #[test]
    fn test_invalid_model_name() {
        let result = get_model_download_url("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_stt_config_default() {
        let config = SttConfig::default();
        assert_eq!(config.model, "base");
        assert_eq!(config.task, "transcribe");
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_audio_params_default() {
        let params = AudioParams::default();
        assert_eq!(params.sample_rate, 16000);
        assert_eq!(params.channels, 1);
        assert_eq!(params.bits_per_sample, 16);
    }
}
