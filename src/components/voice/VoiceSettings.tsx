/**
 * Voice Settings Component - Configure voice settings
 */

import React from "react";
import { useVoiceStore } from "../../stores/voiceStore";

export function VoiceSettings() {
  const { settings, updateSettings } = useVoiceStore();

  const handleToggle = () => {
    updateSettings({ enabled: !settings.enabled });
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500 mb-4">
        Configure voice input and output settings.
      </p>

      {/* Enable Voice */}
      <div className="flex items-center justify-between p-3 border rounded-lg">
        <div>
          <p className="font-medium">Enable Voice</p>
          <p className="text-sm text-gray-500">Allow voice input and output</p>
        </div>
        <button
          onClick={handleToggle}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.enabled ? "bg-blue-500" : "bg-gray-300"
          }`}
        >
          <div
            className={`w-5 h-5 bg-white rounded-full shadow transform transition-transform ${
              settings.enabled ? "translate-x-6" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>

      {/* STT Model */}
      <div className="p-3 border rounded-lg space-y-2">
        <label className="block text-sm font-medium">Speech-to-Text Model</label>
        <select
          value={settings.sttModel}
          onChange={(e) => updateSettings({ sttModel: e.target.value })}
          disabled={!settings.enabled}
          className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        >
          <option value="tiny">Tiny (Fast)</option>
          <option value="base">Base (Recommended)</option>
          <option value="small">Small (Better)</option>
          <option value="medium">Medium (Best)</option>
          <option value="large">Large (Highest Quality)</option>
        </select>
      </div>

      {/* Language */}
      <div className="p-3 border rounded-lg space-y-2">
        <label className="block text-sm font-medium">Language</label>
        <select
          value={settings.language}
          onChange={(e) => updateSettings({ language: e.target.value })}
          disabled={!settings.enabled}
          className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        >
          <option value="en">English</option>
          <option value="ko">Korean</option>
          <option value="ja">Japanese</option>
          <option value="zh">Chinese</option>
          <option value="es">Spanish</option>
          <option value="fr">French</option>
          <option value="de">German</option>
        </select>
      </div>

      {/* TTS Voice */}
      <div className="p-3 border rounded-lg space-y-2">
        <label className="block text-sm font-medium">Text-to-Speech Voice</label>
        <select
          value={settings.ttsVoice}
          onChange={(e) => updateSettings({ ttsVoice: e.target.value })}
          disabled={!settings.enabled}
          className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        >
          <option value="default">Default</option>
        </select>
      </div>

      {/* Wake Word */}
      <div className="p-3 border rounded-lg space-y-2">
        <label className="block text-sm font-medium">Wake Word (Optional)</label>
        <input
          type="text"
          value={settings.wakeWord || ""}
          onChange={(e) => updateSettings({ wakeWord: e.target.value || undefined })}
          placeholder="e.g., 'Hey Assistant'"
          disabled={!settings.enabled}
          className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        />
        <p className="text-xs text-gray-400">
          Voice commands will only activate after this phrase
        </p>
      </div>

      {/* VAD Sensitivity */}
      <div className="p-3 border rounded-lg space-y-2">
        <label className="block text-sm font-medium">
          Voice Activity Detection
        </label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.1"
          value={settings.vadSensitivity}
          onChange={(e) => updateSettings({ vadSensitivity: parseFloat(e.target.value) })}
          disabled={!settings.enabled}
          className="w-full"
        />
        <div className="flex justify-between text-xs text-gray-400">
          <span>Low</span>
          <span>High</span>
        </div>
      </div>
    </div>
  );
}
