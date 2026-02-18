/**
 * Voice Button Component - Push-to-talk button
 */

import React, { useEffect, useRef } from "react";
import { Mic, MicOff, Square } from "lucide-react";
import { useVoiceStore } from "../../stores/voiceStore";

interface VoiceButtonProps {
  onTranscript?: (transcript: string) => void;
  size?: "sm" | "md" | "lg";
}

export function VoiceButton({ onTranscript, size = "md" }: VoiceButtonProps) {
  const { settings, isListening, startListening, stopListening, addCommand } = useVoiceStore();
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);

  const sizeClasses = {
    sm: "w-8 h-8",
    md: "w-12 h-12",
    lg: "w-16 h-16",
  };

  const iconSizes = {
    sm: "w-4 h-4",
    md: "w-6 h-6",
    lg: "w-8 h-8",
  };

  const handleMouseDown = async () => {
    if (!settings.enabled) return;

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorderRef.current = new MediaRecorder(stream);

      const chunks: Blob[] = [];
      mediaRecorderRef.current.ondataavailable = (e) => chunks.push(e.data);

      mediaRecorderRef.current.onstop = async () => {
        const blob = new Blob(chunks, { type: "audio/webm" });
        // In production, send to STT service
        console.log("Audio captured:", blob.size, "bytes");

        // Simulate transcription for demo
        const command = {
          id: `cmd-${Date.now()}`,
          transcript: "Voice input received",
          confidence: 0.9,
          timestamp: new Date().toISOString(),
        };

        addCommand(command);
        onTranscript?.(command.transcript);

        stream.getTracks().forEach((track) => track.stop());
      };

      mediaRecorderRef.current.start();
      startListening();
    } catch (error) {
      console.error("Failed to start recording:", error);
    }
  };

  const handleMouseUp = () => {
    if (mediaRecorderRef.current && isListening) {
      mediaRecorderRef.current.stop();
      stopListening();
    }
  };

  if (!settings.enabled) {
    return (
      <button
        disabled
        className={`${sizeClasses[size]} rounded-full bg-gray-200 dark:bg-gray-700 flex items-center justify-center cursor-not-allowed`}
        title="Voice is disabled"
      >
        <MicOff className={`${iconSizes[size]} text-gray-400`} />
      </button>
    );
  }

  return (
    <button
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      className={`${sizeClasses[size]} rounded-full ${
        isListening
          ? "bg-red-500 animate-pulse"
          : "bg-blue-500 hover:bg-blue-600"
      } flex items-center justify-center transition-colors`}
      title={isListening ? "Recording... Release to stop" : "Hold to speak"}
    >
      {isListening ? (
        <Square className={`${iconSizes[size]} text-white`} />
      ) : (
        <Mic className={`${iconSizes[size]} text-white`} />
      )}
    </button>
  );
}
