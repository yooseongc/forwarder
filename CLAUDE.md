# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SSH Forwarder — 다수의 SSH 포트포워딩(Local/Remote/Dynamic)을 관리하는 Windows 데스크톱 앱.
Tauri v2 (Rust) + React 19 + shadcn/ui.

## Commands

```bash
# Dev
npm run tauri dev              # 프론트엔드 + Rust 동시 실행

# Type check
npx tsc --noEmit
cd src-tauri && cargo check

# Test
cd src-tauri && cargo test --lib                    # Rust 유닛 (60개)
cd src-tauri && cargo test --test ssh_integration   # SSH 통합 (4개, Docker 필요)
npm test                                            # Vitest (43개)

# Release
npm run tauri build                      # 로컬 빌드
./scripts/release.sh v1.0.0              # 빌드 + 태그 + gh release upload

# Docker SSH 테스트 서버
cd tests && docker compose up -d         # 시작
cd tests && docker compose down          # 정지
```

## Architecture

```
forwarder/
├── src-tauri/          # Rust 백엔드 (Tauri v2)
│   ├── src/
│   │   ├── lib.rs          # 앱 진입점, 트레이, single-instance, --minimized
│   │   ├── commands.rs     # Tauri IPC 커맨드 (18개)
│   │   ├── error.rs        # AppError (code + message)
│   │   ├── state.rs        # AppState, ConnectionState
│   │   ├── credential.rs   # Windows Credential Manager
│   │   ├── config/         # JSON 설정 영속화
│   │   └── ssh/            # russh + SOCKS5 + 리버스 프록시 + 키 포맷(PPK/EC PEM)
│   └── tests/              # Docker SSH 통합 테스트
├── src/                # React 프론트엔드
│   ├── hooks/              # useConnections, useTauri, useTheme
│   ├── components/         # UI 컴포넌트
│   │   └── ui/             # shadcn/ui (Button, Input, Select, Switch, Card, ...)
│   ├── types/              # TypeScript 타입, extractErrorMessage 유틸
│   ├── i18n/               # ko / en
│   ├── lib/                # shadcn utils (cn)
│   └── __tests__/          # Vitest 테스트
├── tests/              # Docker SSH 서버 (openssh + python http + socat echo)
└── docs/               # 설계 문서 + 스타일 가이드
```

## Key Conventions

- **타입 매핑**: Rust `serde(rename_all = "camelCase")` ↔ TypeScript
- **에러**: 모든 Tauri 커맨드는 `Result<T, AppError>` 반환, 프론트엔드는 `extractErrorMessage()` 로 파싱
- **비밀번호**: Windows Credential Manager만 사용, config 파일에 절대 미저장
- **창 닫기 = 트레이 최소화**, 트레이 더블클릭 = 복원, 종료 시 `disconnect_all()`
- **Single Instance**: 중복 실행 방지 (기존 창으로 포커스)
- **--minimized**: autostart 시 창 숨김
- **터널 graceful shutdown**: `CancellationToken` → 리스너 정리 → abort
- **Remote forwarding**: Arc 래핑 전 `tcpip_forward` 호출, `server_channel_open_forwarded_tcpip`에서 역방향 프록시
- **Config 동시 접근**: `CONFIG_LOCK` static Mutex, `FORWARDER_CONFIG_DIR` 환경변수 오버라이드
- **UI**: shadcn/ui 기반, 다크/라이트/시스템 테마 (CSS `.dark` 클래스), i18n `t()` 함수
- **Import alias**: `@/*` → `src/*` (tsconfig + vite + vitest 모두 설정)

## Data Flow

```
Frontend api.foo() → Tauri invoke → commands.rs → ssh/session.rs (russh)
                                                → config/store.rs (JSON)
                                                → credential.rs (keyring)

State change  → app.emit("connection-status-changed") → Frontend listener
Health check  → 10s is_closed() → "Connection lost" emit
Error         → Result<T, AppError> → { code, message }
```
