import { useState } from 'react';
import { Settings, MessageSquare, FolderOpen, Clock } from 'lucide-react';
import { ChatView } from './components/chat/ChatView';
import { SettingsDialog } from './components/settings/SettingsDialog';
import { useChatStore } from './stores/chatStore';
import './App.css';

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [activeView, setActiveView] = useState<'chat' | 'files' | 'history'>('chat');
  
  const conversations = useChatStore((state) => state.conversations);
  const createConversation = useChatStore((state) => state.createConversation);
  const setActiveConversation = useChatStore((state) => state.setActiveConversation);
  const activeConversationId = useChatStore((state) => state.activeConversationId);

  const handleNewChat = () => {
    createConversation();
  };

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-950">
      {/* Sidebar */}
      <aside className="w-64 bg-white dark:bg-gray-900 border-r flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b">
          <h1 className="text-xl font-bold">AI Assistant</h1>
          <p className="text-xs text-gray-500">Tauri + React</p>
        </div>

        {/* New Chat Button */}
        <div className="p-3">
          <button
            onClick={handleNewChat}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
          >
            <MessageSquare className="w-4 h-4" />
            New Chat
          </button>
        </div>

        {/* Conversation List */}
        <div className="flex-1 overflow-y-auto p-3">
          <h2 className="text-xs font-semibold text-gray-500 uppercase mb-2">Recent</h2>
          <div className="space-y-1">
            {conversations.map((conv) => (
              <button
                key={conv.id}
                onClick={() => setActiveConversation(conv.id)}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm truncate ${
                  conv.id === activeConversationId
                    ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
                    : 'hover:bg-gray-100 dark:hover:bg-gray-800'
                }`}
              >
                {conv.title}
              </button>
            ))}
            {conversations.length === 0 && (
              <p className="text-sm text-gray-400 px-3">No conversations yet</p>
            )}
          </div>
        </div>

        {/* Navigation */}
        <nav className="p-3 border-t space-y-1">
          <button
            onClick={() => setActiveView('files')}
            className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${
              activeView === 'files' ? 'bg-gray-100 dark:bg-gray-800' : 'hover:bg-gray-50 dark:hover:bg-gray-900'
            }`}
          >
            <FolderOpen className="w-4 h-4" />
            Files
          </button>
          <button
            onClick={() => setActiveView('history')}
            className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${
              activeView === 'history' ? 'bg-gray-100 dark:bg-gray-800' : 'hover:bg-gray-50 dark:hover:bg-gray-900'
            }`}
          >
            <Clock className="w-4 h-4" />
            History
          </button>
        </nav>

        {/* Settings Button */}
        <div className="p-3 border-t">
          <button
            onClick={() => setShowSettings(true)}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm hover:bg-gray-100 dark:hover:bg-gray-800"
          >
            <Settings className="w-4 h-4" />
            Settings
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {activeView === 'chat' && <ChatView />}
        {activeView === 'files' && <FilesView />}
        {activeView === 'history' && <HistoryView />}
      </main>

      {/* Settings Dialog */}
      <SettingsDialog isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  );
}

// Placeholder views
function FilesView() {
  return (
    <div className="flex-1 flex items-center justify-center text-gray-500">
      <div className="text-center">
        <FolderOpen className="w-12 h-12 mx-auto mb-4 opacity-50" />
        <h2 className="text-lg font-medium">File Explorer</h2>
        <p className="text-sm">Add folders in Settings to get started</p>
      </div>
    </div>
  );
}

function HistoryView() {
  return (
    <div className="flex-1 flex items-center justify-center text-gray-500">
      <div className="text-center">
        <Clock className="w-12 h-12 mx-auto mb-4 opacity-50" />
        <h2 className="text-lg font-medium">Task History</h2>
        <p className="text-sm">Your task history will appear here</p>
      </div>
    </div>
  );
}

export default App;
