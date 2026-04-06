import { useEffect, useState } from "react";
import type { ConnectionProfile, ConnectionStatus as Status, ProfileStatus } from "@/types";
import { AUTH_LABEL, KIND_LABEL, KIND_STYLE, extractErrorMessage, ruleDescription, statusColor, statusLabel } from "@/types";
import { api } from "@/hooks/useTauri";
import { Button } from "@/components/ui/Button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Pencil, Trash2, RefreshCw, Plug, Unplug, X, Activity } from "lucide-react";
import { t } from "@/i18n";

interface Props {
  profile: ConnectionProfile;
  status: Status;
  profileStatus: ProfileStatus | undefined;
  onEdit: () => void;
  onDelete: () => void;
  onRefresh: () => void;
}

export default function ConnectionStatusView({ profile, status, profileStatus, onEdit, onDelete, onRefresh }: Props) {
  const isConnected = status === "connected";
  const isConnecting = status === "connecting";
  const [actionError, setActionError] = useState<string | null>(null);
  const [pingResult, setPingResult] = useState<string | null>(null);

  useEffect(() => {
    setActionError(null);
    setPingResult(null);
  }, [profile.id]);

  const handleToggle = async () => {
    try {
      setActionError(null);
      if (isConnected) await api.disconnect(profile.id);
      else await api.connect(profile.id);
    } catch (e) { setActionError(extractErrorMessage(e)); }
    onRefresh();
  };

  const handlePing = async () => {
    try {
      setPingResult(null);
      const ms = await api.pingHost(profile.host, profile.port);
      setPingResult(`${ms}ms`);
    } catch (e) {
      setPingResult(extractErrorMessage(e));
    }
  };

  const handleCancel = async () => {
    try {
      setActionError(null);
      await api.disconnect(profile.id);
    } catch (e) { setActionError(extractErrorMessage(e)); }
    onRefresh();
  };

  const handleReconnect = async () => {
    try {
      setActionError(null);
      await api.reconnect(profile.id);
    } catch (e) { setActionError(extractErrorMessage(e)); }
    onRefresh();
  };

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-2xl mx-auto p-6 space-y-5">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-lg font-semibold">{profile.name}</h2>
            <div className="flex items-center gap-2 mt-0.5">
              <p className="text-sm text-muted-foreground">
                {profile.username}@{profile.host}:{profile.port}
              </p>
              <Button variant="ghost" size="icon-xs" onClick={handlePing} title="Ping">
                <Activity className="size-3" />
              </Button>
              {pingResult && (
                <span className={`text-xs ${pingResult.endsWith("ms") ? "text-green-400" : "text-destructive"}`}>
                  {pingResult}
                </span>
              )}
            </div>
          </div>
          <div className="flex gap-1.5">
            <Button variant="ghost" size="sm" onClick={onEdit}>
              <Pencil /> {t("action.edit")}
            </Button>
            <Button variant="destructive" size="sm" onClick={onDelete}>
              <Trash2 /> {t("action.delete")}
            </Button>
          </div>
        </div>

        {/* Status card */}
        <Card>
          <CardContent className="flex items-center gap-3 p-4">
            <span className={`w-2.5 h-2.5 rounded-full shrink-0 ${statusColor(status)}`} />
            <span className="text-sm flex-1">{statusLabel(status)}</span>
            {isConnected && (
              <Button variant="outline" size="sm" onClick={handleReconnect} disabled={isConnecting}>
                <RefreshCw /> {t("action.reconnect")}
              </Button>
            )}
            {isConnecting ? (
              <Button variant="outline" size="sm" onClick={handleCancel}>
                <X /> {t("action.cancel")}
              </Button>
            ) : (
              <Button
                variant={isConnected ? "destructive" : "default"}
                size="sm"
                onClick={handleToggle}
              >
                {isConnected ? <><Unplug /> {t("action.disconnect")}</> : <><Plug /> {t("action.connect")}</>}
              </Button>
            )}
          </CardContent>
        </Card>

        {/* Error */}
        {actionError && (
          <Card className="border-destructive/30 bg-destructive/10">
            <CardContent className="p-3 text-sm text-destructive">{actionError}</CardContent>
          </Card>
        )}

        {/* Tunnel statuses */}
        {profile.forwardingRules.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{t("form.forwardingRules")}</h3>
            <Card>
              <div className="divide-y divide-border">
                {profile.forwardingRules.map((rule) => {
                  const ts = profileStatus?.tunnelStatuses.find((t) => t.ruleId === rule.id);
                  return (
                    <div key={rule.id} className="flex items-center gap-3 px-4 py-2.5">
                      <Badge variant="outline" className={KIND_STYLE[rule.kind]}>
                        {KIND_LABEL[rule.kind]}
                      </Badge>
                      <code className="text-sm flex-1">{ruleDescription(rule)}</code>
                      {!rule.enabled && <span className="text-xs text-muted-foreground">{t("forward.disabled")}</span>}
                      {rule.enabled && isConnected && (
                        <span className={`w-2 h-2 rounded-full ${ts?.active ? "bg-status-connected" : "bg-status-disconnected"}`} />
                      )}
                      {ts?.error && <span className="text-xs text-destructive truncate max-w-48">{ts.error}</span>}
                    </div>
                  );
                })}
              </div>
            </Card>
          </div>
        )}

        <Separator />

        {/* Meta */}
        <div className="text-xs text-muted-foreground space-y-1">
          <p>{t("form.auth")}: {AUTH_LABEL[profile.authMethod.type]}</p>
          {profile.autoConnect && <p>{t("form.autoConnect")}</p>}
        </div>
      </div>
    </div>
  );
}
