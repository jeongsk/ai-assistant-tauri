# AI Personal Assistant - Tauri Project

> OpenClaw + Goose + OpenHands + Cowork + Accomplish의 장점을 통합한
> 개인 비서 데스크톱 애플리케이션

## 프로젝트 개요

- **프레임워크**: Tauri v2 (Rust + Web)
- **프론트엔드**: React + TypeScript
- **에이전트 런타임**: Node.js Sidecar
- **프로토콜**: MCP (Model Context Protocol)

## 핵심 설계 원칙

1. **로컬 우선** - 모든 데이터는 사용자 머신에 저장
2. **확장성** - MCP + Skills로 무한 확장 가능
3. **보안** - 샌드박스 실행, 명시적 권한 관리
4. **성능** - Rust 코어, 경량 프론트엔드
5. **프라이버시** - BYOK (Bring Your Own Key), 로컬 모델 지원

## 기술 스택

| 레이어 | 기술 | 비고 |
|--------|------|------|
| Desktop | Tauri v2 | Rust 코어 |
| Frontend | React 18 + TypeScript | Vite 빌드 |
| State | Zustand + TanStack Query | 경량 상태관리 |
| UI | shadcn/ui + Tailwind CSS | 일관된 디자인 |
| Agent Runtime | Node.js 22 (Sidecar) | MCP 생태계 활용 |
| Local LLM | Ollama (Sidecar) | 프라이빗 추론 |
| Database | SQLite (rusqlite) | 로컬 영속성 |

## 참고 프로젝트별 차용 기능

### OpenClaw
- 스킬 시스템 (동적 로딩)
- 세션/서브에이전트 관리
- Cron + Heartbeat (예약 작업)
- 메모리 시스템 (MEMORY.md)

### Goose (Block)
- MCP 우선 아키텍처
- Goosehints (프로젝트별 지시사항)
- Recipes (재사용 가능한 템플릿)
- 서브에이전트

### OpenHands
- Event-sourced 상태 관리
- 샌드박스 실행 환경
- 다중 에이전트 오케스트레이션
- 모델 무관 설계 (SDK)

### Claude Cowork (Anthropic)
- 폴더 기반 권한 관리
- Global/Folder Instructions
- 비개발자 친화적 UX
- MCP 커넥터

### Accomplish
- MIT 오픈소스
- BYOK + Ollama
- Skills 시스템
- 프라이버시 우선

## 릴리스 로드맵

### MVP (v0.1)
- [x] 프로젝트 초기화
- [ ] 기본 채팅 (OpenAI, Anthropic)
- [ ] 폴더 권한 관리
- [ ] 파일 읽기/쓰기 (MCP)
- [ ] 로컬 모델 (Ollama)
- [ ] 설정 UI
- [ ] 대화 기록 저장

### v0.2
- [ ] 스킬 시스템
- [ ] 레시피 엔진
- [ ] 서브에이전트
- [ ] Browser MCP
- [ ] 프로젝트 컨텍스트 (.agenthints)
- [ ] 메모리 시스템

### v0.3
- [ ] 스킬 마켓플레이스
- [ ] 다중 제공자 라우팅
- [ ] Cron 작업
- [ ] 외부 MCP (GitHub, Slack)
- [ ] 플러그인 API

---
작성일: 2026-02-17
