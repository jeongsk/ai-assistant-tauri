import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import App from "./App";

// Mock Tauri APIs
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

// Mock stores
vi.mock("./stores/chatStore", () => ({
  useChatStore: vi.fn((selector) => {
    const state = {
      loadConversations: vi.fn(),
      isLoaded: true,
    };
    return selector(state);
  }),
}));

vi.mock("./stores/settingsStore", () => ({
  useSettingsStore: vi.fn((selector) => {
    const state = {
      loadFolderPermissions: vi.fn(),
      folderPermissionsLoaded: true,
    };
    return selector(state);
  }),
}));

// Mock hooks
vi.mock("./hooks/useAgent", () => ({
  useAgent: () => ({
    initialized: true,
    loading: false,
    error: null,
  }),
}));

// Mock components that use browser APIs or complex state
vi.mock("./components/chat/ChatView", () => ({
  ChatView: () => <div data-testid="chat-view">Chat View</div>,
}));

vi.mock("./components/chat/ConversationList", () => ({
  ConversationList: () => (
    <div data-testid="conversation-list">Conversations</div>
  ),
}));

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the application", () => {
    render(<App />);
    expect(screen.getByText("AI Assistant")).toBeInTheDocument();
  });

  it("shows the subtitle", () => {
    render(<App />);
    expect(screen.getByText("Tauri + React")).toBeInTheDocument();
  });

  it("renders navigation items", () => {
    render(<App />);
    expect(screen.getByText("Files")).toBeInTheDocument();
    expect(screen.getByText("History")).toBeInTheDocument();
  });

  it("renders settings button", () => {
    render(<App />);
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });
});
