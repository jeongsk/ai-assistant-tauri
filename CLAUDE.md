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
| `src-tauri/src/agent/` | v0.6: 멀티모달, 컨텍스트 관리, 서브에이전트 오케스트레이션 |
| `src-tauri/src/workflow/` | v0.6: 워크플로우 저장소, 실행 엔진, 노드, 트리거 |
| `src-tauri/src/sync/` | v0.6: 동기화 관리자, 충돌 해결, 오프라인 큐 |
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
- **v0.3** ✅: 서브에이전트, 다중 제공자 라우팅, Cron 작업, Tauri 통합, Agent Runtime 연동, DB 영속성
- **v0.4** ✅: 메모리 시스템, 음성 지원, 플러그인 시스템, 통합 기능, 협업 기능
- **v0.5** ✅: WASM 플러그인 런타임, 통합 서비스 연동, 템플릿 공유, 음성 명령어, 테스트 완료
- **v0.6** 🚧: AI 에이전트 고도화, 워크플로우 자동화, 고급 음성, 클라우드 동기화 (기초 구조 완료)

---

# v0.5 Release Notes (2026-02-22)

## Overview
v0.5는 v0.4에서 구현된 기능들의 실제 동작을 구현하는 메이저 업데이트입니다.

## New Features

### 🔌 WASM Plugin Runtime (Phase 1)
- Wasmtime 기반 샌드박스 실행 환경
- WASI 호스트 구현
- 리소스 모니터링 (메모리, CPU, 실행 시간)

### 🔗 Integration Services (Phase 2)
- PostgreSQL/MySQL 실제 쿼리 실행
- Git commit/push/pull 연동
- AWS S3 업로드/다운로드

### 📄 Template Sharing (Phase 3)
- JSON import/export
- 템플릿 버전 관리
- 팀 공유 기능

### 🎤 Voice Commands (Phase 4)
- 음성 명령어 파싱 (영어/한국어)
- AgentRuntime 라우팅
- 멀티턴 음성 대화

### ✅ Testing (Phase 5)
- 81개 테스트 전체 통과
- 빌드 검증 완료
- 경고 정리 (46 → 11)

자세한 내용은 `docs/v0.5-release-notes.md` 참고.

---

# v0.4 Release Notes (2025-02-20)

## Overview
v0.4는 음성 인식, 플러그인 시스템, 외부 서비스 통합, 템플릿 관리 기능을 추가한 메이저 업데이트입니다.

## New Features

### 🎤 Voice Support
- **Speech-to-Text (STT)**: Whisper 모델 기반 음성 인식
  - 지원 모델: tiny, base, small, medium, large
  - 다국어 지원 (영어, 한국어, 일본어, 중국어, 스페인어, 프랑스어, 독일어)
  - VAD (Voice Activity Detection) 설정
- **Text-to-Speech (TTS)**: 플랫폼별 음성 합성
  - Windows: SAPI
  - macOS: NSSpeechSynthesizer
  - Linux: espeak
- **Wake Word**: 선택적 웨이크 워드 활성화
- **UI**: Settings > Voice 탭에서 설정 가능

### 🔌 Plugin System
- **Plugin Loader**: 매니페스트 기반 플러그인 로드
- **Sandbox Execution**: 권한 기반 샌드박스 실행 환경
- **Resource Limits**: 메모리, CPU, 실행 시간 제한
- **Permission Types**:
  - 파일 시스템 접근 (paths, access level)
  - 네트워크 접속 (hosts)
  - 데이터베이스 접근 (tables)
  - 시스템 기능 (capabilities)
- **UI**: Settings > Plugins 탭에서 관리

### 🔗 External Integrations
- **Database**: PostgreSQL, MySQL 연결 설정
- **Git**: 저장소 관리 (경로, 사용자 정보)
- **Cloud Storage**: AWS S3, GCS, Azure Blob 연결
- **UI**: 메인 네비게이션 > Integrations, Settings > Integrations

