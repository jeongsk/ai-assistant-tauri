/**
 * Voice Processor - Handles voice command processing
 */

export interface VoiceCommand {
  id: string;
  transcript: string;
  confidence: number;
  intent?: string;
  entities?: Record<string, any>;
  timestamp: string;
}

export interface VoiceProcessorConfig {
  language: string;
  wakeWord?: string;
  vadSensitivity: number;
  continuousListening: boolean;
}

export const DEFAULT_VOICE_CONFIG: VoiceProcessorConfig = {
  language: 'en',
  vadSensitivity: 0.5,
  continuousListening: false,
};

export class VoiceProcessor {
  private config: VoiceProcessorConfig;
  private isListening: boolean = false;
  private commandQueue: VoiceCommand[] = [];

  constructor(config?: Partial<VoiceProcessorConfig>) {
    this.config = { ...DEFAULT_VOICE_CONFIG, ...config };
  }

  startListening(): void {
    this.isListening = true;
  }

  stopListening(): void {
    this.isListening = false;
  }

  getListeningState(): boolean {
    return this.isListening;
  }

  processTranscript(transcript: string, confidence: number): VoiceCommand {
    const command: VoiceCommand = {
      id: `cmd-${Date.now()}`,
      transcript,
      confidence,
      intent: this.detectIntent(transcript),
      entities: this.extractEntities(transcript),
      timestamp: new Date().toISOString(),
    };

    this.commandQueue.push(command);
    return command;
  }

  private detectIntent(transcript: string): string {
    const lower = transcript.toLowerCase();

    if (lower.includes('search') || lower.includes('find')) {
      return 'search';
    }
    if (lower.includes('create') || lower.includes('make') || lower.includes('new')) {
      return 'create';
    }
    if (lower.includes('delete') || lower.includes('remove')) {
      return 'delete';
    }
    if (lower.includes('update') || lower.includes('change') || lower.includes('modify')) {
      return 'update';
    }
    if (lower.includes('read') || lower.includes('show') || lower.includes('display')) {
      return 'read';
    }
    if (lower.includes('help') || lower.includes('what can you do')) {
      return 'help';
    }
    if (lower.includes('stop') || lower.includes('cancel')) {
      return 'cancel';
    }

    return 'unknown';
  }

  private extractEntities(transcript: string): Record<string, any> {
    const entities: Record<string, any> = {};

    const pathRegex = /(?:^|\s)(\/[\w/.-]+)/g;
    const paths = transcript.match(pathRegex);
    if (paths) {
      entities.paths = paths.map((p: string) => p.trim());
    }

    const quoteRegex = /"([^"]+)"|'([^']+)'/g;
    const quotes: string[] = [];
    let match;
    while ((match = quoteRegex.exec(transcript)) !== null) {
      quotes.push(match[1] || match[2]);
    }
    if (quotes.length > 0) {
      entities.quoted = quotes;
    }

    const numberRegex = /\b(\d+(?:\.\d+)?)\b/g;
    const numbers = transcript.match(numberRegex);
    if (numbers) {
      entities.numbers = numbers.map((n: string) => parseFloat(n));
    }

    return entities;
  }

  getPendingCommands(): VoiceCommand[] {
    return [...this.commandQueue];
  }

  clearQueue(): void {
    this.commandQueue = [];
  }

  checkWakeWord(transcript: string): boolean {
    if (!this.config.wakeWord) return true;
    const lower = transcript.toLowerCase();
    const wakeWordLower = this.config.wakeWord.toLowerCase();
    return lower.includes(wakeWordLower);
  }

  commandToAction(command: VoiceCommand): {
    action: string;
    params: Record<string, any>;
  } | null {
    const { intent, entities, transcript } = command;

    switch (intent) {
      case 'search':
        return {
          action: 'search',
          params: {
            query: entities?.quoted?.[0] || transcript.replace(/search|find/gi, '').trim(),
          },
        };

      case 'create':
        return {
          action: 'create',
          params: {
            type: entities?.quoted?.[0] || 'unknown',
            content: transcript,
          },
        };

      case 'read':
        return {
          action: 'read',
          params: {
            path: entities?.paths?.[0] || entities?.quoted?.[0],
          },
        };

      case 'delete':
        return {
          action: 'delete',
          params: {
            target: entities?.quoted?.[0] || entities?.paths?.[0],
          },
        };

      case 'help':
        return { action: 'help', params: {} };

      case 'cancel':
        return { action: 'cancel', params: {} };

      default:
        return { action: 'chat', params: { message: transcript } };
    }
  }
}

let processorInstance: VoiceProcessor | null = null;

export function getVoiceProcessor(config?: Partial<VoiceProcessorConfig>): VoiceProcessor {
  if (!processorInstance) {
    processorInstance = new VoiceProcessor(config);
  }
  return processorInstance;
}
