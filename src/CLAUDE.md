# CLAUDE.md — React Frontend (src/)

## Build & Test

```bash
npm run dev          # Vite 개발 서버 (http://localhost:1420)
npm run build        # 프로덕션 빌드
npx tsc --noEmit     # 타입 체크
npm test             # Vitest (43개)
npm run test:watch   # 감시 모드
```

## Tech Stack

- **React 19** + TypeScript + Vite
- **Tailwind CSS v4** (@theme 블록, Vite 플러그인)
- **shadcn/ui** — Button, Input, Select, Switch, Card, Badge, Label, Separator
- **Radix / base-ui** primitives (shadcn 내부)
- **lucide-react** — 아이콘
- **Vitest** + @testing-library/react
- **Path alias**: `@/*` → `src/*` (tsconfig + vite + vitest)

## Module Structure

### types/index.ts
- 데이터 모델: `ConnectionProfile`, `ForwardingRule`, `AuthMethod`, `ConnectionStatus`, `AppError`, `ErrorCode`
- 공유 상수: `AUTH_OPTIONS`, `KIND_OPTIONS`, `KIND_LABEL`, `KIND_STYLE`, `AUTH_LABEL`
- 팩토리: `newProfile()`, `newForwardingRule()`
- 유틸: `statusLabel()`, `statusColor()`, `ruleDescription()`, `getKeyPath()`, `needsPassword()`, `hasKeyFile()`, `extractErrorMessage()`

### i18n/
- `index.ts`: `t(key)` 함수, `useLocale()` 훅, localStorage 저장
- `ko.ts` / `en.ts`: 한국어/영어 메시지 (동일 키 구조)

### hooks/
- **useTauri.ts**: `api` 객체 — 모든 Tauri `invoke()` 래퍼 (18개 커맨드), `onStatusChange()` 이벤트 구독
- **useConnections.ts**: 프로파일 + 실시간 상태 관리 (`connection-status-changed` 이벤트 구독)
- **useTheme.ts**: 라이트/다크/시스템 테마 전환, `.dark` 클래스 토글, localStorage 저장

### components/
- **Layout.tsx**: 뷰 라우팅 (status / edit / settings / empty), 설정 버튼
- **ConnectionList.tsx**: 좌측 사이드바, 프로파일 목록 + 상태 인디케이터
- **ConnectionForm.tsx**: 프로파일 편집 폼 (서버 정보 / 인증 / 포워딩 규칙 / 옵션)
  - 비밀번호 표시/숨김 토글, 저장된 비밀번호 삭제 버튼
  - 인증 타입 변경 시 password 초기화
- **ConnectionStatus.tsx**: 연결 상태 뷰, 연결/해제/재연결/취소/Ping 버튼
- **ForwardingRule.tsx**: 포워딩 규칙 편집 행
  - Local/Remote에 따른 **동적 라벨** (로컬 바인드 ↔ 서버 바인드, 원격 대상 ↔ 로컬 대상)
  - **Swap 버튼** (바인드/대상 교체)
  - 힌트 텍스트 (어느 쪽 기준인지 표시)
- **SettingsView.tsx**: 테마 / 언어 / 자동시작 / 설정 내보내기/가져오기
- **ui/**: shadcn/ui 컴포넌트들

### lib/utils.ts
- `cn()`: clsx + tailwind-merge (shadcn 관례)

## Test Structure (43개)

| 파일 | 수 | 내용 |
|------|---|------|
| `__tests__/types.test.ts` | 19 | 유틸리티 함수, 팩토리, 상수 |
| `__tests__/hooks/useTauri.test.ts` | 10 | invoke 호출 검증, 이벤트 구독 |
| `__tests__/components/ConnectionStatus.test.tsx` | 7 | 상태별 렌더링, 버튼, 에러, i18n |
| `__tests__/components/ConnectionForm.test.tsx` | 7 | 폼 렌더링, 저장, 에러, i18n |

### 설정
- `vitest.config.ts`: happy-dom, globals, `@/*` alias, setup 파일
- `__tests__/setup.ts`: `@tauri-apps/api/core`, `@tauri-apps/api/event` 전역 mock

## Styling

- **Tailwind CSS v4**: `@import "tailwindcss"` + `@theme` 블록
- **테마 변수**: `:root` (라이트 기본) + `.dark` 오버라이드, 토큰 기반 (background/foreground/card/muted/accent/destructive/...)
- **다국어**: `t("key")` 함수로 ko/en 동적 전환 (`LocaleContext` 기반 re-render, 페이지 리로드 없음)
- **Tauri 드래그**: `[data-tauri-drag-region]` — 윈도우 드래그 영역
