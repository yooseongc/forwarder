# SSH Forwarder — UI Style Guide

## Foundation

- **Framework**: shadcn/ui (Radix + base-ui primitives)
- **CSS**: Tailwind CSS v4 (`@theme` 블록 + `.dark` 클래스)
- **Icons**: lucide-react
- **Font**: Geist Variable (shadcn 기본)

## Color Tokens

CSS 변수로 정의 (`:root` 라이트, `.dark` 오버라이드). Tailwind 클래스로 사용: `bg-background`, `text-foreground`, `border-border`, 등.

### Surface
| Token | Usage |
|-------|-------|
| `background` | 앱 전체 배경 |
| `card` | 카드 배경 |
| `muted` | 호버, 비활성 배경 |
| `sidebar` | 사이드바 배경 |
| `popover` | 팝오버/드롭다운 배경 |

### Text
| Token | Usage |
|-------|-------|
| `foreground` | 주 텍스트 |
| `muted-foreground` | 보조 텍스트, 라벨, 힌트 |
| `card-foreground` | 카드 내 텍스트 |

### Interactive
| Token | Usage |
|-------|-------|
| `primary` | 주 액션 버튼 |
| `secondary` | 보조 액션 |
| `accent` | 강조 (사이드바 active 등) |
| `destructive` | 삭제, 에러 |
| `border` | 카드/구분선 border |
| `input` | 입력 필드 border |
| `ring` | focus ring |

### Status (커스텀)
| Token | Color |
|-------|-------|
| `status-connected` | green-500 |
| `status-disconnected` | zinc-600 |
| `status-connecting` | yellow-500 |
| `status-error` | red-500 |

## Typography

- **Base**: 13px / line-height 1.6 / Geist Variable
- Page title: `text-lg font-semibold`
- Section header: `text-xs font-medium text-muted-foreground uppercase tracking-wider`
- Body: `text-sm text-foreground`
- Label: `text-xs font-medium text-muted-foreground`
- Caption/hint: `text-[10px] text-muted-foreground/60`

## Components (shadcn/ui)

### Button
Variants: `default` | `secondary` | `destructive` | `outline` | `ghost` | `link`
Sizes: `default` (h-8) | `sm` (h-7) | `xs` (h-6) | `lg` (h-9) | `icon` / `icon-sm` / `icon-xs` / `icon-lg`

### Input / Select
- `h-8`, `rounded-lg`, focus ring, destructive aria-invalid
- Label은 `space-y-1.5`로 상하 간격

### Switch
- `bg-accent` when checked, `bg-zinc-700` otherwise
- Thumb: white w-4 h-4

### Card
- `rounded-xl border bg-card`
- 리스트: `divide-y divide-border`

### Badge
- `outline` variant + custom color (`bg-blue-600/20 text-blue-400` 등)

## Layout

```
┌──────────┬────────────────────────────┐
│ Sidebar  │ Top Bar (h-12, drag)       │
│ (w-56)   ├────────────────────────────│
│          │ Content                    │
│          │ max-w-2xl mx-auto p-6      │
│          │ space-y-5 (섹션 간격)       │
└──────────┴────────────────────────────┘
```

- Sidebar width: `w-56`
- Top bar height: `h-12`, `data-tauri-drag-region`
- Content: `max-w-2xl mx-auto p-6 space-y-5`
- 섹션 카드: `rounded-xl border bg-card`, 내부 `p-4`

## Spacing

| 위치 | 값 |
|------|-----|
| 콘텐츠 padding | `p-6` |
| 섹션 간격 | `space-y-5` |
| 카드 내부 | `p-4` |
| 폼 필드 간격 | `gap-4` |
| Label → Input | `space-y-1.5` |
| 사이드바 아이템 | `space-y-0.5` |

## Status Indicators

- 사이드바 점: `w-2 h-2 rounded-full` + status color
- 상태 카드 점: `w-2.5 h-2.5 rounded-full`
- 터널 상태 점: `w-2 h-2`

## Forwarding Kind Badge
| Kind | Style |
|------|-------|
| L (Local) | `bg-blue-600/20 text-blue-400` |
| R (Remote) | `bg-amber-600/20 text-amber-400` |
| D (Dynamic) | `bg-purple-600/20 text-purple-400` |

## Error Display

```tsx
<Card className="border-destructive/30 bg-destructive/10">
  <CardContent className="p-3 text-sm text-destructive">
    {extractErrorMessage(error)}
  </CardContent>
</Card>
```

항상 `extractErrorMessage()`로 Tauri AppError 객체 파싱.

## i18n

- 모든 UI 문자열은 `t("key")` 함수로 렌더링
- 키는 `ko.ts`에 정의, `en.ts`에 대응
- 언어 변경 시 `window.location.reload()` (React 상태 동기화 위해)

## Theme

- `:root`에 라이트 변수, `.dark` 클래스에 다크 변수 오버라이드
- `useTheme()` 훅: `setTheme("light" | "dark" | "system")`
- 시스템 모드는 `prefers-color-scheme` 미디어 쿼리 감지