### 📄 Template Library
- **Template Management**: 템플릿 생성, 수정, 삭제
- **Visibility Levels**: private, public, team
- **Categories**: 템플릿 분류
- **Search**: 이름으로 템플릿 검색
- **UI**: 메인 네비게이션 > Templates, Settings > Templates

## Database Schema Changes

### New Tables
- `voice_settings`: 음성 설정 저장
- `plugins`: 설치된 플러그인 정보
- `templates`: 템플릿 저장

## API Changes

### New Tauri Commands

#### Voice
- `init_stt(model: String) -> Result<String, String>`
- `voice_transcribe(audio_data: Vec<u8>, language: String) -> Result<TranscriptionResult, String>`
- `get_available_models() -> Result<Vec<String>, String>`
- `init_tts(voice: String) -> Result<String, String>`
- `voice_synthesize(text: String, language: String) -> Result<SynthesisResult, String>`
- `voice_get_available_voices() -> Result<Vec<VoiceInfo>, String>`
- `get_voice_settings() -> Result<VoiceSettings, String>`
- `update_voice_settings(...) -> Result<(), String>`

#### Plugins
- `list_plugins() -> Result<Vec<Plugin>, String>`
- `get_plugin(id: String) -> Result<Plugin, String>`
- `install_plugin(...) -> Result<String, String>`
- `uninstall_plugin(id: String) -> Result<String, String>`
- `enable_plugin(id: String) -> Result<(), String>`
- `disable_plugin(id: String) -> Result<(), String>`

#### Templates
- `list_templates() -> Result<Vec<Template>, String>`
- `get_template(id: String) -> Result<Template, String>`
- `create_template(...) -> Result<String, String>`
- `update_template(...) -> Result<(), String>`
- `delete_template(id: String) -> Result<(), String>`
- `search_templates(query: String) -> Result<Vec<Template>, String>`

#### Integrations
- `test_database_connection(config: DatabaseConfig) -> Result<bool, String>`
- `get_database_connection_string(name: String) -> Result<String, String>`
- `validate_git_repository(path: String) -> Result<GitStatus, String>`
- `get_git_status(path: String) -> Result<GitStatus, String>`
- `get_git_current_commit(path: String) -> Result<String, String>`
- `test_cloud_connection(config: CloudConfig) -> Result<bool, String>`
- `list_cloud_objects(config: CloudConfig) -> Result<Vec<CloudObject>, String>`
- `get_cloud_endpoint(provider: String) -> Result<String, String>`

## Frontend Changes

### New Stores
- `stores/voiceStore.ts`: 음성 설정 및 STT/TTS 관리
- `stores/pluginStore.ts`: 플러그인 관리
- `stores/collaborationStore.ts`: 템플릿 및 협업 기능
- `stores/integrationsStore.ts`: 외부 통합 관리

### New Components
- `components/voice/VoiceSettings.tsx`: 음성 설정 UI
- `components/voice/VoiceButton.tsx`: 음성 입력 버튼
- `components/plugins/PluginList.tsx`: 플러그인 목록
- `components/collaboration/TemplateLibrary.tsx`: 템플릿 라이브러리
- `components/collaboration/ExportDialog.tsx`: 내보내기 대화상자
- `components/integrations/IntegrationsPanel.tsx`: 통합 패널

### Updated Components
- `components/settings/SettingsDialog.tsx`: Voice, Plugins, Templates, Integrations 탭 추가
- `App.tsx`: Integrations, Templates 네비게이션 추가

## Technical Details

