import { useState } from "react";
import { useConnections } from "../hooks/useConnections";
import type { ConnectionProfile } from "../types";
import { newProfile } from "../types";
import { api } from "../hooks/useTauri";
import ConnectionList from "./ConnectionList";
import ConnectionForm from "./ConnectionForm";
import ConnectionStatusView from "./ConnectionStatus";
import SettingsView from "./SettingsView";

type View =
  | { type: "status"; profileId: string }
  | { type: "edit"; profile: ConnectionProfile }
  | { type: "settings" }
  | { type: "empty" };

export default function Layout() {
  const { profiles, statuses, loading, refresh, getStatus } = useConnections();
  const [view, setView] = useState<View>({ type: "empty" });

  const handleSelect = (id: string) => setView({ type: "status", profileId: id });
  const handleAdd = () => setView({ type: "edit", profile: newProfile() });

  const handleEdit = (profileId: string) => {
    const profile = profiles.find((p) => p.id === profileId);
    if (profile) setView({ type: "edit", profile });
  };

  const handleSave = async () => {
    try {
      await refresh();
      if (view.type === "edit") {
        setView({ type: "status", profileId: view.profile.id });
      }
    } catch (e) {
      console.error("Failed to refresh after save:", e);
    }
  };

  const handleDelete = async (profileId: string) => {
    if (!confirm("이 연결 프로파일을 삭제하시겠습니까?")) return;
    try {
      await api.deleteProfile(profileId);
      await refresh();
      setView({ type: "empty" });
    } catch (e) {
      console.error("Delete failed:", e);
    }
  };

  const handleCancel = () => {
    if (view.type === "edit") {
      const existing = profiles.find((p) => p.id === view.profile.id);
      setView(existing ? { type: "status", profileId: existing.id } : { type: "empty" });
    }
  };

  if (loading) {
    return (
      <div className="h-screen bg-background flex items-center justify-center text-muted-foreground text-sm">
        로딩 중...
      </div>
    );
  }

  const selectedId =
    view.type === "status" ? view.profileId
    : view.type === "edit" ? view.profile.id
    : null;

  const selectedProfile = selectedId ? profiles.find((p) => p.id === selectedId) : null;

  return (
    <div className="h-screen bg-background flex">
      <ConnectionList
        profiles={profiles}
        selectedId={selectedId}
        getStatus={getStatus}
        onSelect={handleSelect}
        onAdd={handleAdd}
      />

      <div className="flex-1 flex flex-col min-w-0">
        {/* Top bar */}
        <div className="h-12 border-b border-border flex items-center justify-end px-4" data-tauri-drag-region>
          <button
            onClick={() => setView({ type: "settings" })}
            className={`p-1.5 rounded-md transition-colors cursor-pointer
              ${view.type === "settings" ? "text-accent bg-accent/10" : "text-muted-foreground hover:text-foreground hover:bg-muted"}`}
            title="설정"
          >
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <circle cx="8" cy="8" r="2.5" />
              <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.1 3.1l1.4 1.4M11.5 11.5l1.4 1.4M3.1 12.9l1.4-1.4M11.5 4.5l1.4-1.4" />
            </svg>
          </button>
        </div>

        {/* Content */}
        {view.type === "empty" && (
          <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm">
            연결을 선택하거나 새로 추가하세요
          </div>
        )}

        {view.type === "status" && selectedProfile && (
          <ConnectionStatusView
            profile={selectedProfile}
            status={getStatus(selectedProfile.id)}
            profileStatus={statuses.get(selectedProfile.id)}
            onEdit={() => handleEdit(selectedProfile.id)}
            onDelete={() => handleDelete(selectedProfile.id)}
            onRefresh={refresh}
          />
        )}

        {view.type === "edit" && (
          <ConnectionForm
            profile={view.profile}
            onSave={handleSave}
            onCancel={handleCancel}
          />
        )}

        {view.type === "settings" && <SettingsView />}
      </div>
    </div>
  );
}
