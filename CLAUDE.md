# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AI Personal Assistant - Tauri v2 데스크톱 애플리케이션. 로컬 우선, MCP 기반, BYOK(Bring Your Own Key)를 지원하는 개인 비서 앱입니다.

## Commands

### Development
```bash
# 프론트엔드 개발 서버 + Tauri 실행
npm run tauri dev

# 프론트엔드만 개발 서버
npm run dev

# 빌드
npm run build
npm run tauri build
```

### Agent Runtime (Node.js Sidecar)
```bash
cd agent-runtime
npm run build    # TypeScript 컴파일
npm run dev      # 개발 모드 (tsx watch)
```

### Rust (Tauri Core)
```bash
cd src-tauri
cargo build      # 디버그 빌드
cargo build --release  # 릴리스 빌드
cargo test       # Rust 테스트
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Tauri Desktop App                  │
├─────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript + Vite)               │
│  - src/App.tsx, src/main.tsx                        │
│  - @tauri-apps/api로 Rust와 통신                     │
├─────────────────────────────────────────────────────┤
│  Rust Core (src-tauri/)                             │
│  - Tauri commands (lib.rs)                          │
│  - SQLite 영속성 (구현됨)                            │
│  - Sidecar 프로세스 관리                             │
├─────────────────────────────────────────────────────┤
│  Agent Runtime (agent-runtime/) - Node.js Sidecar   │
│  - JSON-RPC로 Rust와 통신 (stdio)                   │
│  - src/index.ts: 진입점, 요청 라우팅                │
│  - src/agent/core.ts: 에이전트 로직                 │
│  - src/providers/: LLM 제공자 (OpenAI, Anthropic, Ollama) │
│  - src/providers/router.ts: 다중 LLM 라우팅         │
│  - src/mcp/client.ts: MCP 서버 관리                 │
│  - src/memory/manager.ts: 컨텍스트 메모리 (계획됨)   │
└─────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **로컬 우선** - 모든 데이터는 사용자 머신에 저장
2. **MCP 우선** - Model Context Protocol로 도구 확장
3. **BYOK** - 사용자 API 키 사용, 로컬 모델(Ollama) 지원
4. **프라이버시** - 샌드박스 실행, 명시적 권한 관리

### Communication Flow

- Frontend → Rust: `@tauri-apps/api` invoke()
- Rust → Agent Runtime: Sidecar + JSON-RPC over stdio
- Agent Runtime → LLM: Provider 추상화
- Agent Runtime → Tools: MCP Client

## Key Files

| 파일 | 설명 |
|------|------|
| `src-tauri/src/lib.rs` | Tauri commands 정의 |
| `src-tauri/src/scheduler/` | Cron 작업 스케줄링 및 실행 |
| `src-tauri/src/db/mod.rs` | SQLite 데이터베이스 연산, Cron 작업 실행 |
| `agent-runtime/src/index.ts` | JSON-RPC 요청 처리, 초기화 |
| `agent-runtime/src/providers/base.ts` | Provider 인터페이스 (Message, ChatOptions, ChatResponse) |
| `agent-runtime/src/providers/router.ts` | 다중 LLM 제공자 라우팅 |
| `agent-runtime/src/mcp/` | MCP stdio 클라이언트 구현 (완료) |
| `agent-runtime/src/memory/` | 메모리 관리자, 영속성 (완료) |

## Provider Types

`BaseProvider`를 상속받아 구현:
- `openai` - OpenAI API
- `anthropic` - Claude API
- `ollama` - 로컬 LLM

## Tech Stack

| 레이어 | 기술 |
|--------|------|
| Desktop | Tauri v2 (Rust) |
| Frontend | React 19 + TypeScript + Vite |
| Agent Runtime | Node.js 22 (Sidecar) |
| Protocol | MCP (Model Context Protocol) |
| Local LLM | Ollama (Sidecar, 계획됨) |
| Database | SQLite (rusqlite) |

## Roadmap Reference

- **MVP (v0.1)** ✅: 기본 채팅, 폴더 권한, 파일 R/W, Ollama, 설정 UI, Agent Runtime 통합
- **v0.2** ✅: 스킬 시스템, 레시피 엔진, Browser MCP, 메모리 지속성
- **v0.3** 🔄: 서브에이전트, 다중 제공자 라우팅, **Cron 작업 ✅**, 마켓플레이스
- **v0.4** 🔄: 메모리 시스템 ✅, 음성 지원, 플러그인 시스템, 통합 기능, 협업 기능

## 최근 완료된 작업

### MCP 통신 완료 (2025-02-18)
- `agent-runtime/src/mcp/types.ts`: MCP 프로토콜 타입 정의 완료
- `agent-runtime/src/mcp/stdio.ts`: JSON-RPC over stdio 전송 계층 구현
- `agent-runtime/src/mcp/client.ts`: 다중 MCP 서버 관리 클라이언트 구현
- 15개 테스트 통과

### 메모리 영속화 (2025-02-18)
- `agent-runtime/src/memory/manager.ts`: 파일 기반 JSON 영속성 구현
- 장기 메모리 생성/검색/삭제 기능
- 자동 저장 (configurable interval)
- export/import 기능
- 26개 테스트 통과

### Cron 작업 실행 (2025-02-18)
- `src-tauri/src/scheduler/runner.rs`: JobExecutor 구현 (시스템 작업 실제 실행)
- `src-tauri/src/scheduler/scheduler.rs`: JobScheduler 구현 (주기적 작업 체크/실행)
- `src-tauri/src/db/mod.rs`: run_cron_job_now 실제 실행 로직 구현
- 지원 시스템 작업: 메시지 정리, DB vacuum, 설정 동기화
- 16개 테스트 통과
