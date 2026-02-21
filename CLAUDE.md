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
- **Phase 1** (2주): 플러그인 실행 엔진
- **Phase 2** (2주): 통합 서비스 데이터 연동
- **Phase 3** (1주): 템플릿 공유
- **Phase 4** (1주): 고급 음성 명령어
- **Phase 5** (1주): 테스트 및 안정화

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