### Module Structure (src-tauri/src/)
```
voice/
├── mod.rs       # VoiceSettings, TranscriptionResult, SynthesisResult
├── stt.rs       # Speech-to-Text implementation (Whisper)
└── tts.rs       # Text-to-Speech implementation (platform-specific)

plugins/
├── mod.rs       # Plugin types, permissions, state
├── loader.rs    # Plugin loading and validation
├── sandbox.rs   # Sandboxed execution environment
└── api.rs       # Plugin API definitions

collaboration/
├── mod.rs       # Template, SharedWorkflow, ExportOptions
├── templates.rs # TemplateManager implementation
└── export_mod.rs # JSON/Markdown/HTML export

integration/
├── mod.rs       # Integration types, status
├── database.rs  # PostgreSQL/MySQL connection
├── cloud.rs     # AWS S3, GCS, Azure Blob
└── git.rs       # Git repository operations
```

## Known Limitations

1. **Voice**: Whisper 모델 다운로드 필요 (첫 실행 시)
2. **Plugins**: 실행 중인 플러그인 중지 기능 미구현
3. **Integrations**: 연결 테스트만 지원, 실제 데이터 전송 미구현
4. **Templates**: 공유 기능은 UI만 구현됨

## Migration Guide

v0.3 → v0.4 업그레이드 시:
1. `npm install`으로 새로운 의존성 설치
2. `cd src-tauri && cargo build`로 Rust 백엔드 빌드
3. 데이터베이스 마이그레이션 자동 수행 (voice_settings 테이블 생성)

## Future Work (v0.5)

### Overview
v0.5는 v0.4에서 구현된 기능들의 실제 동작을 구현하는 릴리스입니다.

### 플러그인 실행 엔진
- **현재**: UI 및 타입만 구현됨
- **v0.5 목표**:
  - WASM/WASI 기반 플러그인 샌드박스 실행
  - 플러그인 생명주크 관리 (시작/중지/재시작)
  - 플러그인 간 메시지 전달
  - 리소스 사용량 모니터링

### 통합 서비스 데이터 연동
- **현재**: 연결 설정 UI만 구현됨
- **v0.5 목표**:
  - PostgreSQL/MySQL 실제 쿼리 실행
  - Git 저장소 실제 작업 (clone, pull, push, commit)
  - AWS S3/GCS/Azure Blob 업로드/다운로드
  - 통합 결과를 채팅에 표시

### 템플릿 공유 기능
- **현재**: 로컬 템플릿 관리만 가능
- **v0.5 목표**:
  - 템플릿 JSON export/import
  - Marketplace에서 템플릿 공유
  - 팀 공유 템플릿 (공유 폴더/DB)
  - 템플릿 버전 관리

### 고급 음성 명령어
- **현재**: 기본 STT/TTS만 구현됨
- **v0.5 목표**:
  - 음성 명령어 파싱 및 실행
  - 자연어 명령어 → 스킬/레시피 실행
  - 음성으로 대화 전환
  - 멀티턴어 음성 대화

### 기타 개선사항
- **Marketplace 개선**:
  - 실제 외부 마켓플레이스 API 연동
  - 리뷰 및 평점 시스템
  - 자동 업데이트 확인

- **성능 최적화**:
  - 대용량 파일 처리 개선
  - 메모리 사용량 최적화
  - 캐싱 전략 개선

- **보안 강화**:
  - API 키 암호화 저장
  - 플러그인 권한 세분화
  - 감사 로그

### v0.5 일정
- **Phase 1** (2주): 플러그인 실행 엔진 ✅
- **Phase 2** (2주): 통합 서비스 데이터 연동 ✅
- **Phase 3** (1주): 템플릿 공유 ✅
- **Phase 4** (1주): 고급 음성 명령어 ✅
- **Phase 5** (1주): 테스트 및 안정화 ✅

## 최근 완료된 작업

### WASM Plugin Runtime 실제 구현 (2026-02-22)
- `src-tauri/src/plugins/runtime.rs`: Wasmtime v22+ WASI preview1 통합 완료
  - `WasiP1Ctx` 타입 사용, `add_to_linker_sync()` API 통합
  - 실제 Store/Instance 생성 (placeholder 제거)
  - Fuel metering 활성화 (`consume_fuel(true)`)
  - 실제 WASM 함수 호출 구현 (`add`, `init`, `shutdown` 등)
