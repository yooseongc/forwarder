# CLAUDE.md — React Frontend (src/)

## Build & Test

```bash
npm run dev          # Vite 개발 서버 (http://localhost:1420)
npm run build        # 프로덕션 빌드 (dist/)
npx tsc --noEmit     # 타입 체크
npm test             # Vitest 테스트 실행 (44개)
npm run test:watch   # 감시 모드
```

## Module Structure

### types/index.ts — 타입 + 공유 상수
- 데이터 모델: `ConnectionProfile`, `ForwardingRule`, `AuthMethod`, `ConnectionStatus`, `AppError`, `ErrorCode`
- 공유 상수: `AUTH_OPTIONS`, `KIND_OPTIONS`, `KIND_LABEL`, `KIND_STYLE`, `AUTH_LABEL`
- 팩토리: `newProfile()`, `newForwardingRule()`
- 유틸리티: `statusLabel()`, `statusColor()`, `ruleDescription()`, `getKeyPath()`, `needsPassword()`, `hasKeyFile()`

### i18n/ — 다국어 지원
- `index.ts`: `t(key)` 함수, `setLocale()`, `getLocale()` — 타입 안전한 키
- `ko.ts`: 한국어 메시지 (기본)
- `en.ts`: 영어 메시지

### hooks/useTauri.ts — Tauri API 래퍼
- `api` 객체: 17개 커맨드 래핑 (CRUD, 연결, 재연결, 터널토글, 자격증명, 설정, import/export)
- `onStatusChange()`: `connection-status-changed` 이벤트 구독

### hooks/useConnections.ts — 연결 상태 관리
- `profiles`, `statuses` (Map), `getStatus()`, `refresh()`
- Tauri 이벤트로 실시간 상태 반영, `.catch()` 구독 실패 처리

### components/Layout.tsx — 메인 레이아웃
- 뷰 라우팅: `status` | `edit` | `settings` | `empty`
- try-catch 에러 처리 (handleSave, handleDelete)

### components/ConnectionForm.tsx — 프로파일 편집
- 인증 타입 변경 시 password 초기화
- 저장 실패 시 `saveError` 인라인 표시

### components/ConnectionStatus.tsx — 연결 상태 뷰
- 연결/해제/재연결 버튼
- `actionError` 인라인 에러 표시

### components/SettingsView.tsx — 설정 화면
- 자동 시작 토글
- 설정 내보내기/가져오기 (JSON 다운로드/업로드, 비밀번호 미포함)

### components/ui/ — 공통 UI
- `Button`, `Input`, `Select`, `Toggle`

## Test Structure (44개)

| 파일 | 수량 | 내용 |
|------|------|------|
| `types.test.ts` | 19 | 유틸리티 함수, 팩토리, 상수 |
| `hooks/useTauri.test.ts` | 10 | invoke 호출 검증, 이벤트 구독 |
| `components/ConnectionStatus.test.tsx` | 7 | 상태별 렌더링, 버튼, 에러 |
| `components/ConnectionForm.test.tsx` | 8 | 폼 렌더링, 저장, 에러, 규칙 추가 |

## Styling

- **Tailwind CSS v4**: `@import "tailwindcss"` + `@theme` (vite 플러그인)
- **다크 테마 전용**: `surface-0`~`3`, `accent`, `status-*`
- **Tauri 드래그**: `[data-tauri-drag-region]`
