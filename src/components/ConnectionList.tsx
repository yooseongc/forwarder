import type { ConnectionProfile, ConnectionStatus } from "../types";
import { statusColor } from "../types";
import Button from "./ui/Button";

interface Props {
  profiles: ConnectionProfile[];
  selectedId: string | null;
  getStatus: (id: string) => ConnectionStatus;
  onSelect: (id: string) => void;
  onAdd: () => void;
}

export default function ConnectionList({
  profiles,
  selectedId,
  getStatus,
  onSelect,
  onAdd,
}: Props) {
  return (
    <aside className="w-56 min-w-56 bg-sidebar border-r border-border flex flex-col">
      {/* Header */}
      <div className="h-12 px-4 flex items-center justify-between border-b border-border" data-tauri-drag-region>
        <span className="text-sm font-semibold text-foreground pointer-events-none tracking-tight">
          SSH Forwarder
        </span>
        <Button variant="ghost" size="icon" onClick={onAdd} title="새 연결 추가" className="h-7 w-7">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <line x1="7" y1="2" x2="7" y2="12" />
            <line x1="2" y1="7" x2="12" y2="7" />
          </svg>
        </Button>
      </div>

      {/* Profile list */}
      <nav className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        {profiles.length === 0 && (
          <p className="text-xs text-muted-foreground px-3 py-8 text-center">
            연결 프로파일이 없습니다
          </p>
        )}
        {profiles.map((p) => {
          const status = getStatus(p.id);
          const active = selectedId === p.id;
          return (
            <button
              key={p.id}
              onClick={() => onSelect(p.id)}
              className={`w-full text-left px-3 py-2 rounded-md flex items-center gap-2.5 transition-colors group
                ${active
                  ? "bg-sidebar-active text-foreground"
                  : "text-sidebar-foreground hover:bg-sidebar-hover hover:text-foreground"
                } cursor-pointer`}
            >
              <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${statusColor(status)}`} />
              <div className="min-w-0 flex-1">
                <div className="text-[13px] font-medium truncate">{p.name || "이름 없음"}</div>
                <div className="text-[11px] text-muted-foreground truncate">
                  {p.username}@{p.host}
                </div>
              </div>
              {p.autoConnect && (
                <span className="text-[10px] font-medium text-accent/60 shrink-0 uppercase">auto</span>
              )}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