- `src-tauri/src/plugins/wasi_host.rs`: WASI 호스트 업데이트
  - `build_p1()` 메서드 사용 for Wasmtime 22+ 호환
  - `WasiP1Ctx` 반환 타입 업데이트
- `src-tauri/src/plugins/sandbox.rs`: Path Traversal 취약점 수정
  - `canonicalize()` 사용하여 `../` 공격 방지
- `src-tauri/src/plugins/executor.rs`: Non-UTF-8 경로 처리 수정
  - `.unwrap()` 제거, 안전한 에러 처리 추가
- `src-tauri/tests/plugins_test.rs`: WASM 통합 테스트 추가
  - 8개 테스트 전체 통과 (module loading, instantiation, function calling, fuel metering)
- **보안 수정**: Path Traversal 취약점, Non-UTF-8 path panic 수정
- **테스트 결과**: 90개 테스트 전체 통과 (82 unit + 8 integration)

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

---

# v0.6 Release Notes (2026-02-22)

## Overview
v0.6는 AI 에이전트 시스템 고도화, 워크플로우 자동화, 및 클라우드 동기화의 기초 구조를 완성한 메이저 업데이트입니다.

## New Features

### 🤖 AI Agent System Enhancement (v0.6)
- **Multimodal Input Processing**: 텍스트, 이미지, 혼합 입력 처리
  - 지원 이미지 형식: PNG, JPEG, GIF, WebP, BMP
  - 이미지 분석: 캡셔닝, 객체 감지, OCR, 태깁
  - Vision Provider 추상화로 다양한 비전 모델 지원 가능

- **Context Management**: 대화 컨텍스트 관리 및 압축
  - 단기/장기 메모리 분리
  - 우선순위 기반 메시지 관리 (Low, Normal, High, Critical)
  - 컨텍스트 압축 전략 (RemoveOldest, Summarize, PriorityOnly, Hybrid)
  - 토큰 한도 자동 모니터링 및 압축

- **Sub-Agent Orchestration**: 전문화된 서브에이전트 오케스트레이션
  - 에이전트 타입: General, CodeGenerator, CodeReviewer, Researcher, DataAnalyst, FileOperator, WebScraper
  - 우선순위 기반 작업 큐
  - 병렬 작업 실행 및 결과 집계
  - 의존성 관리

### ⚙️ Workflow Automation (v0.6)
- **Workflow Store**: 워크플로우 저장소
  - In-memory 저장소 구현 (DB 연동은 향후)
  - 워크플로우 CRUD 작업
  - 실행 기록 관리

- **Workflow Engine**: 워크플로우 실행 엔진
  - 노드 기반 실행
  - 기본 노드 타입: Trigger, Action, Condition, Loop, Agent
  - 에러 핸들링 및 결과 집계

- **Trigger System**: 다양한 트리거 타입 지원
  - Schedule: Cron 기반 스케줄링
  - Webhook: HTTP 웹훅
  - FileSystem: 파일 시스템 이벤트
  - Voice: 음성 명령어
  - Manual: 수동 실행

### ☁️ Cloud Synchronization (v0.6)
- **Sync Manager**: 클라우드 동기화 관리자
  - 업로드/다운로드/삭제 작업 큐
  - 자동 동기화 지원
  - 동기화 결과 집계

- **Conflict Resolution**: 동기화 충돌 해결
  - 전략: ClientWins, ServerWins, Merge, Manual
  - 충돌 감지 및 해결 API
  - 머지 전략 지원

- **Offline Queue**: 오프라인 큐 관리
  - 네트워크 연결 실패 시 작업 대기
  - 재시도 메커니즘
  - 실패 작업 관리

## Module Structure (src-tauri/src/)

### Agent Module (`agent/`)
- `mod.rs`: Agent 모듈 진입점
- `multimodal.rs`: 멀티모달 입력 처리 (텍스트, 이미지)
- `context.rs`: 컨텍스트 관리 및 압축
- `orchestrator.rs`: 서브에이전트 오케스트레이션
- `commands.rs`: Tauri 명령어

