import { useState } from "react";
import type { ConnectionProfile, ConnectionStatus as Status, ProfileStatus } from "../types";
import { AUTH_LABEL, KIND_LABEL, KIND_STYLE, extractErrorMessage, ruleDescription, statusColor, statusLabel } from "../types";
import { api } from "../hooks/useTauri";
import Button from "./ui/Button";

interface Props {
  profile: ConnectionProfile;
  status: Status;
  profileStatus: ProfileStatus | undefined;
  onEdit: () => void;
  onDelete: () => void;
  onRefresh: () => void;
}

export default function ConnectionStatusView({
  profile,
  status,
  profileStatus,
  onEdit,
  onDelete,
  onRefresh,
}: Props) {
  const isConnected = status === "connected";
  const isConnecting = status === "connecting";
  const [actionError, setActionError] = useState<string | null>(null);

  const handleToggle = async () => {
    try {
      setActionError(null);
      if (isConnected) {
        await api.disconnect(profile.id);
      } else {
        await api.connect(profile.id);
      }
    } catch (e) {
      setActionError(extractErrorMessage(e));
    }
    onRefresh();
  };

  const handleReconnect = async () => {
    try {
      setActionError(null);
      await api.reconnect(profile.id);
    } catch (e) {
      setActionError(extractErrorMessage(e));
    }
    onRefresh();
  };

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-2xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-lg font-semibold text-foreground">{profile.name}</h2>
            <p className="text-sm text-muted-foreground mt-0.5">
              {profile.username}@{profile.host}:{profile.port}
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={onEdit}>편집</Button>
            <Button variant="destructive" size="sm" onClick={onDelete}>삭제</Button>
          </div>
        </div>

        {/* Status card */}
        <div className="flex items-center gap-3 p-4 rounded-lg border border-border bg-card">
          <span className={`w-2.5 h-2.5 rounded-full ${statusColor(status)}`} />
          <span className="text-sm text-card-foreground flex-1">{statusLabel(status)}</span>
          {isConnected && (
            <Button variant="outline" size="sm" onClick={handleReconnect} disabled={isConnecting}>
              재연결
            </Button>
          )}
          <Button
            variant={isConnected ? "destructive" : "default"}
            size="sm"
            onClick={handleToggle}
            disabled={isConnecting}
          >
            {isConnecting ? "연결 중..." : isConnected ? "연결 해제" : "연결"}
          </Button>
        </div>

        {/* Error */}
        {actionError && (
          <div className="px-4 py-3 rounded-lg border border-destructive/30 bg-destructive/10 text-sm text-destructive">
            {actionError}
          </div>
        )}

        {/* Tunnel statuses */}
        {profile.forwardingRules.length > 0 && (
          <div className="space-y-2">
            <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">포워딩 규칙</h3>
            <div className="rounded-lg border border-border divide-y divide-border overflow-hidden">
              {profile.forwardingRules.map((rule) => {
                const tunnelStatus = profileStatus?.tunnelStatuses.find((t) => t.ruleId === rule.id);
                return (
                  <div key={rule.id} className="flex items-center gap-3 px-4 py-2.5 bg-card">
                    <span className={`text-[11px] font-mono font-bold px-1.5 py-0.5 rounded ${KIND_STYLE[rule.kind]}`}>
                      {KIND_LABEL[rule.kind]}
                    </span>
                    <span className="text-sm text-card-foreground flex-1 font-mono">
                      {ruleDescription(rule)}
                    </span>
                    {!rule.enabled && <span className="text-xs text-muted-foreground">비활성</span>}
                    {rule.enabled && isConnected && (
                      <span className={`w-1.5 h-1.5 rounded-full ${tunnelStatus?.active ? "bg-status-connected" : "bg-status-disconnected"}`} />
                    )}
                    {tunnelStatus?.error && (
                      <span className="text-xs text-destructive truncate max-w-48">{tunnelStatus.error}</span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Info */}
        <div className="text-xs text-muted-foreground space-y-1">
          <p>인증: {AUTH_LABEL[profile.authMethod.type]}</p>
          {profile.autoConnect && <p>시작 시 자동 연결 활성화됨</p>}
        </div>
      </div>
    </div>
  );
}
