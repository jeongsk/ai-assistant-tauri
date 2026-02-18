/**
 * Chat Store - Zustand state management with SQLite persistence
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  isStreaming?: boolean;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  updatedAt: Date;
}

interface ChatState {
  // State
  conversations: Conversation[];
  activeConversationId: string | null;
  isStreaming: boolean;
  isLoaded: boolean;

  // Actions
  loadConversations: () => Promise<void>;
  createConversation: () => string;
  setActiveConversation: (id: string) => void;
  addMessage: (conversationId: string, message: Omit<Message, 'id' | 'timestamp'>) => string;
  updateMessage: (conversationId: string, messageId: string, content: string) => void;
  setStreaming: (streaming: boolean) => void;
  deleteConversation: (id: string) => void;
  clearMessages: (conversationId: string) => void;
}

// Load messages for a conversation from DB
async function loadMessagesFromDB(conversationId: string): Promise<Message[]> {
  try {
    const messages = await invoke<Array<{
      id: string;
      conversation_id: string;
      role: string;
      content: string;
      metadata: string | null;
      created_at: string;
    }>>('load_messages', { conversationId });

    return messages.map((msg) => ({
      id: msg.id,
      role: msg.role as 'user' | 'assistant' | 'system',
      content: msg.content,
      timestamp: new Date(msg.created_at),
    }));
  } catch (error) {
    console.error('Failed to load messages:', error);
    return [];
  }
}

export const useChatStore = create<ChatState>((set, get) => ({
  // Initial state
  conversations: [],
  activeConversationId: null,
  isStreaming: false,
  isLoaded: false,

  // Load conversations from DB
  loadConversations: async () => {
    try {
      const conversations = await invoke<Array<{
        id: string;
        title: string;
        created_at: string;
        updated_at: string;
      }>>('load_conversations');

      const loadedConversations: Conversation[] = await Promise.all(
        conversations.map(async (conv) => {
          const messages = await loadMessagesFromDB(conv.id);
          return {
            id: conv.id,
            title: conv.title,
            messages,
            createdAt: new Date(conv.created_at),
            updatedAt: new Date(conv.updated_at),
          };
        })
      );

      set({
        conversations: loadedConversations,
        isLoaded: true,
        activeConversationId: loadedConversations[0]?.id || null,
      });
    } catch (error) {
      console.error('Failed to load conversations:', error);
      set({ isLoaded: true });
    }
  },

  // Create new conversation
  createConversation: () => {
    const id = crypto.randomUUID();
    const conversation: Conversation = {
      id,
      title: 'New Chat',
      messages: [],
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // Save to DB
    invoke('save_conversation', { id, title: conversation.title })
      .catch((error) => console.error('Failed to save conversation:', error));

    set((state) => ({
      conversations: [...state.conversations, conversation],
      activeConversationId: id,
    }));

    return id;
  },

  // Set active conversation
  setActiveConversation: (id) => {
    set({ activeConversationId: id });
  },

  // Add message to conversation
  addMessage: (conversationId, message) => {
    const id = crypto.randomUUID();
    const timestamp = new Date();
    const fullMessage: Message = {
      ...message,
      id,
      timestamp,
    };

    // Save to DB
    invoke('save_message', {
      id,
      conversationId,
      role: message.role,
      content: message.content,
      metadata: null,
    }).catch((error) => console.error('Failed to save message:', error));

    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? {
              ...conv,
              messages: [...conv.messages, fullMessage],
              updatedAt: new Date(),
              title:
                conv.messages.length === 0 && message.role === 'user'
                  ? message.content.slice(0, 50) +
                    (message.content.length > 50 ? '...' : '')
                  : conv.title,
            }
          : conv
      ),
    }));

    // Update conversation title in DB if first message
    const conversation = get().conversations.find((c) => c.id === conversationId);
    if (conversation && conversation.messages.length === 0 && message.role === 'user') {
      const newTitle =
        message.content.slice(0, 50) + (message.content.length > 50 ? '...' : '');
      invoke('save_conversation', { id: conversationId, title: newTitle }).catch(
        (error) => console.error('Failed to update conversation title:', error)
      );
    }

    return id;
  },

  // Update message content (for streaming)
  updateMessage: (conversationId, messageId, content) => {
    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? {
              ...conv,
              messages: conv.messages.map((msg) =>
                msg.id === messageId ? { ...msg, content } : msg
              ),
            }
          : conv
      ),
    }));
  },

  // Set streaming state
  setStreaming: (streaming) => {
    set({ isStreaming: streaming });
  },

  // Delete conversation
  deleteConversation: (id) => {
    // Delete from DB
    invoke('delete_conversation', { id }).catch((error) =>
      console.error('Failed to delete conversation:', error)
    );

    set((state) => ({
      conversations: state.conversations.filter((conv) => conv.id !== id),
      activeConversationId:
        state.activeConversationId === id
          ? state.conversations.find((c) => c.id !== id)?.id || null
          : state.activeConversationId,
    }));
  },

  // Clear messages in conversation
  clearMessages: (conversationId) => {
    // Note: This clears in-memory only. For full DB clear, we'd need a new command.
    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? { ...conv, messages: [], updatedAt: new Date() }
          : conv
      ),
    }));
  },
}));

// Selector hooks
export const useActiveConversation = () => {
  const conversations = useChatStore((state) => state.conversations);
  const activeId = useChatStore((state) => state.activeConversationId);
  return conversations.find((c) => c.id === activeId);
};

export const useMessages = () => {
  const conversation = useActiveConversation();
  return conversation?.messages || [];
};