### Workflow Module (`workflow/`)
- `mod.rs`: Workflow 모듈 진입점
- `store.rs`: 워크플로우 저장소 (InMemoryWorkflowStore, WorkflowStore trait)
- `engine.rs`: 워크플로우 실행 엔진
- `nodes.rs`: 노드 타입 및 실행기 정의
- `triggers.rs`: 트리거 관리자
- `commands.rs`: Tauri 명령어

### Sync Module (`sync/`)
- `mod.rs`: Sync 모듈 진입점
- `manager.rs`: 동기화 관리자 (SyncManager, CloudProvider trait)
- `conflict.rs`: 충돌 해결 (ConflictResolver, ConflictStrategy)
- `offline.rs`: 오프라인 큐 (OfflineQueue, PendingOperation)
- `commands.rs`: Tauri 명령어

## New Tauri Commands (v0.6)

### Agent Commands
- `agent_multimodal_process`: 멀티모달 입력 처리
- `agent_analyze_image`: 이미지 분석
- `agent_context_add_message`: 컨텍스트에 메시지 추가
- `agent_context_get_messages`: 컨텍스트 메시지 조회
- `agent_context_clear`: 컨텍스트 초기화
- `agent_context_token_count`: 토큰 수 조회
- `agent_context_is_near_limit`: 한도 근접 확인
- `agent_context_compress`: 컨텍스트 압축
- `agent_context_set_strategy`: 압축 전략 설정
- `agent_orchestrator_add_task`: 오케스트레이터 작업 추가
- `agent_orchestrator_execute_all`: 모든 작업 실행
- `agent_orchestrator_queue_length`: 큐 길이 조회
- `agent_orchestrator_clear_completed`: 완료된 결과 초기화

### Workflow Commands
- `workflow_create`: 워크플로우 생성
- `workflow_get`: 워크플로우 조회
- `workflow_list`: 모든 워크플로우 목록
- `workflow_list_active`: 활성화된 워크플로우 목록
- `workflow_update`: 워크플로우 수정
- `workflow_delete`: 워크플로우 삭제
- `workflow_add_node`: 워크플로우에 노드 추가
- `workflow_add_connection`: 노드 연결 추가
- `workflow_execute`: 워크플로우 실행
- `workflow_create_execution`: 실행 레코드 생성
- `workflow_get_execution`: 실행 레코드 조회
- `workflow_get_executions`: 워크플로우 실행 목록
- `workflow_update_execution`: 실행 상태 업데이트
- `workflow_register_trigger`: 트리거 등록
- `workflow_unregister_trigger`: 트리거 해제
- `workflow_list_triggers`: 활성 트리거 목록
- `workflow_trigger_count`: 트리거 수 조회

### Sync Commands
- `sync_now`: 지금 동기화 실행
- `sync_queue_upload`: 업로드 작업 큐에 추가
- `sync_queue_download`: 다운로드 작업 큐에 추가
- `sync_queue_delete`: 삭제 작업 큐에 추가
- `sync_pending_count`: 대기 작업 수 조회
- `sync_needs_sync`: 동기화 필요 여부 확인
- `sync_clear_pending`: 대기 작업 초기화
- `sync_set_conflict_strategy`: 충돌 해결 전략 설정
- `sync_detect_conflict`: 충돌 감지
- `sync_resolve_conflict`: 충돌 해결
- `sync_offline_push`: 오프라인 큐에 작업 추가
- `sync_offline_pop_ready`: 준비된 작업 꺼내기
- `sync_offline_peek`: 다음 작업 확인
- `sync_offline_mark_failed`: 작업 실패 표시
- `sync_offline_length`: 큐 길이 조회
- `sync_offline_clear`: 오프라인 큐 초기화
- `sync_offline_get_failed`: 실패 작업 조회
- `sync_offline_get_by_entity`: 엔티티별 작업 조회

