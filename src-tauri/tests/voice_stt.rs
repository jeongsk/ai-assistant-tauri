// Integration tests for Whisper STT functionality
//
// Note: These tests require actual Whisper model files to run.
// Tests are marked with #[ignore] by default and can be run with:
// cargo test --package ai-assistant-tauri --test voice_stt -- --ignored

#[cfg(feature = "voice")]
mod tests {
    // Import from the main library
    use ai_assistant_tauri_lib::voice::stt::{
        parse_wav_header, extract_pcm_data,
        resample_audio, apply_vad, get_model_download_url,
        SttConfig,
    };

    #[test]
    fn test_audio_resampling() {
        // Test resampling from 16kHz to 8kHz
        let input = vec![1.0f32; 16000]; // 1 second at 16kHz
        let resampled = resample_audio(&input, 16000, 8000);
        assert_eq!(resampled.len(), 8000); // Should be 1 second at 8kHz
    }

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
    fn test_vad_with_silence() {
        let silence = vec![0.0; 1000];
        let vad_result = apply_vad(&silence, 0.5);
        assert!(!vad_result.is_speech);
        assert!(vad_result.energy_db < -30.0);
    }

    #[test]
    fn test_vad_with_loud_audio() {
        let loud = vec![0.5; 1000];
        let vad_result = apply_vad(&loud, 0.5);
        assert!(vad_result.is_speech);
        assert!(vad_result.confidence > 0.0);
    }

    #[test]
    fn test_stt_config_default() {
        let config = SttConfig::default();
        assert_eq!(config.model, "base");
        assert_eq!(config.task, "transcribe");
        assert_eq!(config.threads, 4);
        assert_eq!(config.vad_threshold, 0.5);
    }

    #[test]
    fn test_model_download_url() {
        let url = get_model_download_url("base".to_string()).unwrap();
        assert!(url.contains("ggml-base.bin"));
        assert!(url.contains("huggingface.co"));
    }

    #[test]
    fn test_invalid_model_name() {
        let result = get_model_download_url("invalid-model".to_string());
        assert!(result.is_err());
    }

    // The following tests require actual Whisper model files
    // Run with: cargo test --package ai-assistant-tauri --test voice_stt -- --ignored

    #[test]
    #[ignore = "Requires actual Whisper model and audio file"]
    fn test_full_transcription() {
        // This test requires both a model and test audio
        // Skip if not available
        let model_path = std::path::Path::new("./models/whisper/ggml-tiny.bin");
        let audio_path = std::path::Path::new("./test_data/speech_16k.wav");

        if !model_path.exists() || !audio_path.exists() {
            return;
        }

        // Load model using SttEngine
        let config = SttConfig {
            model: "tiny".to_string(),
            ..Default::default()
        };

        let engine = ai_assistant_tauri_lib::voice::stt::SttEngine::new(config);
        assert!(engine.is_ok());

        // Load audio and parse
        let audio_data = std::fs::read(audio_path).unwrap();
        let params = parse_wav_header(&audio_data).unwrap();
        let samples = extract_pcm_data(&audio_data, &params).unwrap();

        // Should have samples
        assert!(!samples.is_empty());
    }
}
