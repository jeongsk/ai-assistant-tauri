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
- **v0.3** ✅: 서브에이전트, 다중 제공자 라우팅, **Cron 작업 ✅**, **Tauri 통합 ✅**, **Agent Runtime 연동 ✅**, **DB 영속성 ✅**
- **v0.4** ✅: 메모리 시스템 ✅, 음성 지원 ✅, 플러그인 시스템 ✅, 통합 기능 ✅, 협업 기능 ✅

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

### JobScheduler Tauri 통합 (2025-02-19)
- `src-tauri/src/lib.rs`: JobScheduler 상태를 Tauri app에 추가
- 앱 시작 시 스케줄러 자동 초기화 및 시작
- DB에서 활성화된 cron jobs 자동 로드
- Tauri commands: `scheduler_start`, `scheduler_stop`, `scheduler_status`, `scheduler_execute_job`, `scheduler_cancel_execution`
- `src-tauri/src/db/mod.rs`: `load_scheduled_jobs` 함수 추가

### Agent Runtime Job 실행 연동 (2025-02-19)
- `agent-runtime/src/index.ts`: `execute_skill`, `execute_recipe`, `execute_prompt` JSON-RPC 핸들러 추가
- `src-tauri/src/scheduler/runner.rs`: `AgentRuntimeClient` 구현 (Sidecar JSON-RPC 통신)
- Skill/Recipe/Prompt Job을 Agent Runtime을 통해 실제 실행
- 모든 16개 테스트 통과

### DB 실행 결과 영속성 (2025-02-19)
- `src-tauri/src/scheduler/runner.rs`: 작업 실행 결과 DB 자동 저장
- `create_execution_record`: 작업 시작 시 `job_executions` 테이블에 레코드 생성
- `save_execution_result`: 작업 완료 시 상태, 결과, 에러를 DB 업데이트
- `cleanup_completed`: 완료된 작업을 정리하며 DB에 결과 저장
- 모든 16개 테스트 통과

### v0.4 모듈 구현 상태 확인 (2025-02-19)
- **Voice Module** ✅: `voice/mod.rs`, `voice/stt.rs`, `voice/tts.rs` - 음성 인식/합성 타입 정의
- **Plugins Module** ✅:
  - `plugins/mod.rs`: Plugin 타입, 권한, 상태 정의
  - `plugins/loader.rs`: 플러그인 로드 및 검증
  - `plugins/sandbox.rs`: 샌드박스 실행 환경, 권한 체크
  - `plugins/api.rs`: 플러그인용 API 메서드 정의
- **Collaboration Module** ✅:
  - `collaboration/mod.rs`: Template, SharedWorkflow, ExportOptions 타입 정의
  - `collaboration/templates.rs`: TemplateManager, 기본 템플릿
  - `collaboration/export_mod.rs`: JSON/Markdown/HTML 내보내기 기능
