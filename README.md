# SSH Forwarder

다수의 SSH 포트포워딩을 관리하는 Windows 데스크톱 앱.

![Tauri](https://img.shields.io/badge/Tauri-v2-blue)
![Rust](https://img.shields.io/badge/Rust-1.90-orange)
![React](https://img.shields.io/badge/React-19-61dafb)
![shadcn/ui](https://img.shields.io/badge/shadcn%2Fui-latest-black)

## Features

### 포워딩
- **Local (-L)** — 앱 호스트 → SSH 서버 → 원격 대상
- **Remote (-R)** — SSH 서버 → 앱 호스트 (역방향 프록시)
- **Dynamic (-D, SOCKS5)** — 동적 프록시
- 방향 전환 swap 버튼 + 동적 라벨 (로컬 바인드 / 서버 바인드 등)

### 연결 관리
- 다중 프로파일 관리
- 인증: 비밀번호 / SSH 키 파일 / 키 파일 + 패스프레이즈
- 비밀번호는 Windows Credential Manager에 저장 (설정 파일에 절대 저장 안 함)
- 연결 상태 실시간 모니터링 + 터널별 상태
- 10초 주기 health check, 연결 끊김 자동 감지
- 연결 중 취소, 재연결
- Ping (TCP 연결 테스트 + 응답 시간)

### UI / UX
- **shadcn/ui** 기반 모던한 UI
- 다크 / 라이트 / 시스템 테마 전환
- 다국어 지원 (한국어 / English)
- 비밀번호 표시/숨김 토글
- 구조화된 에러 메시지

### 시스템 통합
- **시스템 트레이** — 창 닫기 시 트레이 최소화, 더블클릭 복원
- **Single Instance** — 중복 실행 방지
- **자동 시작** — Windows 시작 시 트레이 모드 실행 (`--minimized`)
- **자동 연결** — 시작 시 지정된 프로파일 자동 연결
- 앱 종료 시 모든 세션 graceful shutdown

### 데이터
- 설정 내보내기 / 가져오기 (JSON, 비밀번호 미포함)
- 설정 파일 손상 시 자동 백업 + 복구

## Quick Start

```bash
# 의존성 설치
npm install

# 개발 서버
npm run tauri dev

# 릴리스 빌드
npm run tauri build

# 또는 태그 + 릴리스 업로드
./scripts/release.sh v1.0.0
```

## Testing

```bash
# Rust 유닛 테스트 (42개)
cd src-tauri && cargo test --lib

# Rust SSH 통합 테스트 (Docker 필요, 4개)
cd tests && docker compose up -d
cd src-tauri && cargo test --test ssh_integration
cd tests && docker compose down

# TypeScript 테스트 (43개)
npm test
```

### Docker SSH 테스트 서버

```bash
cd tests && docker compose up -d
```

| 서비스 | 주소 | 용도 |
|--------|------|------|
| SSH | `127.0.0.1:2222` | `testuser` / `testpass123` |
| HTTP | 컨테이너 내부 `:8080` | Local/SOCKS5 터널 검증 |
| Echo | `127.0.0.1:9999` | Raw TCP 터널 검증 |

### 테스트 시나리오

**Local forwarding**: 규칙 `127.0.0.1:18080 → 127.0.0.1:8080` 설정 후 브라우저에서 `http://127.0.0.1:18080`

**SOCKS5**: 규칙 `127.0.0.1:11080` (Dynamic) 설정 후
```bash
curl --socks5 127.0.0.1:11080 http://127.0.0.1:8080
```

**Reverse**: 앱 호스트에서 `python -m http.server 7777` 실행 후 규칙 `Server bind 0.0.0.0:17777 → Local 127.0.0.1:7777` 설정. Docker에서 확인:
```bash
docker exec tests-ssh-server-1 wget -qO- http://127.0.0.1:17777
```

## Architecture

```
┌─────────────────────────────────────────────┐
│  React + shadcn/ui (Tauri WebView)          │
│  Layout → ConnectionList / Form / Status    │
└──────────────┬──────────────────────────────┘
               │ Tauri IPC (invoke + events)
┌──────────────▼──────────────────────────────┐
│  Rust Backend                                │
│  commands.rs → state.rs → ssh/session.rs    │
│       │              ↓           ↓          │
│  config/store    credential    russh        │
│  (JSON file)    (keyring)    (SSH/SOCKS5)   │
└──────────────────────────────────────────────┘
```

자세한 설계는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) 참조.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Backend | Rust, russh 0.46, tokio, tokio-util |
| Frontend | React 19, TypeScript, Vite, Tailwind CSS v4 |
| UI | shadcn/ui, Radix primitives, lucide-react |
| Credential | Windows Credential Manager (`keyring`) |
| Test | cargo test, Vitest, @testing-library/react, Docker |

## Release

로컬 빌드 + GitHub Release 업로드:

```bash
./scripts/release.sh v1.0.0
```

1. `npm run tauri build` — `.msi` + `.exe` 생성
2. `git tag` + `git push origin v1.0.0`
3. `gh release create` — 빌드 결과물 업로드

## License

MIT
