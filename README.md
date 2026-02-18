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
| Frontend | React 19 + TypeScript |
| State Management | Zustand |
| Styling | Tailwind CSS |
| Agent Runtime | Node.js 22 (Sidecar) |
| Local LLM | Ollama |
| Testing (Frontend) | Vitest + React Testing Library |
| Testing (Agent) | Jest + ts-jest |
| Testing (Rust) | cargo test |

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

### Testing

```bash
# Frontend tests (Vitest)
npm run test           # Watch mode
npm run test:run       # Run once
npm run test:coverage  # With coverage

# Agent Runtime tests (Jest)
cd agent-runtime && npm run test

# Rust tests (cargo)
cd src-tauri && cargo test
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

### MVP (v0.1) ✅
- [x] Project setup
- [x] Provider implementations
- [x] Basic UI components
- [x] Rust commands
- [x] Agent Runtime integration
- [x] Full E2E functionality
- [x] Test infrastructure (v0.1.1)

### v0.2 ✅
- [x] Skills system
- [x] Recipe engine
- [x] Browser automation
- [x] Memory persistence

## License

MIT

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) 
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
