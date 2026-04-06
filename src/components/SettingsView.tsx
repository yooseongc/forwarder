import { useEffect, useState } from "react";
import { extractErrorMessage } from "@/types";
import { api } from "@/hooks/useTauri";
import { useTheme, type Theme } from "@/hooks/useTheme";
import { Button } from "@/components/ui/Button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { t, useLocaleContext, type Locale } from "@/i18n";
import { Download, Upload, Sun, Moon, Monitor, ArrowLeft, Globe } from "lucide-react";

interface Props {
  onClose?: () => void;
}

export default function SettingsView({ onClose }: Props) {
  const [autostart, setAutostart] = useState(false);
  const [loading, setLoading] = useState(true);
  const [msg, setMsg] = useState<{ text: string; error: boolean } | null>(null);
  const { theme, setTheme } = useTheme();
  const { locale, setLocale } = useLocaleContext();

  const themeOptions: { value: Theme; label: string; icon: React.ReactNode }[] = [
    { value: "light", label: t("theme.light"), icon: <Sun className="size-4" /> },
    { value: "dark", label: t("theme.dark"), icon: <Moon className="size-4" /> },
    { value: "system", label: t("theme.system"), icon: <Monitor className="size-4" /> },
  ];

  useEffect(() => {
    api.getAutostartEnabled()
      .then(setAutostart)
      .catch((e) => console.error("Failed to get autostart:", e))
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
      setMsg({ text: t("settings.exported"), error: false });
    } catch (e) {
      setMsg({ text: `${t("settings.exportFailed")}: ${extractErrorMessage(e)}`, error: true });
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
        await api.importConfig(await file.text());
        setMsg({ text: t("settings.imported"), error: false });
      } catch (e) {
        setMsg({ text: `${t("settings.importFailed")}: ${extractErrorMessage(e)}`, error: true });
      }
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
          <h2 className="text-lg font-semibold">{t("settings.title")}</h2>
        </div>

        {/* Theme */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{t("theme.title")}</h3>
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
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{t("language.title")}</h3>
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
                    onClick={() => setLocale(opt.value)}
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
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{t("settings.general")}</h3>
          <Card>
            <CardContent className="p-4 space-y-3">
              <div className="flex items-center gap-3">
                <Switch checked={autostart} onCheckedChange={toggleAutostart} />
                <Label>{t("settings.autostart")}</Label>
              </div>
              <p className="text-xs text-muted-foreground pl-11">
                {t("settings.autostartDesc")}
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Data */}
        <div className="space-y-3">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{t("settings.data")}</h3>
          <Card>
            <CardContent className="p-4 space-y-3">
              <div className="flex gap-2">
                <Button variant="outline" size="sm" onClick={handleExport}><Download /> {t("settings.export")}</Button>
                <Button variant="outline" size="sm" onClick={handleImport}><Upload /> {t("settings.import")}</Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {t("settings.importExportDesc")}
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
