import { useState } from 'react';
import { Settings, FolderOpen, Clock, Menu, X } from 'lucide-react';
import { ChatView } from './components/chat/ChatView';
import { ConversationList } from './components/chat/ConversationList';
import { SettingsDialog } from './components/settings/SettingsDialog';
import './App.css';

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [activeView, setActiveView] = useState<'chat' | 'files' | 'history'>('chat');
  const [sidebarOpen, setSidebarOpen] = useState(true);

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
      {/* Sidebar */}
      <aside
        className={`${
          sidebarOpen ? 'w-64' : 'w-0'
        } bg-white dark:bg-gray-900 border-r transition-all duration-300 overflow-hidden flex flex-col`}
      >
        {/* Logo */}
        <div className="p-4 border-b flex items-center justify-between">
          <div>
            <h1 className="text-lg font-bold">AI Assistant</h1>
            <p className="text-xs text-gray-500">Tauri + React</p>
          </div>
          <button
            onClick={() => setSidebarOpen(false)}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded lg:hidden"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Conversations */}
        <div className="flex-1 overflow-hidden">
          <ConversationList />
        </div>

        {/* Navigation */}
        <nav className="p-2 border-t space-y-0.5">
          <NavItem
            icon={<FolderOpen className="w-4 h-4" />}
            label="Files"
            active={activeView === 'files'}
            onClick={() => setActiveView('files')}
          />
          <NavItem
            icon={<Clock className="w-4 h-4" />}
            label="History"
            active={activeView === 'history'}
            onClick={() => setActiveView('history')}
          />
        </nav>

        {/* Settings */}
        <div className="p-2 border-t">
          <button
            onClick={() => setShowSettings(true)}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm hover:bg-gray-100 dark:hover:bg-gray-800"
          >
            <Settings className="w-4 h-4" />
            Settings
          </button>
        </div>
      </aside>

      {/* Toggle sidebar button (when closed) */}
      {!sidebarOpen && (
        <button
          onClick={() => setSidebarOpen(true)}
          className="absolute top-4 left-4 p-2 bg-white dark:bg-gray-800 border rounded-lg shadow-sm hover:bg-gray-50 z-10"
        >
          <Menu className="w-5 h-5" />
        </button>
      )}

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {activeView === 'chat' && <ChatView />}
        {activeView === 'files' && <FilesView />}
        {activeView === 'history' && <HistoryView />}
      </main>

      {/* Settings Dialog */}
      <SettingsDialog isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  );
}

interface NavItemProps {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}

function NavItem({ icon, label, active, onClick }: NavItemProps) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors ${
        active
          ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
          : 'hover:bg-gray-100 dark:hover:bg-gray-800'
      }`}
    >
      {icon}
      {label}
    </button>
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
