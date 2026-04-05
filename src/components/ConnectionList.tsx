import type { ConnectionProfile, ConnectionStatus } from "@/types";
import { statusColor } from "@/types";
import { Button } from "@/components/ui/Button";
import { Plus } from "lucide-react";
import { t } from "@/i18n";

interface Props {
  profiles: ConnectionProfile[];
  selectedId: string | null;
  getStatus: (id: string) => ConnectionStatus;
  onSelect: (id: string) => void;
  onAdd: () => void;
}

export default function ConnectionList({ profiles, selectedId, getStatus, onSelect, onAdd }: Props) {
  return (
    <aside className="w-56 min-w-56 bg-sidebar border-r border-sidebar-border flex flex-col">
      <div className="h-12 px-4 flex items-center justify-between border-b border-sidebar-border" data-tauri-drag-region>
        <span className="text-sm font-semibold text-sidebar-foreground pointer-events-none tracking-tight">
          SSH Forwarder
        </span>
        <Button variant="ghost" size="icon-sm" onClick={onAdd} title="새 연결 추가">
          <Plus className="size-4" />
        </Button>
      </div>

      <nav className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        {profiles.length === 0 && (
          <p className="text-xs text-muted-foreground px-3 py-8 text-center">
            {t("layout.noProfiles")}
          </p>
        )}
        {profiles.map((p) => {
          const status = getStatus(p.id);
          const active = selectedId === p.id;
          return (
            <button
              key={p.id}
              onClick={() => onSelect(p.id)}
              className={`w-full text-left px-3 py-2 rounded-md flex items-center gap-2.5 transition-colors cursor-pointer
                ${active
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground"
                }`}
            >
              <span className={`w-2 h-2 rounded-full shrink-0 ${statusColor(status)}`} />
              <div className="min-w-0 flex-1">
                <div className="text-[13px] font-medium truncate">{p.name || "이름 없음"}</div>
                <div className="text-[11px] text-muted-foreground truncate">
                  {p.username}@{p.host}
                </div>
              </div>
              {p.autoConnect && (
                <span className="text-[10px] font-medium text-sidebar-primary/70 shrink-0 uppercase">auto</span>
              )}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
