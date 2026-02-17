# AI Personal Assistant

A privacy-first AI assistant desktop app built with Tauri, React, and MCP.

## Features

- 🤖 Multi-provider support (OpenAI, Anthropic, Ollama)
- 🔒 Local-first with folder-based permissions
- 📁 File operations (read, write, organize)
- 🎨 Modern UI with dark mode
- 🔧 Extensible via MCP protocol

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri v2 |
| Frontend | React 18 + TypeScript |
| State Management | Zustand |
| Styling | Tailwind CSS |
| Agent Runtime | Node.js (Sidecar) |
| Local LLM | Ollama |

## Getting Started

### Prerequisites

- Node.js 22+
- Rust (via rustup)
- System dependencies (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

### Development

```bash
# Install dependencies
npm install

# Install agent-runtime dependencies
cd agent-runtime && npm install && cd ..

# Run in development mode
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Project Structure

```
ai-assistant-tauri/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # Custom hooks
│   ├── stores/             # Zustand stores
│   └── services/           # Tauri API wrappers
├── src-tauri/              # Rust backend
│   └── src/
│       └── lib.rs          # Tauri commands
└── agent-runtime/          # Node.js sidecar
    └── src/
        ├── agent/          # Agent logic
        ├── providers/      # LLM providers
        ├── mcp/            # MCP client
        └── memory/         # Memory management
```

## Roadmap

### MVP (v0.1)
- [x] Project setup
- [x] Provider implementations
- [x] Basic UI components
- [x] Rust commands
- [ ] Agent Runtime integration
- [ ] Full E2E functionality

### v0.2
- [ ] Skills system
- [ ] Recipe engine
- [ ] Browser automation
- [ ] Memory persistence

## License

MIT

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) 
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
