# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-02-18

### Test Infrastructure

#### Added
- **Frontend Testing** - Vitest + React Testing Library setup
  - `vitest.config.ts` configuration with jsdom environment
  - `src/test/setup.ts` for testing-library/jest-dom
  - `src/App.test.tsx` with 4 passing tests
- **Agent Runtime Testing** - Jest + ts-jest setup
  - `jest.config.js` with ESM module support
  - `src/providers/base.test.ts` with 6 passing tests
- **Rust Backend Testing** - cargo test infrastructure
  - `tempfile` dev-dependency for test fixtures
  - 5 unit tests in `src/lib.rs` (greet, get_version, serialization tests)

#### Test Results
| Layer | Framework | Tests | Status |
|-------|-----------|-------|--------|
| Frontend | Vitest | 4 | ✅ Pass |
| Agent Runtime | Jest | 6 | ✅ Pass |
| Rust Backend | cargo test | 5 | ✅ Pass |

#### Scripts Added
- `npm run test` - Run tests in watch mode
- `npm run test:run` - Run tests once
- `npm run test:coverage` - Run tests with coverage report

---

## [0.1.0] - 2025-02-18

### MVP Release

#### Added
- **Multi-provider support** - OpenAI, Anthropic, Ollama LLM providers
- **Agent Runtime** - Node.js sidecar for LLM communication via JSON-RPC
- **SQLite persistence** - Conversations, messages, and settings stored locally
- **Folder permissions** - Read/read-write access control for directories
- **File operations** - Read, write, list directory contents
- **Modern UI** - React + Tailwind CSS with dark mode support
- **MCP protocol support** - Model Context Protocol for extensibility

#### Fixed
- Sidecar binary path resolution for both development and production builds
- Agent Runtime wrapper script to support multiple directory structures

#### Technical Details
- Tauri v2 desktop framework
- React 19 + TypeScript + Vite frontend
- Zustand state management
- Node.js 22 sidecar for agent runtime
- SQLite database via rusqlite

---

## Roadmap

### v0.2 (Completed)
- Skills system
- Recipe engine
- Browser automation (MCP)
- Memory persistence

### v0.3 (Planned)
- Sub-agents
- Provider routing
- Cron jobs
- Marketplace

### v0.4 (Planned)
- Memory system
- Voice support
- Plugin system
- Integrations
- Collaboration features
