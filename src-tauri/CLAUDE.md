# CLAUDE.md — Rust Backend (src-tauri)

## Build & Test

```bash
cargo check            # 타입 체크
cargo test --lib       # 유닛 테스트 (70개)
cargo test --test ssh_integration  # SSH 통합 (Docker 필요, 4개)
cargo build --release  # 릴리스 빌드
```

## Module Structure

### lib.rs — 앱 진입점
- Tauri Builder + 플러그인: single-instance, autostart, dialog, shell
- Tray icon 설정 (`setup_tray`), 창 닫기 → 숨기기
- `--minimized` 인자 감지 → 창 숨김
- `disconnect_all`: 종료 시 전체 SSH 세션 정리

### commands.rs — Tauri IPC (20 commands)
Profile CRUD: `get_profiles`, `save_profile`, `delete_profile`
Connection: `connect_profile`, `disconnect_profile`, `reconnect_profile`
Tunnel: `enable_tunnel`
Status: `get_all_status`
Credential: `save_credential`, `delete_credential`, `has_credential`
Autostart: `get_autostart_enabled`, `set_autostart_enabled`
File: `open_key_file_dialog`
Network: `ping_host`
Config: `export_config`, `import_config`
Host key: `reset_host_key`, `reset_all_host_keys`

모두 `Result<T, AppError>` 반환. `do_connect()` 공용 헬퍼 (커맨드 + auto-connect 공유).
Connection state race 방지: SSH 연결 후 재잠금 시 `Connecting` 상태 검증.
Health check: 10s 주기 `is_closed()` 감시 → 끊김 시 auto-reconnect (지수 백오프, 최대 5회) 또는 Error emit.

### error.rs — 구조화된 에러
`AppError { code: ErrorCode, message: String }` (Serialize)
- ErrorCode: PROFILE_NOT_FOUND / AUTH_FAILED / CONNECTION_FAILED / TUNNEL_BIND_FAILED / TUNNEL_UNSUPPORTED / CONFIG_ERROR / CREDENTIAL_ERROR / HOST_KEY_MISMATCH / INTERNAL

### state.rs — 런타임 상태
- `AppState` (Clone): `Arc<Mutex<HashMap<String, ConnectionState>>>`
- `ConnectionState`: 상태 전이 메서드 (`new_connecting`, `set_connected`, `set_error`, `set_disconnected`)
- `TunnelError`: rule_id + message

### ssh/known_hosts.rs — 호스트 키 검증
- `%APPDATA%/forwarder/known_hosts` 파일 관리 (OpenSSH 호환 형식)
- `verify_or_store`: TOFU — 최초 접속 시 키 저장, 이후 변경 감지
- `remove_host_key` / `clear_all`: 사용자 초기화
- `KNOWN_HOSTS_LOCK` (static Mutex) 동시 접근 보호

### ssh/session.rs — SSH 클라이언트
- `ClientHandler`: known_hosts 기반 `check_server_key` (TOFU + 변경 감지), `server_channel_open_forwarded_tcpip` (역방향 프록시)
- `SshSession::connect(profile, remote_rules)`:
  - 인증 (password / keyFile / keyFileWithPassphrase)
  - Remote forwarding 사전 등록 (`tcpip_forward` on `&mut Handle` before Arc wrap)
- `start_tunnels` / `start_single_tunnel`: 내부 `spawn_tunnel()` 헬퍼로 통합
- `start_health_check`: 10s 주기 health 감시 task
- `disconnect`: `cancel.cancel()` + `task.abort()` (graceful)
- Local/Dynamic: `tokio::select!` 기반 accept loop (cancel 감시)
- Remote: keep-alive task가 cancel 시 `cancel_tcpip_forward` 호출

### ssh/key_format.rs — 키 파일 포맷 지원
- `decode_key`: 포맷 자동 감지 후 적절한 파서로 분기
- 지원 포맷: OpenSSH, RSA PEM, PKCS#8, SEC1 EC PEM (P-256/P-384), PuTTY PPK v2/v3
- PPK 암호화: AES-256-CBC (v2: SHA-1 KDF, v3: Argon2id KDF), HMAC MAC 검증
- 공개키 파일 감지 시 명확한 에러 메시지

### ssh/socks5.rs — SOCKS5 프록시
- `handle_client`: negotiate → CONNECT 파싱 → SSH 채널 열기 → proxy
- 입력 검증: MAX_AUTH_METHODS=255, 빈 도메인 거부
- IPv4/IPv6(std::net::Ipv6Addr)/Domain 지원

### config/store.rs — JSON 영속화
- 경로: `%APPDATA%/forwarder/config.json` (`FORWARDER_CONFIG_DIR` 환경변수로 오버라이드)
- 손상 시 `.json.bak` 백업 + 기본값 폴백
- `CONFIG_LOCK` (static std::sync::Mutex) 동시 접근 보호

### credential.rs — Windows Credential Manager
`keyring` crate, 서비스명 `ssh-forwarder`, 키 = profile ID.
테스트는 각 테스트마다 고유 키 사용 (병렬 실행 안전).

## Test Structure (70 유닛 + 4 통합)

| 모듈 | 수 | 내용 |
|------|---|------|
| error.rs | 5 | ErrorCode 직렬화, 생성자, Display, From, JSON |
| config/types.rs | 7 | serde round-trip, 기본값, camelCase |
| ssh/types.rs | 5 | ConnectionStatus, TunnelStatus 직렬화 |
| state.rs | 5 | 상태 전이, AppState, TunnelError |
| config/store.rs | 7 | CRUD, 손상 복구, 배치, CONFIG_LOCK |
| ssh/known_hosts.rs | 10 | 호스트 키 CRUD, 검증, 변경 감지, 초기화 |
| ssh/key_format.rs | 18 | 포맷 감지, EC PEM/PPK 디코딩, hex/SSH reader |
| ssh/socks5.rs | 8 | SOCKS5 파싱 (IPv4/IPv6/Domain + 에러) |
| credential.rs | 5 | save/get/delete/has/overwrite (고유 키) |
| tests/ssh_integration.rs | 4 | password/key 인증, Local forward echo |

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` 2 | 데스크톱 앱 프레임워크 |
| `tauri-plugin-single-instance` 2 | 중복 실행 방지 |
| `tauri-plugin-autostart` 2 | Windows 시작 프로그램 |
| `tauri-plugin-dialog` 2 | 파일 선택 |
| `russh` 0.46 + `russh-keys` 0.46 | SSH 프로토콜 |
| `keyring` 3 | OS 자격증명 저장소 |
| `tokio` 1 + `tokio-util` 0.7 | 비동기 + CancellationToken |
| `tempfile` 3 (dev) | 테스트 임시 디렉토리 |
