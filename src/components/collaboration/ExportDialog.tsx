/**
 * Export Dialog Component - Export conversations
 */

import React, { useState } from "react";
import { Download, X, FileJson, FileText, Code } from "lucide-react";
import { useCollaborationStore } from "../../stores/collaborationStore";
import { ExportOptions, ConversationExport } from "../../types/collaboration";

interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  conversations: ConversationExport[];
}

export function ExportDialog({ isOpen, onClose, conversations }: ExportDialogProps) {
  const { exportConversations } = useCollaborationStore();
  const [options, setOptions] = useState<ExportOptions>({
    format: "json",
    includeMetadata: true,
    includeTimestamps: true,
    prettyPrint: true,
  });

  const handleExport = async () => {
    const blob = await exportConversations(conversations, options);

    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `conversations-export.${options.format}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    onClose();
  };

  if (!isOpen) return null;

  const formatOptions = [
    { value: "json", label: "JSON", icon: FileJson },
    { value: "markdown", label: "Markdown", icon: FileText },
    { value: "html", label: "HTML", icon: Code },
  ] as const;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-lg shadow-xl w-full max-w-md">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <h2 className="text-lg font-semibold">Export Conversations</h2>
          <button onClick={onClose} className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <p className="text-sm text-gray-500">
            Export {conversations.length} conversation(s)
          </p>

          {/* Format Selection */}
          <div>
            <label className="block text-sm font-medium mb-2">Format</label>
            <div className="grid grid-cols-3 gap-2">
              {formatOptions.map(({ value, label, icon: Icon }) => (
                <button
                  key={value}
                  onClick={() => setOptions({ ...options, format: value })}
                  className={`flex flex-col items-center gap-1 p-3 border rounded-lg ${
                    options.format === value
                      ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                      : "hover:bg-gray-50 dark:hover:bg-gray-800"
                  }`}
                >
                  <Icon className="w-5 h-5" />
                  <span className="text-sm">{label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Options */}
          <div className="space-y-2">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={options.includeMetadata}
                onChange={(e) => setOptions({ ...options, includeMetadata: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm">Include metadata</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={options.includeTimestamps}
                onChange={(e) => setOptions({ ...options, includeTimestamps: e.target.checked })}
                className="w-4 h-4"
              />
              <span className="text-sm">Include timestamps</span>
            </label>
            {options.format === "json" && (
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={options.prettyPrint}
                  onChange={(e) => setOptions({ ...options, prettyPrint: e.target.checked })}
                  className="w-4 h-4"
                />
                <span className="text-sm">Pretty print</span>
              </label>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-6 py-4 border-t">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm border rounded hover:bg-gray-50 dark:hover:bg-gray-800"
          >
            Cancel
          </button>
          <button
            onClick={handleExport}
            className="flex items-center gap-2 px-4 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            <Download className="w-4 h-4" />
            Export
          </button>
        </div>
      </div>
    </div>
  );
}
