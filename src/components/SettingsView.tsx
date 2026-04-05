import { useEffect, useState } from "react";
import { extractErrorMessage } from "../types";
import { api } from "../hooks/useTauri";
import Button from "./ui/Button";
import Toggle from "./ui/Toggle";

export default function SettingsView() {
  const [autostart, setAutostart] = useState(false);
  const [loading, setLoading] = useState(true);
  const [msg, setMsg] = useState<{ text: string; error: boolean } | null>(null);

  useEffect(() => {
    api.getAutostartEnabled()
      .then(setAutostart)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await api.setAutostartEnabled(enabled);
      setAutostart(enabled);
    } catch (e) {
      console.error("Failed to toggle autostart:", e);
    }
  };

  const handleExport = async () => {
    try {
      setMsg(null);
      const json = await api.exportConfig();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "forwarder-config.json";
      a.click();
      URL.revokeObjectURL(url);
      setMsg({ text: "설정을 내보냈습니다.", error: false });
    } catch (e) {
      setMsg({ text: `내보내기 실패: ${extractErrorMessage(e)}`, error: true });
    }
  };

  const handleImport = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        setMsg(null);
        const json = await file.text();
        await api.importConfig(json);
        setMsg({ text: "설정을 가져왔습니다. 새로고침하면 반영됩니다.", error: false });
      } catch (e) {
        setMsg({ text: `가져오기 실패: ${extractErrorMessage(e)}`, error: true });
      }
    };
    input.click();
  };

  if (loading) return null;

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-2xl mx-auto p-6 space-y-6">
        <h2 className="text-lg font-semibold text-foreground">설정</h2>

        {/* General */}
        <section className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">일반</h3>
          <div className="p-4 rounded-lg border border-border bg-card space-y-3">
            <Toggle
              label="Windows 시작 시 자동 실행"
              checked={autostart}
              onChange={toggleAutostart}
            />
            <p className="text-xs text-muted-foreground ml-11">
              활성화하면 Windows 로그인 시 트레이에서 자동으로 시작됩니다.
            </p>
          </div>
        </section>

        {/* Data */}
        <section className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">데이터</h3>
          <div className="p-4 rounded-lg border border-border bg-card space-y-3">
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={handleExport}>설정 내보내기</Button>
              <Button variant="outline" size="sm" onClick={handleImport}>설정 가져오기</Button>
            </div>
            <p className="text-xs text-muted-foreground">
              프로파일 설정을 JSON 파일로 내보내거나 가져올 수 있습니다. 비밀번호는 포함되지 않습니다.
            </p>
            {msg && (
              <p className={`text-xs ${msg.error ? "text-destructive" : "text-status-connected"}`}>
                {msg.text}
              </p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
