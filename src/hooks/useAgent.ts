/**
 * Agent Hook
 * Manages communication with the agent runtime
 */

import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../stores/chatStore";
import { useSettingsStore } from "../stores/settingsStore";

export interface AgentMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

interface ChatResponse {
  content: string;
  error?: string;
}

export function useAgent() {
  const [initialized, setInitialized] = useState(false);

  const activeConversationId = useChatStore(
    (state) => state.activeConversationId,
  );
  const addMessage = useChatStore((state) => state.addMessage);
  const setStreaming = useChatStore((state) => state.setStreaming);

  const providers = useSettingsStore((state) => state.providers);
  const activeProvider = useSettingsStore((state) => state.activeProvider);
  const syncProvidersToAgent = useSettingsStore(
    (state) => state.syncProvidersToAgent,
  );

  // Initialize agent on mount
  useEffect(() => {
    if (!initialized) {
      invoke("init_agent")
        .then(async () => {
          console.log("Agent initialized");
          // Sync providers to agent runtime after initialization
          await syncProvidersToAgent();
          setInitialized(true);
        })
        .catch((err) => {
          console.warn("Agent initialization failed (expected in dev):", err);
          // Don't block - we'll use fallback
          setInitialized(true);
        });
    }
  }, [initialized, syncProvidersToAgent]);

  const sendMessage = useCallback(
    async (content: string) => {
      if (!activeConversationId) return;

      // Add user message
      addMessage(activeConversationId, {
        role: "user",
        content,
      });

      setStreaming(true);

      try {
        const providerConfig = providers[activeProvider];

        // Try to use agent runtime via Tauri
        try {
          const response: ChatResponse = await invoke("agent_chat", {
            messages: [{ role: "user", content }],
            provider: activeProvider,
          });

          if (response.error) {
            throw new Error(response.error);
          }

          addMessage(activeConversationId, {
            role: "assistant",
            content: response.content,
          });
        } catch (tauriError) {
          // Fallback: simulate response for development
          console.warn("Tauri call failed, using fallback:", tauriError);

          await new Promise((resolve) => setTimeout(resolve, 800));

          addMessage(activeConversationId, {
            role: "assistant",
            content: generateFallbackResponse(
              content,
              activeProvider,
              providerConfig,
            ),
          });
        }
      } catch (error) {
        addMessage(activeConversationId, {
          role: "assistant",
          content: `Error: ${error instanceof Error ? error.message : "Unknown error"}`,
        });
      } finally {
        setStreaming(false);
      }
    },
    [activeConversationId, addMessage, setStreaming, providers, activeProvider],
  );

  return { sendMessage, initialized };
}

// Fallback response generator for development
function generateFallbackResponse(
  userMessage: string,
  provider: string,
  config: any,
): string {
  const model = config?.model || "unknown";

  return `🔧 **Development Mode**

The agent runtime is not connected. To enable full functionality:

1. Build the agent-runtime: \`cd agent-runtime && npm run build\`
2. Start Tauri in dev mode: \`npm run tauri dev\`

**Your message:** "${userMessage.slice(0, 100)}${userMessage.length > 100 ? "..." : ""}"

**Selected provider:** ${provider}
**Model:** ${model}

In production, this would be a real AI response!`;
}
