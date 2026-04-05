import { useEffect, useState } from "react";
import { extractErrorMessage } from "@/types";
import { api } from "@/hooks/useTauri";
import { useTheme, type Theme } from "@/hooks/useTheme";
import { Button } from "@/components/ui/Button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { useLocale, type Locale } from "@/i18n";
import { Download, Upload, Sun, Moon, Monitor, ArrowLeft, Globe } from "lucide-react";

const themeOptions: { value: Theme; label: string; icon: React.ReactNode }[] = [
  { value: "light", label: "라이트", icon: <Sun className="size-4" /> },
  { value: "dark", label: "다크", icon: <Moon className="size-4" /> },
  { value: "system", label: "시스템", icon: <Monitor className="size-4" /> },
];

interface Props {
  onClose?: () => void;
}

export default function SettingsView({ onClose }: Props) {
  const [autostart, setAutostart] = useState(false);
  const [loading, setLoading] = useState(true);
  const [msg, setMsg] = useState<{ text: string; error: boolean } | null>(null);
  const { theme, setTheme } = useTheme();
  const { locale, setLocale } = useLocale();

  useEffect(() => {
    api.getAutostartEnabled().then(setAutostart).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const toggleAutostart = async (enabled: boolean) => {
    try { await api.setAutostartEnabled(enabled); setAutostart(enabled); }
    catch (e) { console.error("Failed to toggle autostart:", e); }
  };

  const handleExport = async () => {
    try {
      setMsg(null);
      const json = await api.exportConfig();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a"); a.href = url; a.download = "forwarder-config.json"; a.click();
      URL.revokeObjectURL(url);
      setMsg({ text: "설정을 내보냈습니다.", error: false });
    } catch (e) { setMsg({ text: `내보내기 실패: ${extractErrorMessage(e)}`, error: true }); }
  };

  const handleImport = () => {
    const input = document.createElement("input"); input.type = "file"; input.accept = ".json";
    input.onchange = async () => {
      const file = input.files?.[0]; if (!file) return;
      try {
        setMsg(null);
        await api.importConfig(await file.text());
        setMsg({ text: "설정을 가져왔습니다.", error: false });
      } catch (e) { setMsg({ text: `가져오기 실패: ${extractErrorMessage(e)}`, error: true }); }
    };
    input.click();
  };

  if (loading) return null;

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-2xl mx-auto p-6 space-y-5">
        <div className="flex items-center gap-3">
          {onClose && (
            <Button variant="ghost" size="icon-sm" onClick={onClose}>
              <ArrowLeft className="size-4" />
            </Button>
          )}
          <h2 className="text-lg font-semibold">설정</h2>
        </div>

        {/* Theme */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">테마</h3>
          <Card>
            <CardContent className="p-4">
              <div className="flex gap-2">
                {themeOptions.map((opt) => (
                  <Button
                    key={opt.value}
                    variant={theme === opt.value ? "default" : "outline"}
                    size="sm"
                    onClick={() => setTheme(opt.value)}
                  >
                    {opt.icon}
                    {opt.label}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Language */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">언어 / Language</h3>
          <Card>
            <CardContent className="p-4">
              <div className="flex gap-2">
                {([
                  { value: "ko" as Locale, label: "한국어" },
                  { value: "en" as Locale, label: "English" },
                ]).map((opt) => (
                  <Button
                    key={opt.value}
                    variant={locale === opt.value ? "default" : "outline"}
                    size="sm"
                    onClick={() => { setLocale(opt.value); window.location.reload(); }}
                  >
                    <Globe className="size-4" />
                    {opt.label}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* General */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">일반</h3>
          <Card>
            <CardContent className="p-4 space-y-3">
              <div className="flex items-center gap-3">
                <Switch checked={autostart} onCheckedChange={toggleAutostart} />
                <Label>Windows 시작 시 자동 실행</Label>
              </div>
              <p className="text-xs text-muted-foreground pl-11">
                활성화하면 Windows 로그인 시 트레이에서 자동 시작됩니다.
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Data */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">데이터</h3>
          <Card>
            <CardContent className="p-4 space-y-3">
              <div className="flex gap-2">
                <Button variant="outline" size="sm" onClick={handleExport}><Download /> 내보내기</Button>
                <Button variant="outline" size="sm" onClick={handleImport}><Upload /> 가져오기</Button>
              </div>
              <p className="text-xs text-muted-foreground">
                프로파일 설정을 JSON으로 내보내거나 가져옵니다. 비밀번호는 포함되지 않습니다.
              </p>
              {msg && (
                <p className={`text-xs ${msg.error ? "text-destructive" : "text-green-400"}`}>{msg.text}</p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
