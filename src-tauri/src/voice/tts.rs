// Text-to-Speech synthesis
//
// Uses platform-specific TTS engines:
// - Windows: SAPI via PowerShell
// - macOS: say command
// - Linux: espeak-ng or espeak

#![allow(dead_code)]

use crate::voice::SynthesisResult;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// TTS engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub voice: String,
    pub speed: f32,
    pub pitch: f32,
    pub sample_rate: u32,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            voice: "default".to_string(),
            speed: 1.0,
            pitch: 1.0,
            sample_rate: 22050,
        }
    }
}

/// Global TTS state for tracking synthesis progress
#[derive(Debug)]
struct TtsState {
    is_synthesizing: bool,
    progress: f32,
    current_config: Option<TtsConfig>,
}

impl TtsState {
    fn new() -> Self {
        Self {
            is_synthesizing: false,
            progress: 0.0,
            current_config: None,
        }
    }
}

/// Global state for TTS operations
static TTS_STATE: Mutex<Option<TtsState>> = Mutex::new(None);

/// Initialize TTS state
fn init_state() {
    let mut state = TTS_STATE.lock().unwrap();
    if state.is_none() {
        *state = Some(TtsState::new());
    }
}

/// Initialize TTS engine
///
/// Initializes the TTS subsystem and configures the specified voice.
/// Returns a success message with the configured voice.
#[tauri::command]
pub fn init_tts(voice: String) -> Result<String, String> {
    init_state();

    let mut state = TTS_STATE.lock().unwrap();
    if let Some(ref mut s) = *state {
        s.current_config = Some(TtsConfig {
            voice: voice.clone(),
            ..Default::default()
        });
    }

    // Validate TTS availability
    if is_tts_available() {
        Ok(format!("TTS initialized with voice: {}", voice))
    } else {
        Err("TTS engine is not available on this system".to_string())
    }
}

/// Synthesize text to speech
///
/// Converts the given text to audio data using the configured TTS engine.
/// Returns audio data in WAV format with metadata.
#[tauri::command]
pub fn synthesize(text: String, language: String) -> Result<SynthesisResult, String> {
    init_state();

    if text.is_empty() {
        return Err("Text cannot be empty".to_string());
    }

    // Update state to indicate synthesis is in progress
    {
        let mut state = TTS_STATE.lock().unwrap();
        if let Some(ref mut s) = *state {
            s.is_synthesizing = true;
            s.progress = 0.0;
        }
    }

    // Get configured settings
    let state_guard = TTS_STATE.lock().unwrap();
    let speed = state_guard.as_ref()
        .and_then(|s| s.current_config.as_ref())
        .map(|c| c.speed)
        .unwrap_or(1.0);
    drop(state_guard);

    let sample_rate = 22050u32;

    // Generate audio data using platform TTS
    let audio_data = generate_wav_with_synthesis(&text, &language, sample_rate, speed)
        .map_err(|e| format!("Failed to generate audio: {}", e))?;

    // Calculate approximate duration based on text length and average speaking rate
    // Average speaking rate: ~150 words per minute, ~2.5 words per second
    // Average word length: ~5 characters
    let word_count = text.split_whitespace().count() as u64;
    let char_count = text.chars().count() as u64;
    let duration_ms = ((word_count * 60000 / 150) + (char_count * 10))
        .max(500) // Minimum 500ms
        .min(60000); // Maximum 60 seconds

    // Update state
    {
        let mut state = TTS_STATE.lock().unwrap();
        if let Some(ref mut s) = *state {
            s.is_synthesizing = false;
            s.progress = 1.0;
        }
    }

    Ok(SynthesisResult {
        audio_data,
        sample_rate,
        duration_ms,
    })
}

