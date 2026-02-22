## [0.6.0] - 2026-02-22

### Added
- **Agent Module**: Multimodal input processing, context management, sub-agent orchestration
- **Workflow Module**: Visual workflow editor foundation, trigger system, execution engine
- **Sync Module**: Cloud synchronization architecture, conflict resolution, offline queue
- **Database Migrations**: v8 (workflows), v9 (voice profiles), v10 (sync tables)

### Architecture
- `src-tauri/src/agent/` - AI agent enhancement (multimodal, context, orchestrator)
- `src-tauri/src/workflow/` - Workflow automation (store, engine, nodes, triggers)
- `src-tauri/src/sync/` - Cloud sync (manager, conflict, offline)

---

## [0.5.0] - 2026-02-22

### Added
- **WASM Plugin Runtime**: Wasmtime v22+ with WASI preview1 support
- **Integration Services**: PostgreSQL/MySQL query execution, Git operations, AWS S3
- **Template Sharing**: JSON import/export, versioning, team sharing
- **Voice Commands**: Voice command parsing (EN/KO), AgentRuntime routing

### Changed
- All 90 tests passing, build verification complete
- Security fixes: Path traversal prevention, non-UTF-8 path handling

---

## [0.4.0] - 2026-02-18

### Added
- Memory system for persistent context across conversations
- Voice input and output capabilities
- Plugin system for extending functionality
- External service integrations
- Real-time collaboration features

### Changed
- Added auto-claude entries to .gitignore for better project hygiene

---

## [0.3.0] - 2026-02-18

### Added
- Sub-agents support for specialized task handling
- Intelligent routing system for optimal task distribution
- Marketplace for discovering and installing extensions
- Scheduled tasks (Cron) for automated operations
- Browser MCP with rate limiting and settings UI

---

## [0.2.0] - 2026-02-18

### Added
- Skill system for defining agent capabilities
- Recipe system for reusable task workflows
- SQLite database for persistent data storage
- Ollama support for local LLM integration
- Provider sync for keeping LLM providers up to date
- Improved settings UI for better configuration
- TaskHistory component for tracking agent activities
- FileExplorer component for file navigation

### Changed
- Improved agent runtime JSON-RPC handling
- Enhanced sidecar management and UI

---

## [0.1.0] - 2026-02-18

### Added
- Tauri + React application foundation
- Agent runtime scaffold with Rust backend
- LLM providers integration and memory manager
- Frontend UI components library
- Tauri service layer with custom hooks
- Rust commands for file operations

### Fixed
- TypeScript types for API responses

### Documentation
- Added CLAUDE.md project guide

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
