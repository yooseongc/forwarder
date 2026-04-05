# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SSH Forwarder — 다수의 SSH 포트포워딩(Local/Remote/Dynamic)을 관리하는 Windows 데스크톱 앱.
Tauri v2 (Rust 백엔드) + React (TypeScript 프론트엔드) 구조.

## Commands

```bash
# 개발 서버 (프론트엔드 + Rust 동시 빌드 & 실행)
npm run tauri dev

# 프론트엔드만
npm run dev

# 타입 체크
npx tsc --noEmit              # TypeScript
cd src-tauri && cargo check   # Rust

# 릴리스 빌드 (.msi / .exe 생성)
npm run tauri build

# 테스트
cd src-tauri && cargo test --lib          # Rust 유닛 (42개)
cd src-tauri && cargo test --test ssh_integration  # SSH 통합 (Docker 필요, 4개)
npm test                                  # TypeScript (Vitest, 44개)

# Docker SSH 서버 (통합 테스트용)
cd tests && docker compose up -d          # 시작
cd tests && docker compose down           # 정지
```

## Architecture

```
forwarder/
├── src-tauri/          # Rust 백엔드 (Tauri 앱)
│   ├── src/
│   │   ├── lib.rs          # 앱 진입점, 트레이, --minimized 지원
│   │   ├── commands.rs     # Tauri IPC 커맨드 (17개)
│   │   ├── error.rs        # AppError 구조화된 에러 (code + message)
│   │   ├── state.rs        # AppState, ConnectionState
│   │   ├── credential.rs   # Windows Credential Manager
│   │   ├── config/         # 설정 영속화 (%APPDATA%/forwarder/config.json)
│   │   └── ssh/            # SSH 세션, L/R/D 터널링, SOCKS5
│   └── tests/              # Rust 통합 테스트 (Docker SSH)
├── src/                # React 프론트엔드
│   ├── hooks/              # Tauri API 래퍼, 연결 상태 관리 훅
│   ├── components/         # UI 컴포넌트
│   │   └── ui/             # 공통 원자 컴포넌트
│   ├── types/              # TypeScript 타입 + 공유 상수
│   ├── i18n/               # 다국어 지원 (ko, en)
│   └── __tests__/          # Vitest 테스트
├── tests/              # Docker SSH 서버 인프라
└── docs/               # 설계 문서
```

## Data Flow

```
Frontend invoke() → Tauri IPC → commands.rs → ssh/session.rs (russh)
                                            → config/store.rs (JSON)
                                            → credential.rs (keyring)

State change  → app.emit("connection-status-changed") → Frontend listener
Health check  → 10s interval is_closed() → "Connection lost" emit
Error flow    → Result<T, AppError> → { code: "ERROR_CODE", message: "..." }
```

## Key Conventions

- Rust ↔ TypeScript 타입: `serde(rename_all = "camelCase")` 자동 매핑
- 모든 Tauri 커맨드: `commands.rs`에 집중, `Result<T, AppError>` 반환
- 비밀번호: Windows Credential Manager만 사용, config 파일에 절대 미저장
- 창 닫기 = 트레이 최소화, 트레이 더블클릭 = 복원, 종료 시 `disconnect_all()`
- `--minimized`: autostart 시 트레이 모드 시작 (창 숨김)
- 터널 graceful shutdown: `CancellationToken` → 리스너 정리 → abort
- Remote forwarding: `connect()` 시 Arc 래핑 전에 `tcpip_forward` 호출, `server_channel_open_forwarded_tcpip`으로 역방향 프록시
- Config 동시 접근: `CONFIG_LOCK` (static Mutex), `FORWARDER_CONFIG_DIR` 환경변수 오버라이드
- i18n: `src/i18n/` — `t("key")` 함수, ko/en 지원