## Frontend Types (src/types/)

- `agent.ts`: Agent 관련 TypeScript 타입
  - ImageFormat, InputType, ImageAnalysis
  - Message, MessageRole, MessagePriority
  - CompressionStrategy, CompressionResult
  - AgentType, TaskPriority, SubAgentTask, AggregatedResult

- `workflow.ts`: Workflow 관련 TypeScript 타입
  - Workflow, WorkflowDefinition, WorkflowNode
  - ExecutionStatus, WorkflowExecution, ExecutionResult
  - Trigger, TriggerType, TriggerHandle
  - NodePosition, NodeConnection, HttpMethod, FsEvent

- `sync.ts`: Sync 관련 TypeScript 타입
  - SyncEntity, SyncOperation, SyncResult
  - ConflictStrategy, SyncConflict, ConflictResolution
  - PendingOperation

## Frontend Stores (src/stores/)

- `agentStore.ts`: Agent 기능 Zustand 스토어
  - 컨텍스트 관리 (addMessage, getMessages, clearContext, compressContext)
  - 멀티모달 처리 (processMultimodal, analyzeImage)
  - 오케스트레이터 (addTask, executeAll, getQueueLength)

- `workflowStore.ts`: Workflow 기능 Zustand 스토어
  - 워크플로우 CRUD (loadWorkflows, createWorkflow, updateWorkflow, deleteWorkflow)
  - 노드 관리 (addNode, addConnection)
  - 실행 관리 (executeWorkflow, createExecution, getExecutions)
  - 트리거 관리 (registerTrigger, unregisterTrigger, listTriggers)

- `syncStore.ts`: Sync 기능 Zustand 스토어
  - 동기화 관리 (syncNow, queueUpload, queueDownload, queueDelete)
  - 충돌 해결 (detectConflict, resolveConflict)
  - 오프라인 큐 (pushToQueue, popReadyFromQueue, markFailed)

## Test Coverage

### Integration Tests
- `tests/agent_integration_test.rs`: 10개 에이전트 통합 테스트
- `tests/workflow_integration_test.rs`: 10개 워크플로우 통합 테스트
- `tests/sync_integration_test.rs`: 10개 동기화 통합 테스트

### Unit Tests
- 126개 단위 테스트 통과 (v0.5에서 121개에서 증가)
- 각 모듈별 포괄적인 테스트 커버리지

## Known Limitations

1. **Workflow Store**: 현재는 InMemoryStore만 구현. DB 영속성은 향후 작업
2. **Vision Provider**: MultimodalProcessor는 placeholder 구현. 실제 vision API 연동 필요
3. **Cloud Provider**: SyncManager는 mock 구현만 포함. 실제 클라우드 API 연동 필요
4. **Frontend UI**: v0.6 기능의 UI 컴포넌트는 향후 작업

## Migration Guide

v0.5 → v0.6 업그레이드 시:
1. `cargo build`로 Rust 백엔드 빌드
2. `npm install`로 새로운 의존성 설치
3. 데이터베이스 마이그레이션 없음 (v0.6은 새로운 구조 추가)
4. 새로운 Tauri 명령어 사용 가능

## Future Work (v0.7)

### Workflow Enhancements
- DB 영속성 (SQLite)
- 시각적 워크플로우 에디터 UI
- 더 많은 노드 타입
- 워크플로우 템플릿

### Cloud Integration
- 실제 클라우드 제공자 연동 (AWS S3, Google Drive, Dropbox)
- 실시간 동기화
- 백그라운드 동기화 스케줄링

### AI Enhancements
- 실제 Vision API 연동 (GPT-4 Vision, Claude 3.5 Sonnet)
- 실제 LLM 기반 컨텍스트 압축
- 더 많은 서브에이전트 타입

### UI Components
- 워크플로우 빌더 UI
- 동기화 상태 대시보드
- 에이전트 오케스트레이션 시각화
- 멀티모달 채팅 UI
