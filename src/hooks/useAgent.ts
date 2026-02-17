/**
 * Agent Hook
 * Manages communication with the agent runtime
 */

import { useCallback } from 'react';
import { useChatStore } from '../stores/chatStore';
import { useSettingsStore } from '../stores/settingsStore';

export interface AgentMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export function useAgent() {
  const activeConversationId = useChatStore((state) => state.activeConversationId);
  const addMessage = useChatStore((state) => state.addMessage);
  const updateMessage = useChatStore((state) => state.updateMessage);
  const setStreaming = useChatStore((state) => state.setStreaming);
  
  const providers = useSettingsStore((state) => state.providers);
  const activeProvider = useSettingsStore((state) => state.activeProvider);

  const sendMessage = useCallback(async (content: string) => {
    if (!activeConversationId) return;

    // Add user message
    addMessage(activeConversationId, {
      role: 'user',
      content,
    });

    setStreaming(true);

    try {
      const providerConfig = providers[activeProvider];
      
      // TODO: Call Agent Runtime via Tauri sidecar
      // For now, return a placeholder
      await simulateResponse(content, providerConfig);

      addMessage(activeConversationId, {
        role: 'assistant',
        content: `Response from ${activeProvider}:\n\nThis is a placeholder. Connect the Agent Runtime to get real responses.\n\nYour message was: "${content.slice(0, 100)}..."`,
      });
    } catch (error) {
      addMessage(activeConversationId, {
        role: 'assistant',
        content: `Error: ${error instanceof Error ? error.message : 'Unknown error'}`,
      });
    } finally {
      setStreaming(false);
    }
  }, [activeConversationId, addMessage, setStreaming, providers, activeProvider]);

  return { sendMessage };
}

// Simulate response for development
async function simulateResponse(content: string, config: any): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 500 + Math.random() * 1000));
}