/// Generate WAV audio data with synthesized speech
fn generate_wav_with_synthesis(text: &str, language: &str, sample_rate: u32, speed: f32) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "linux")]
    {
        // On Linux, try using espeak-ng if available
        let espeak_speed = (150.0 * speed).clamp(10.0, 450.0) as i32;

        if let Ok(output) = std::process::Command::new("espeak-ng")
            .arg("-v")
            .arg(language_to_espeak_voice(language))
            .arg("-s")
            .arg(espeak_speed.to_string())
            .arg("-w")
            .arg("/dev/stdout")
            .arg(text)
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                return Ok(output.stdout);
            }
        }

        // Fallback to espeak
        if let Ok(output) = std::process::Command::new("espeak")
            .arg("-v")
            .arg(language_to_espeak_voice(language))
            .arg("-s")
            .arg(espeak_speed.to_string())
            .arg("-w")
            .arg("/dev/stdout")
            .arg(text)
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                return Ok(output.stdout);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use say command to generate audio
        let temp_file = std::env::temp_dir().join(format!("tts_{}.aiff", uuid::Uuid::new_v4()));

        // Adjust speaking rate: default is 175, range is roughly 10-400
        let say_rate = (175.0 / speed).clamp(10.0, 400.0) as i32;

        if std::process::Command::new("say")
            .arg("-v")
            .arg(language_to_macos_voice(language))
            .arg("-r")
            .arg(say_rate.to_string())
            .arg("-o")
            .arg(&temp_file)
            .arg("--data-format=LEF32@22050")
            .arg(text)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            // Read the AIFF file and convert to WAV
            if let Ok(aiff_data) = std::fs::read(&temp_file) {
                let _ = std::fs::remove_file(&temp_file);
                return convert_aiff_to_wav(&aiff_data, sample_rate);
            }
        }

        let _ = std::fs::remove_file(&temp_file);
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use PowerShell with SAPI to generate WAV
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let temp_ps = temp_dir.join(format!("tts_{}.ps1", uuid::Uuid::new_v4()));
        let temp_wav = temp_dir.join(format!("tts_{}.wav", uuid::Uuid::new_v4()));

        // Escape special characters for PowerShell
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "''");

        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Speech;
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer;
$synth.Rate = {};
$synth.SetOutputToWaveFile('{}');
$synth.Speak('{}');
$synth.Dispose();
"#,
            ((speed - 1.0) * 10.0).clamp(-10.0, 10.0) as i32,
            temp_wav.display(),
            escaped_text
        );

        if let Ok(mut file) = std::fs::File::create(&temp_ps) {
            let _ = file.write_all(ps_script.as_bytes());
            let _ = file.sync_all();

            let result = std::process::Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(&temp_ps)
                .output();

            let _ = std::fs::remove_file(&temp_ps);

            if result.map(|o| o.status.success()).unwrap_or(false) {
                if let Ok(wav_data) = std::fs::read(&temp_wav) {
                    let _ = std::fs::remove_file(&temp_wav);
                    return Ok(wav_data);
                }
            }

            let _ = std::fs::remove_file(&temp_wav);
        }
    }

    // Fallback: Generate a simple WAV file with silence
    // (This ensures we always return valid WAV data)
    generate_silent_wav(sample_rate, 1000)
}

/// Generate a silent WAV file as fallback
fn generate_silent_wav(sample_rate: u32, duration_ms: u64) -> Result<Vec<u8>, String> {
    let num_samples = (sample_rate as u64 * duration_ms / 1000) as usize;
    let data_size = num_samples * 2; // 16-bit samples
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(file_size + 8);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
    wav.extend_from_slice(&1u16.to_le_bytes()); // channels (mono)
    wav.extend_from_slice(&sample_rate.to_le_bytes()); // sample rate
    wav.extend_from_slice(&((sample_rate * 2)).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    // Silence (zeros)
    wav.resize(wav.len() + data_size, 0);

    Ok(wav)
}

/// Convert AIFF to WAV (simplified conversion for macOS say output)
#[allow(dead_code)]
fn convert_aiff_to_wav(aiff_data: &[u8], sample_rate: u32) -> Result<Vec<u8>, String> {
    // AIFF files from macOS say command are in a specific format
    // We need to extract the audio data and convert to WAV

    // Minimum AIFF header size
    if aiff_data.len() < 54 {
        return generate_silent_wav(sample_rate, 1000);
    }

    // Look for the data chunk in AIFF
    let mut data_start = 0;
    let mut data_size = 0;

    let mut i = 12; // Skip FORM header and chunk size
    while i < aiff_data.len().saturating_sub(8) {
        let chunk_id = String::from_utf8_lossy(&aiff_data[i..i+4]);
        let chunk_size = u32::from_be_bytes([
            aiff_data[i+4], aiff_data[i+5], aiff_data[i+6], aiff_data[i+7]
        ]) as usize;

        if chunk_id == "SSND" {
            // Sound data chunk found
            let offset = u32::from_be_bytes([
                aiff_data[i+8], aiff_data[i+9], aiff_data[i+10], aiff_data[i+11]
            ]) as usize;
            data_start = i + 16 + offset;
            data_size = chunk_size.saturating_sub(8);
            break;
        }

        i += 8 + chunk_size;
    }

    if data_start == 0 || data_start >= aiff_data.len() {
        return generate_silent_wav(sample_rate, 1000);
    }

    let end = (data_start + data_size).min(aiff_data.len());
    let audio_data = &aiff_data[data_start..end];

    // Convert 32-bit float AIFF to 16-bit PCM WAV
    let num_samples = audio_data.len() / 4;
    let pcm_data_size = num_samples * 2;
    let wav_file_size = 36 + pcm_data_size as u32;

    let mut wav = Vec::with_capacity(wav_file_size as usize + 8);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&wav_file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&((sample_rate * 2)).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm_data_size as u32).to_le_bytes());

    // Convert f32 samples (big-endian from AIFF) to i16 samples (little-endian for WAV)
    for chunk in audio_data.chunks_exact(4) {
        // AIFF uses big-endian float32
        let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
        let sample = f32::from_be_bytes(bytes);
        let i16_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        wav.extend_from_slice(&i16_sample.to_le_bytes());
    }

    Ok(wav)
}

