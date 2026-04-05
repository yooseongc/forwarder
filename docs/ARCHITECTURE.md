# SSH Forwarder — Architecture

## Overview

SSH 포트포워딩을 GUI로 관리하는 Windows 데스크톱 앱.
Local/Remote/Dynamic 포워딩, 다중 프로파일, 상태 모니터링, 자동 시작, i18n, 테마 전환.

## System

```
┌─────────────────────────────────────────────────────┐
│  Windows                                            │
│  ┌───────────────────────────────────────────────┐  │
│  │  Tauri WebView — React + shadcn/ui            │  │
│  │  Layout → ConnectionList + Content area       │  │
│  │          (Status / Form / Settings)           │  │
│  └──────────────┬────────────────────────────────┘  │
│                 │ Tauri IPC (18 commands + events)   │
│  ┌──────────────▼────────────────────────────────┐  │
│  │  Rust Backend                                  │  │
│  │  commands.rs ─ state.rs ─ config/store.rs     │  │
│  │  error.rs             ssh/session.rs          │  │
│  │                       ssh/socks5.rs           │  │
│  │                       credential.rs           │  │
│  └───────────────────────────────────────────────┘  │
│  ┌──────────┐                                       │
│  │ Tray +   │  자동 시작, --minimized, single-inst  │
│  │ Autostart│                                       │
│  └──────────┘                                       │
└─────────────────────────────────────────────────────┘
```

## Data Model

```
ConnectionProfile
├── id: UUID
├── name, host, port, username
├── authMethod: Password | KeyFile | KeyFileWithPassphrase
├── forwardingRules[]: { kind: L|R|D, bind, remote, enabled }
└── autoConnect: bool
```

**영속화**
- 설정: `%APPDATA%/forwarder/config.json` (CONFIG_LOCK static Mutex, 손상 시 `.json.bak` 폴백)
- 비밀번호: Windows Credential Manager (`keyring`, 서비스 `ssh-forwarder`)
- 테스트: `FORWARDER_CONFIG_DIR` 환경변수 오버라이드

## Error Handling

```
AppError { code: ErrorCode, message: String }
ErrorCode: PROFILE_NOT_FOUND | AUTH_FAILED | CONNECTION_FAILED
           TUNNEL_BIND_FAILED | TUNNEL_UNSUPPORTED | CONFIG_ERROR
           CREDENTIAL_ERROR | INTERNAL
```
- 모든 Tauri 커맨드가 `Result<T, AppError>` 반환
- 프론트엔드는 `extractErrorMessage()`로 안전 파싱 (`[object Object]` 방지)

## State Management

**Backend** — `AppState.connections: Arc<Mutex<HashMap<String, ConnectionState>>>`

상태 전이 메서드:
- `new_connecting()` → Connecting
- `set_connected()` → Connected + session 저장
- `set_error()` → Error
- `set_disconnected()` → cancel token + abort + session drop

**Frontend** — `useConnections` 훅
- `profiles`, `statuses: Map<id, ProfileStatus>`, `getStatus()`, `refresh()`
- Tauri `connection-status-changed` 이벤트 구독 → 실시간 반영

## SSH Tunneling

### Local (-L)
```
[앱 호스트 bind] → accept → channel_open_direct_tcpip → [원격 대상]
```

### Remote (-R)
```
[SSH 서버 bind] → server_channel_open_forwarded_tcpip callback → [앱 호스트 local target]
```
- `connect()` 시 Arc 래핑 전 `tcpip_forward` 호출 (&mut Handle 필요)
- 역방향 연결은 `ClientHandler`의 callback에서 `ReverseMap`으로 타겟 매핑
- Cancel 시 `cancel_tcpip_forward` 호출

### Dynamic (-D, SOCKS5)
```
[로컬 SOCKS5] → handle_client → CONNECT 파싱 → channel_open_direct_tcpip → proxy_channel
```

### Graceful Shutdown
`CancellationToken::cancel()` → 리스너 루프 종료 → 태스크 abort
앱 종료: `disconnect_all()` → 전체 세션 정리 → `app.exit()`

## System Integration

### Tray
- 창 닫기(X) → `window.hide()` (트레이 최소화)
- 트레이 더블클릭 → 복원
- 컨텍스트 메뉴: 창 열기 / 종료

### Autostart
- `tauri-plugin-autostart`: Windows 레지스트리 등록
- `--minimized` 인자 → 창 숨긴 채 시작
- `auto_connect: true` 프로파일 자동 연결

### Single Instance
- `tauri-plugin-single-instance`: 중복 실행 방지
- 두 번째 실행 시도 → 기존 창 포커스

## i18n / Theme

- **i18n**: `src/i18n/` (ko/en) — `t("key")` 함수, `useLocale()` 훅, localStorage
- **Theme**: `src/hooks/useTheme.ts` — 라이트/다크/시스템, `.dark` 클래스 토글, localStorage

## Testing (89 total)

| 카테고리 | 수량 |
|----------|------|
| Rust 유닛 (error, types, state, store, socks5, credential) | 42 |
| Rust SSH 통합 (password/key auth, local forward echo) | 4 |
| TS 유닛 (types utilities) | 19 |
| TS API (useTauri invoke) | 10 |
| TS 컴포넌트 (ConnectionStatus + ConnectionForm) | 14 |

**Docker 테스트 서버**: Alpine + OpenSSH + Python HTTP(8080) + socat echo(9999)

## Security

- **Host key**: TOFU 정책 (known_hosts 파일 TODO)
- **Credential**: OS 자격증명 관리자만 사용, config 평문 미저장
- **Config export**: 비밀번호 미포함
- **SOCKS5**: auth method 수 제한, 빈 도메인 거부

## Known Limitations

| 항목 | 비고 |
|------|------|
| Host key known_hosts 파일 | TOFU만 구현, 파일 읽기/쓰기 미구현 |
| Auto-reconnect 백오프 | 수동 `reconnect` 커맨드만 제공 |
| Rust 트레이 i18n | "창 열기"/"종료" 한국어 고정 (프론트는 ko/en 전환) |
