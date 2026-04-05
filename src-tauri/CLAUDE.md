# CLAUDE.md — Rust Backend (src-tauri)

## Build & Test

```bash
cargo check            # 타입 체크
cargo test --lib       # 유닛 테스트 (42개)
cargo test --test ssh_integration  # SSH 통합 테스트 (Docker 필요, 4개)
cargo build --release  # 릴리스 빌드
```

## Module Structure

### lib.rs — 앱 진입점
- Tauri Builder: 플러그인 등록, 상태 관리, 트레이 아이콘
- `setup_tray()`: 트레이 메뉴(창 열기/종료) + 더블클릭 복원
- `disconnect_all()`: 종료 시 모든 활성 SSH 세션 정리
- `--minimized` 인자 감지: autostart 시 창 숨김
- `on_window_event`: CloseRequested → 숨기기 (트레이 최소화)

### commands.rs — Tauri IPC 인터페이스 (17개 커맨드)
- CRUD: `get_profiles`, `save_profile`, `delete_profile`
- 연결: `connect_profile`, `disconnect_profile`, `reconnect_profile`
- 터널: `enable_tunnel` (런타임 개별 터널 시작)
- 상태: `get_all_status`
- 자격증명: `save_credential`, `delete_credential`, `has_credential`
- 설정: `get_autostart_enabled`, `set_autostart_enabled`
- 파일: `open_key_file_dialog`
- Config: `export_config`, `import_config`
- 모든 커맨드는 `Result<T, AppError>` 반환
- `do_connect()`: connect + auto-connect 공용, Connecting 상태 검증 (경합 방지)
- Health check 태스크: 연결 성공 시 10초 주기 `is_closed()` 감시 → 끊김 시 Error emit

### error.rs — 구조화된 에러 타입
- `AppError { code: ErrorCode, message: String }` — Serialize
- ErrorCode (SCREAMING_SNAKE_CASE): `PROFILE_NOT_FOUND`, `AUTH_FAILED`, `CONNECTION_FAILED`, `TUNNEL_BIND_FAILED`, `TUNNEL_UNSUPPORTED`, `CONFIG_ERROR`, `CREDENTIAL_ERROR`, `INTERNAL`

### state.rs — 런타임 상태
- `AppState` (Clone): `Arc<Mutex<HashMap<String, ConnectionState>>>`
- `ConnectionState`: 상태 전이 메서드 (`new_connecting`, `set_connected`, `set_error`, `set_disconnected`)
- `TunnelError`: named struct (rule_id, message)

### ssh/session.rs — SSH 클라이언트
- `ClientHandler`: `check_server_key` (TOFU 정책), `server_channel_open_forwarded_tcpip` (역방향 프록시)
- `SshSession::connect(profile, remote_rules)`: 인증 + Remote tcpip_forward 등록 (Arc 전)
- `start_tunnels()`: Local/Remote/Dynamic 태스크 스폰, CancellationToken
- `start_single_tunnel()`: 런타임 개별 터널 추가
- `start_health_check()`: 10초 주기 is_closed() 감시
- `disconnect()`: cancel → abort
- `proxy_channel()`: `Channel::into_stream()` + `tokio::io::copy`
- Remote: `run_remote_keepalive` — tcpip_forward 유지 + cancel 시 `cancel_tcpip_forward`

### ssh/socks5.rs — SOCKS5 프록시
- 입력 검증: MAX_AUTH_METHODS=255, 빈 도메인 거부
- IPv6: `std::net::Ipv6Addr` 사용

### config/store.rs — JSON 영속화
- `FORWARDER_CONFIG_DIR` 환경변수 오버라이드
- 손상 시 `.json.bak` 백업 + 폴백
- `CONFIG_LOCK` (static Mutex) 동시 접근 보호

### credential.rs — Windows Credential Manager
- 서비스명 `ssh-forwarder`, 키 = profile ID

## Test Structure (42개 유닛 + 4개 통합)

| 모듈 | 수량 | 내용 |
|------|------|------|
| `error.rs` | 5 | ErrorCode 직렬화, 생성자, Display, From, JSON |
| `config/types.rs` | 7 | serde 라운드트립, 기본값, camelCase |
| `ssh/types.rs` | 5 | ConnectionStatus, TunnelStatus 직렬화 |
| `state.rs` | 5 | 상태 전이, AppState, TunnelError |
| `config/store.rs` | 7 | CRUD, 손상 복구, 배치, CONFIG_LOCK |
| `ssh/socks5.rs` | 8 | SOCKS5 프로토콜 파싱 (IPv4/IPv6/Domain) |
| `credential.rs` | 5 | save/get/delete/has/overwrite (Windows CM) |
| `tests/ssh_integration.rs` | 4 | password/key 인증, local forward echo |

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `russh` 0.46 | SSH 프로토콜 (순수 Rust) |
| `russh-keys` 0.46 | SSH 키 파싱 |
| `keyring` 3 | OS 자격증명 저장소 |
| `tauri` 2 | 데스크톱 앱 프레임워크 |
| `tauri-plugin-autostart` 2 | Windows 시작 프로그램 등록 |
| `tauri-plugin-dialog` 2 | 파일 선택 다이얼로그 |
| `tokio` 1 | 비동기 런타임 |
| `tokio-util` 0.7 | CancellationToken |
| `tempfile` 3 (dev) | 테스트용 임시 디렉토리 |