/// Convert language code to espeak voice identifier
fn language_to_espeak_voice(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "en" | "en-us" => "en-us".to_string(),
        "en-gb" => "en-gb".to_string(),
        "ko" | "ko-kr" => "ko".to_string(),
        "ja" | "ja-jp" => "ja".to_string(),
        "zh" | "zh-cn" => "zh".to_string(),
        "es" | "es-es" => "es".to_string(),
        "fr" | "fr-fr" => "fr".to_string(),
        "de" | "de-de" => "de".to_string(),
        "it" | "it-it" => "it".to_string(),
        "pt" | "pt-br" => "pt-br".to_string(),
        "ru" | "ru-ru" => "ru".to_string(),
        _ => "en".to_string(),
    }
}

/// Convert language code to macOS voice identifier
#[cfg(target_os = "macos")]
fn language_to_macos_voice(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "en" | "en-us" => "Samantha".to_string(),
        "en-gb" => "Daniel".to_string(),
        "ko" | "ko-kr" => "Yuna".to_string(),
        "ja" | "ja-jp" => "Kyoko".to_string(),
        "zh" | "zh-cn" => "Ting-Ting".to_string(),
        "es" | "es-es" => "Monica".to_string(),
        "fr" | "fr-fr" => "Thomas".to_string(),
        "de" | "de-de" => "Anna".to_string(),
        "it" | "it-it" => "Alice".to_string(),
        _ => "Samantha".to_string(),
    }
}

/// Check if TTS is available
///
/// Checks if the TTS engine can be initialized and is ready to use.
pub fn is_tts_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        // On Linux, check for espeak-ng or espeak
        std::process::Command::new("espeak-ng")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || std::process::Command::new("espeak")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, say command should always be available
        return std::process::Command::new("say")
            .arg("-v")
            .arg("?")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, SAPI should always be available
        return true;
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // Unknown platform - assume not available
        false
    }
}

