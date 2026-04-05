# SSH Forwarder

다수의 SSH 포트포워딩을 관리하는 Windows 데스크톱 앱.

![Tauri](https://img.shields.io/badge/Tauri-v2-blue)
![Rust](https://img.shields.io/badge/Rust-1.90-orange)
![React](https://img.shields.io/badge/React-19-61dafb)

## Features

- **Local / Remote / Dynamic(SOCKS5)** 포워딩 지원
- **다중 프로파일** 관리 — 프로파일별 SSH 서버 + 포워딩 규칙
- **인증** — 비밀번호, SSH 키 파일, 키 파일 + 패스프레이즈
- **비밀번호 안전 저장** — Windows Credential Manager
- **시스템 트레이** — 창 닫기 시 트레이 최소화, 더블클릭 복원
- **자동 시작** — Windows 시작 시 트레이 모드 실행 + 프로파일 자동 연결
- **연결 상태 모니터링** — 실시간 상태 표시, 터널별 상태, 연결 끊김 감지
- **재연결** — 수동 재연결 버튼
- **Ping** — TCP 연결 테스트 (응답 시간 표시)
- **설정 내보내기/가져오기** — JSON (비밀번호 미포함)
- **다국어** — 한국어 / English
- **다크/라이트 테마** — 시스템 설정 따르기 지원
- **Single Instance** — 중복 실행 방지

## Development

```bash
# 의존성 설치
npm install

# 개발 서버
npm run tauri dev

# 테스트
cd src-tauri && cargo test --lib   # Rust (42개)
npm test                           # TypeScript (44개)

# 릴리스 빌드
npm run tauri build
```

### Docker SSH 테스트 서버

```bash
cd tests && docker compose up -d    # 시작 (포트 2222)
cd tests && docker compose down     # 정지
```

- SSH: `127.0.0.1:2222` / `testuser` / `testpass123`
- HTTP: 포트 8080 (컨테이너 내부, 터널 테스트용)
- Echo: 포트 9999

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Backend | Rust, russh, tokio |
| Frontend | React 19, TypeScript, Tailwind CSS v4, shadcn/ui |
| Credential | Windows Credential Manager (keyring) |
| Test | cargo test, Vitest, Docker |

## Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions가 자동으로 빌드하여 Release에 `.msi` / `.exe` 인스톨러를 첨부합니다.

## License

MIT
