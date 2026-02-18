/**
 * Voice Type Definitions
 */

export interface VoiceSettings {
  id: string;
  enabled: boolean;
  sttModel: string;
  ttsVoice: string;
  language: string;
  wakeWord?: string;
  vadSensitivity: number;
  updatedAt: string;
}

export interface VoiceCommand {
  id: string;
  transcript: string;
  confidence: number;
  intent?: string;
  entities?: Record<string, unknown>;
  timestamp: string;
}

export interface TranscriptionResult {
  text: string;
  confidence: number;
  language: string;
  durationMs: number;
}

export interface SynthesisResult {
  audioData: number[];
  sampleRate: number;
  durationMs: number;
}

export const DEFAULT_VOICE_SETTINGS: Omit<VoiceSettings, 'id' | 'updatedAt'> = {
  enabled: false,
  sttModel: 'base',
  ttsVoice: 'default',
  language: 'en',
  vadSensitivity: 0.5,
};