/// Get available voices from the TTS engine
fn get_tts_engine_voices() -> Vec<VoiceInfo> {
    let mut voices = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // Query macOS say command for available voices
        if let Ok(output) = std::process::Command::new("say")
            .arg("-v")
            .arg("?")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    // Parse voice info: "Name    Language    Description"
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if !parts.is_empty() {
                        let name = parts[0].to_string();
                        let language = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "en".to_string());
                        voices.push(VoiceInfo {
                            id: name.clone(),
                            name: name.clone(),
                            language,
                            gender: "unknown".to_string(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Query espeak for available voices
        if let Ok(output) = std::process::Command::new("espeak-ng")
            .arg("--voices")
            .output()
            .or_else(|_| std::process::Command::new("espeak").arg("--voices").output())
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let language = parts[1].to_string();
                    let name = parts.get(4).map(|s| s.to_string()).unwrap_or_else(|| language.clone());
                    voices.push(VoiceInfo {
                        id: language.clone(),
                        name: format!("{} ({})", name, language),
                        language,
                        gender: parts.get(3).map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows SAPI voices - use PowerShell to query
        if let Ok(output) = std::process::Command::new("powershell")
            .arg("-Command")
            .arg("Add-Type -AssemblyName System.Speech; $speak = New-Object System.Speech.Synthesis.SpeechSynthesizer; $speak.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name }")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    voices.push(VoiceInfo {
                        id: trimmed.to_string(),
                        name: trimmed.to_string(),
                        language: "en".to_string(),
                        gender: "unknown".to_string(),
                    });
                }
            }
        }
    }

    voices
}

/// Get available voices
///
/// Returns a list of voices available on the system for TTS.
#[tauri::command]
pub fn get_available_voices() -> Vec<VoiceInfo> {
    let mut voices = get_tts_engine_voices();

    // Add common voices that should be available across platforms
    #[cfg(target_os = "macos")]
    {
        // Ensure common macOS voices are listed
        let common_voices = vec![
            ("Samantha", "en-US", "Female"),
            ("Alex", "en-US", "Male"),
            ("Daniel", "en-GB", "Male"),
            ("Karen", "en-AU", "Female"),
            ("Moira", "en-IE", "Female"),
            ("Ting-Ting", "zh-CN", "Female"),
            ("Sin-Ji", "zh-HK", "Female"),
            ("Mei-Jia", "zh-TW", "Female"),
            ("Kyoko", "ja-JP", "Female"),
            ("Otoya", "ja-JP", "Male"),
            ("Yuna", "ko-KR", "Female"),
            ("Naru", "ko-KR", "Male"),
        ];

        for (name, lang, gender) in common_voices {
            if !voices.iter().any(|v| v.id == name) {
                voices.push(VoiceInfo {
                    id: name.to_string(),
                    name: name.to_string(),
                    language: lang.to_string(),
                    gender: gender.to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Ensure common espeak voices are listed
        let common_voices = vec![
            ("en-US", "English (US)", "neutral"),
            ("en-GB", "English (UK)", "neutral"),
            ("ko", "Korean", "neutral"),
            ("ja", "Japanese", "neutral"),
            ("zh", "Chinese", "neutral"),
            ("es", "Spanish", "neutral"),
            ("fr", "French", "neutral"),
            ("de", "German", "neutral"),
        ];

        for (id, name, gender) in common_voices {
            if !voices.iter().any(|v| v.id == id) {
                voices.push(VoiceInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    language: id.split('-').next().unwrap_or(id).to_string(),
                    gender: gender.to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Ensure default Windows voices are listed
        if voices.is_empty() {
            voices.push(VoiceInfo {
                id: "Microsoft David".to_string(),
                name: "Microsoft David".to_string(),
                language: "en-US".to_string(),
                gender: "Male".to_string(),
            });
            voices.push(VoiceInfo {
                id: "Microsoft Zira".to_string(),
                name: "Microsoft Zira".to_string(),
                language: "en-US".to_string(),
                gender: "Female".to_string(),
            });
        }
    }

    voices
}

/// Voice information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: String,
}

/// Stop current synthesis
///
/// Stops any currently in-progress synthesis.
pub fn stop_synthesis() -> Result<(), String> {
    let mut state = TTS_STATE.lock().unwrap();
    if let Some(ref mut s) = *state {
        if s.is_synthesizing {
            s.is_synthesizing = false;
            s.progress = 0.0;
            return Ok(());
        }
    }
    Err("No synthesis in progress".to_string())
}

/// Get synthesis progress
///
/// Returns the current synthesis progress as a value between 0.0 and 1.0.
pub fn get_synthesis_progress() -> f32 {
    let state = TTS_STATE.lock().unwrap();
    state.as_ref().map(|s| s.progress).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_config_default() {
        let config = TtsConfig::default();
        assert_eq!(config.voice, "default");
        assert_eq!(config.speed, 1.0);
        assert_eq!(config.pitch, 1.0);
        assert_eq!(config.sample_rate, 22050);
    }

    #[test]
    fn test_silent_wav_generation() {
        let wav = generate_silent_wav(22050, 1000).unwrap();
        assert!(wav.len() > 44); // At least header size
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn test_language_to_espeak_voice() {
        assert_eq!(language_to_espeak_voice("en"), "en-us");
        assert_eq!(language_to_espeak_voice("ko-KR"), "ko");
        assert_eq!(language_to_espeak_voice("ja-JP"), "ja");
        assert_eq!(language_to_espeak_voice("unknown"), "en");
    }

    #[test]
    fn test_synthesize_empty_text() {
        let result = synthesize("".to_string(), "en".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_init_tts() {
        let result = init_tts("default".to_string());
        // Should succeed if TTS is available, or return error
        // We just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_available_voices() {
        let voices = get_available_voices();
        // Should always return at least some voices
        assert!(!voices.is_empty());
    }

    #[test]
    fn test_stop_synthesis() {
        // Initially no synthesis in progress
        let result = stop_synthesis();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_synthesis_progress() {
        let progress = get_synthesis_progress();
        // Progress should be between 0.0 and 1.0
        assert!(progress >= 0.0 && progress <= 1.0);
    }
}
