import { useState } from "react";
import { useConnections } from "@/hooks/useConnections";
import type { ConnectionProfile } from "@/types";
import { newProfile } from "@/types";
import { api } from "@/hooks/useTauri";
import ConnectionList from "./ConnectionList";
import ConnectionForm from "./ConnectionForm";
import ConnectionStatusView from "./ConnectionStatus";
import SettingsView from "./SettingsView";
import { Settings } from "lucide-react";
import { t } from "@/i18n";

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
      if (view.type === "edit") setView({ type: "status", profileId: view.profile.id });
    } catch (e) { console.error("Failed to refresh after save:", e); }
  };

  const handleDelete = async (profileId: string) => {
    if (!confirm(t("layout.confirmDelete"))) return;
    try {
      await api.deleteProfile(profileId);
      await refresh();
      setView({ type: "empty" });
    } catch (e) { console.error("Delete failed:", e); }
  };

  const handleCancel = () => {
    if (view.type === "edit") {
      const existing = profiles.find((p) => p.id === view.profile.id);
      setView(existing ? { type: "status", profileId: existing.id } : { type: "empty" });
    }
  };

  if (loading) {
    return <div className="h-screen bg-background flex items-center justify-center text-muted-foreground text-sm">{t("layout.loading")}</div>;
  }

  const selectedId = view.type === "status" ? view.profileId : view.type === "edit" ? view.profile.id : null;
  const selectedProfile = selectedId ? profiles.find((p) => p.id === selectedId) : null;

  return (
    <div className="h-screen bg-background flex">
      <ConnectionList profiles={profiles} selectedId={selectedId} getStatus={getStatus} onSelect={handleSelect} onAdd={handleAdd} />

      <div className="flex-1 flex flex-col min-w-0">
        {/* Top bar */}
        <div className="h-12 border-b border-border flex items-center justify-end px-4" data-tauri-drag-region>
          <button
            onClick={() => setView({ type: "settings" })}
            className={`p-1.5 rounded-md transition-colors cursor-pointer ${
              view.type === "settings" ? "text-foreground bg-muted" : "text-muted-foreground hover:text-foreground hover:bg-muted"
            }`}
            title={t("settings.title")}
          >
            <Settings className="size-4" />
          </button>
        </div>

        {view.type === "empty" && (
          <div className="flex-1 flex items-center justify-center text-muted-foreground text-sm">
            {t("layout.emptyState")}
          </div>
        )}
        {view.type === "status" && selectedProfile && (
          <ConnectionStatusView
            profile={selectedProfile} status={getStatus(selectedProfile.id)}
            profileStatus={statuses.get(selectedProfile.id)}
            onEdit={() => handleEdit(selectedProfile.id)}
            onDelete={() => handleDelete(selectedProfile.id)}
            onRefresh={refresh}
          />
        )}
        {view.type === "edit" && (
          <ConnectionForm profile={view.profile} onSave={handleSave} onCancel={handleCancel} />
        )}
        {view.type === "settings" && <SettingsView onClose={() => setView({ type: "empty" })} />}
      </div>
    </div>
  );
}
