# SSH Forwarder — Architecture Design Document

## 1. Overview

SSH 포트포워딩을 GUI로 관리하는 Windows 데스크톱 앱.
여러 SSH 서버에 대한 Local/Remote/Dynamic 포워딩 규칙을 프로파일로 저장하고,
연결·해제·재연결·상태 모니터링·자동 시작을 제공한다.

## 2. System Architecture

```
┌─────────────────────────────────────────────────────┐
│  Windows                                            │
│  ┌───────────────────────────────────────────────┐  │
│  │  Tauri WebView (React + Tailwind v4)          │  │
│  │  ┌─────────────┐  ┌────────────────────────┐  │  │
│  │  │ Connection  │  │  Content Area           │  │  │
│  │  │ List        │  │  - ConnectionStatus     │  │  │
│  │  │ (sidebar)   │  │  - ConnectionForm       │  │  │
│  │  │             │  │  - SettingsView         │  │  │
│  │  └─────────────┘  └────────────────────────┘  │  │
│  └──────────────┬────────────────────────────────┘  │
│                 │ Tauri IPC (17 commands + events)   │
│  ┌──────────────▼────────────────────────────────┐  │
│  │  Rust Backend                                  │  │
│  │  ┌──────────┐  ┌────────┐  ┌──────────────┐  │  │
│  │  │commands  │  │ state  │  │ config/store │  │  │
│  │  │ error.rs │──│  .rs   │  │   .rs        │  │  │
│  │  └────┬─────┘  └────────┘  └──────┬───────┘  │  │
│  │       │                           │           │  │
│  │  ┌────▼──────────────┐    ┌───────▼────────┐  │  │
│  │  │ ssh/              │    │ %APPDATA%/     │  │  │
│  │  │ ├─ session.rs     │    │ forwarder/     │  │  │
│  │  │ ├─ socks5.rs      │    │ config.json    │  │  │
│  │  │ └─ types.rs       │    └────────────────┘  │  │
│  │  └────┬──────────────┘                        │  │
│  │       │                    ┌────────────────┐  │  │
│  │  ┌────▼──────────────┐    │ credential.rs  │  │  │
│  │  │ SSH Server(s)     │    │ → Win Cred Mgr │  │  │
│  │  │ via russh 0.46    │    └────────────────┘  │  │
│  │  └───────────────────┘                        │  │
│  └───────────────────────────────────────────────┘  │
│  ┌──────────┐                                       │
│  │ System   │  트레이, 자동시작, --minimized         │
│  │ Tray     │                                       │
│  └──────────┘                                       │
└─────────────────────────────────────────────────────┘
```

## 3. Data Model

```
ConnectionProfile
├── id: UUID
├── name, host, port, username
├── authMethod: Password | KeyFile | KeyFileWithPassphrase
├── forwardingRules[]: { kind: L|R|D, bind, remote, enabled }
└── autoConnect: bool
```

**영속화**: `%APPDATA%/forwarder/config.json` (CONFIG_LOCK, 손상 시 .bak 폴백)
**비밀번호**: Windows Credential Manager (`keyring`, 서비스 `ssh-forwarder`)
**테스트**: `FORWARDER_CONFIG_DIR` 환경변수 오버라이드

## 4. Error Handling

```
AppError { code: ErrorCode, message: String }
ErrorCode: PROFILE_NOT_FOUND | AUTH_FAILED | CONNECTION_FAILED
           TUNNEL_BIND_FAILED | TUNNEL_UNSUPPORTED | CONFIG_ERROR
           CREDENTIAL_ERROR | INTERNAL
```

## 5. State Management

**Backend**: `AppState.connections: Arc<Mutex<HashMap<String, ConnectionState>>>`
- 전이 메서드: `new_connecting()` → `set_connected()` / `set_error()` → `set_disconnected()`
- Health check: 10초 주기 `is_closed()` → 끊김 시 Error 전환 + 이벤트 emit

**Frontend**: `useConnections` 훅 — `profiles`, `statuses` (Map), `getStatus()`

## 6. SSH Tunneling

### Local (-L)
TCP listener (CancellationToken) → `channel_open_direct_tcpip` → `proxy_channel`

### Remote (-R)
`connect()` 시 Arc 전에 `tcpip_forward()` 호출 → `ClientHandler::server_channel_open_forwarded_tcpip`에서 역방향 연결 수신 → 로컬 대상으로 프록시
`run_remote_keepalive`: cancel 시 `cancel_tcpip_forward` 호출

### Dynamic (-D, SOCKS5)
TCP listener → `socks5::handle_client` (auth 협상 → CONNECT 파싱 → SSH 채널 → 프록시)

### Graceful Shutdown
`CancellationToken::cancel()` → 리스너 루프 종료 → abort
앱 종료: `disconnect_all()` → 전체 세션 정리

## 7. System Tray & Auto-start

- 창 닫기 = 트레이 최소화, 더블클릭 = 복원
- 종료 시 `disconnect_all()` 호출
- `tauri-plugin-autostart` + `--minimized` → 트레이 모드 시작
- `auto_connect: true` 프로파일 자동 연결

## 8. i18n

- `src/i18n/`: `t("key")` 함수, 타입 안전한 키
- 지원 언어: 한국어 (기본), 영어
- Rust 트레이 메뉴: 현재 한국어 고정 (TODO: 동적 전환)

## 9. Testing (총 90개)

| 카테고리 | 수량 | 설명 |
|----------|------|------|
| Rust 유닛 | 42 | error, types, state, store, socks5, credential |
| Rust 통합 | 4 | SSH 인증(pw/key), local forward echo (Docker) |
| TS 유닛 | 19 | types 유틸리티 |
| TS API | 10 | useTauri invoke 검증 |
| TS 컴포넌트 | 15 | ConnectionStatus(7) + ConnectionForm(8) |

**Docker**: `tests/docker-compose.yml` — Alpine+OpenSSH+socat, testuser:testpass123, 2222/9999

## 10. Security

- **Host key**: TOFU 정책 (Trust On First Use), Changed 시 거부 구조 준비됨
- **Credential**: OS 자격증명 관리자만 사용
- **Config export**: 비밀번호 미포함
- **SOCKS5**: 입력 검증 (method 수, 도메인, 주소 타입)

## 11. Known Limitations

| 항목 | 상태 | 비고 |
|------|------|------|
| Host key known_hosts 파일 | TOFU만 | 실제 파일 읽기/쓰기 미구현, 구조는 준비됨 |
| Auto-reconnect | 수동만 | `reconnect` 커맨드 존재, 자동 지수 백오프 미구현 |
| Rust 트레이 i18n | 한국어 고정 | 프론트엔드 i18n은 ko/en 지원 |
