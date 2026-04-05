# SSH Forwarder — UI Style Guide

## Design System

shadcn/ui 컨셉의 zinc 다크 테마 기반. Tailwind CSS v4 `@theme` 블록으로 정의.

## Color Palette

### Background Layers
| Token | Hex | Usage |
|-------|-----|-------|
| `background` | `#09090b` | 앱 전체 배경 |
| `card` | `#0f0f13` | 카드, 패널 배경 |
| `muted` | `#18181b` | 호버, 비활성 배경 |
| `sidebar` | `#0c0c10` | 사이드바 배경 |

### Text
| Token | Hex | Usage |
|-------|-----|-------|
| `foreground` | `#fafafa` | 주 텍스트 |
| `card-foreground` | `#fafafa` | 카드 내 텍스트 |
| `muted-foreground` | `#a1a1aa` | 보조 텍스트, 라벨 |
| `sidebar-foreground` | `#a1a1aa` | 사이드바 비선택 항목 |

### Accent & Status
| Token | Hex | Usage |
|-------|-----|-------|
| `accent` | `#6366f1` | 주 액션 (indigo) |
| `accent-hover` | `#818cf8` | 호버 상태 |
| `destructive` | `#ef4444` | 삭제, 에러 |
| `status-connected` | `#22c55e` | 연결됨 (green) |
| `status-disconnected` | `#52525b` | 끊김 (zinc) |
| `status-connecting` | `#eab308` | 연결 중 (yellow) |
| `status-error` | `#ef4444` | 에러 (red) |

### Border & Input
| Token | Hex | Usage |
|-------|-----|-------|
| `border` | `#27272a` | 카드/구분선 border |
| `input` | `#27272a` | 입력 필드 border |
| `ring` | `#6366f1` | focus ring |

## Typography

- **Font**: Inter, system-ui, -apple-system
- **Base size**: 13px
- **Line height**: 1.6

| Element | Size | Weight | Color |
|---------|------|--------|-------|
| Page title | text-lg (18px) | font-semibold | foreground |
| Section header | text-xs (12px) | font-medium, uppercase, tracking-wider | muted-foreground |
| Body text | text-sm (14px) | font-normal | foreground |
| Label | text-xs (12px) | font-medium | muted-foreground |
| Monospace | text-sm | font-mono | foreground |
| Caption | text-xs | font-normal | muted-foreground |

## Components

### Button
5개 variant, 3개 size:

| Variant | Style |
|---------|-------|
| `default` | indigo 배경, 흰 텍스트, shadow-sm |
| `secondary` | muted 배경, border |
| `destructive` | red/15 배경, red 텍스트, red border |
| `ghost` | 투명, 호버 시 muted 배경 |
| `outline` | 투명, border, 호버 시 muted 배경 |

| Size | Height |
|------|--------|
| `sm` | h-7, px-3, text-xs |
| `default` | h-8, px-4, text-sm |
| `icon` | h-8, w-8 |

공통: `rounded-md`, `active:scale-[0.97]`, `focus-visible:ring-2`

### Input / Select
- Height: `h-8`
- Border: `border-input`
- Background: `bg-background`
- Focus: `ring-2 ring-ring ring-offset-1`
- Label 위에 1.5 gap

### Toggle (Switch)
- Size: `h-5 w-9`
- On: `bg-accent`, Off: `bg-zinc-700`
- Thumb: `w-4 h-4 bg-white shadow-md`

### Card
- `rounded-lg border border-border bg-card`
- Padding: `p-4`
- 리스트 카드: `divide-y divide-border overflow-hidden`

## Layout

### 전체 구조
```
┌──────────┬────────────────────────┐
│ Sidebar  │ Top Bar (h-12, drag)   │
│ (w-56)   ├────────────────────────│
│          │ Content                 │
│          │ (max-w-2xl mx-auto p-6)│
│          │                        │
└──────────┴────────────────────────┘
```

### Sidebar
- Width: `w-56`
- Background: `bg-sidebar`
- Header: `h-12`, 앱 이름 + 추가 버튼
- Items: `px-3 py-2 rounded-md`, 호버 `bg-sidebar-hover`
- Active: `bg-sidebar-active text-foreground`
- Status dot: `w-1.5 h-1.5 rounded-full`

### Content Area
- Top bar: `h-12`, 오른쪽 설정 아이콘
- Content: `max-w-2xl mx-auto p-6 space-y-6`

## Spacing Rules

| Context | Value |
|---------|-------|
| 컨텐츠 영역 padding | `p-6` (24px) |
| 섹션 간 간격 | `space-y-6` (24px) |
| 카드 내부 padding | `p-4` (16px) |
| 폼 필드 간 간격 | `gap-4` (16px) |
| 라벨 → 입력 간격 | `gap-1.5` (6px) |
| 사이드바 아이템 간격 | `space-y-0.5` (2px) |

## Status Indicators

### Connection Status
- 사이드바: `w-1.5 h-1.5 rounded-full` 점
- 상태 카드: `w-2.5 h-2.5 rounded-full` 점 + 텍스트 라벨

### Forwarding Kind Badge
| Kind | Style |
|------|-------|
| Local (L) | `bg-blue-600/20 text-blue-400` |
| Remote (R) | `bg-amber-600/20 text-amber-400` |
| Dynamic (D) | `bg-purple-600/20 text-purple-400` |

## Error Display

- 인라인 에러: `rounded-lg border border-destructive/30 bg-destructive/10 text-sm text-destructive`
- 항상 `extractErrorMessage()` 유틸로 Tauri AppError 객체 파싱
