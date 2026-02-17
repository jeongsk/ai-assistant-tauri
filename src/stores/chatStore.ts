/**
 * Chat Store - Zustand state management
 */

import { create } from 'zustand';

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
  
  // Actions
  createConversation: () => string;
  setActiveConversation: (id: string) => void;
  addMessage: (conversationId: string, message: Omit<Message, 'id' | 'timestamp'>) => string;
  updateMessage: (conversationId: string, messageId: string, content: string) => void;
  setStreaming: (streaming: boolean) => void;
  deleteConversation: (id: string) => void;
  clearMessages: (conversationId: string) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  // Initial state
  conversations: [],
  activeConversationId: null,
  isStreaming: false,

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
    const fullMessage: Message = {
      ...message,
      id,
      timestamp: new Date(),
    };

    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? {
              ...conv,
              messages: [...conv.messages, fullMessage],
              updatedAt: new Date(),
              title: conv.messages.length === 0 && message.role === 'user'
                ? message.content.slice(0, 50) + (message.content.length > 50 ? '...' : '')
                : conv.title,
            }
          : conv
      ),
    }));

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
    set((state) => ({
      conversations: state.conversations.filter((conv) => conv.id !== id),
      activeConversationId:
        state.activeConversationId === id
          ? state.conversations[0]?.id || null
          : state.activeConversationId,
    }));
  },

  // Clear messages in conversation
  clearMessages: (conversationId) => {
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
