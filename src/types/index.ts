import { t } from "@/i18n";

// ── Data models (1:1 mapping with Rust types, camelCase) ──

export interface ConnectionProfile {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  authMethod: AuthMethod;
  forwardingRules: ForwardingRule[];
  autoConnect: boolean;
  autoReconnect: boolean;
}

export type AuthMethod =
  | { type: "password" }
  | { type: "keyFile"; path: string }
  | { type: "keyFileWithPassphrase"; path: string };

export interface ForwardingRule {
  id: string;
  kind: ForwardingKind;
  bindAddress: string;
  bindPort: number;
  remoteHost: string;
  remotePort: number;
  enabled: boolean;
}

export type ForwardingKind = "local" | "remote" | "dynamic";

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | { reconnecting: { attempt: number } }
  | { error: { message: string } };

export interface ProfileStatus {
  profileId: string;
  profileName: string;
  status: ConnectionStatus;
  tunnelStatuses: TunnelStatus[];
}

export interface TunnelStatus {
  ruleId: string;
  active: boolean;
  error: string | null;
}

export interface StatusChangeEvent {
  profileId: string;
  status: ConnectionStatus;
  tunnelStatuses: TunnelStatus[];
}

export type ErrorCode =
  | "PROFILE_NOT_FOUND"
  | "AUTH_FAILED"
  | "CONNECTION_FAILED"
  | "TUNNEL_BIND_FAILED"
  | "TUNNEL_UNSUPPORTED"
  | "CONFIG_ERROR"
  | "CREDENTIAL_ERROR"
  | "HOST_KEY_MISMATCH"
  | "INTERNAL";

export interface AppError {
  code: ErrorCode;
  message: string;
}

// ── Shared constants ──

export const AUTH_OPTIONS = [
  { value: "password", label: "비밀번호" },
  { value: "keyFile", label: "키 파일" },
  { value: "keyFileWithPassphrase", label: "키 파일 + 암호" },
] as const;

export const KIND_OPTIONS = [
  { value: "local", label: "Local (-L)" },
  { value: "remote", label: "Remote (-R)" },
  { value: "dynamic", label: "Dynamic (-D)" },
] as const;

export const KIND_LABEL: Record<ForwardingKind, string> = {
  local: "L",
  remote: "R",
  dynamic: "D",
};

export const KIND_STYLE: Record<ForwardingKind, string> = {
  local: "bg-blue-600/20 text-blue-400",
  remote: "bg-amber-600/20 text-amber-400",
  dynamic: "bg-purple-600/20 text-purple-400",
};

export const AUTH_LABEL: Record<AuthMethod["type"], string> = {
  password: "비밀번호",
  keyFile: "키 파일",
  keyFileWithPassphrase: "키 파일 + 암호",
};

// ── Factory functions ──

export function newProfile(): ConnectionProfile {
  return {
    id: crypto.randomUUID(),
    name: "",
    host: "",
    port: 22,
    username: "",
    authMethod: { type: "password" },
    forwardingRules: [],
    autoConnect: false,
    autoReconnect: true,
  };
}

export function newForwardingRule(): ForwardingRule {
  return {
    id: crypto.randomUUID(),
    kind: "local",
    bindAddress: "127.0.0.1",
    bindPort: 0,
    remoteHost: "127.0.0.1",
    remotePort: 0,
    enabled: true,
  };
}

// ── Status helpers ──

export function statusLabel(status: ConnectionStatus): string {
  if (status === "connected") return t("status.connected");
  if (status === "connecting") return t("status.connecting");
  if (status === "disconnected") return t("status.disconnected");
  if (typeof status === "object" && "reconnecting" in status)
    return `${t("status.reconnecting")} (${status.reconnecting.attempt}/5)`;
  if (typeof status === "object" && "error" in status)
    return `${t("status.error")}: ${status.error.message}`;
  return t("status.unknown");
}

export function statusColor(status: ConnectionStatus): string {
  if (status === "connected") return "bg-status-connected";
  if (status === "connecting") return "bg-status-connecting";
  if (status === "disconnected") return "bg-status-disconnected";
  if (typeof status === "object" && "reconnecting" in status) return "bg-status-connecting";
  return "bg-status-error";
}

export function ruleDescription(rule: ForwardingRule): string {
  if (rule.kind === "dynamic") {
    return `${rule.bindAddress}:${rule.bindPort} (SOCKS5)`;
  }
  return `${rule.bindAddress}:${rule.bindPort} → ${rule.remoteHost}:${rule.remotePort}`;
}

export function getKeyPath(auth: AuthMethod): string {
  if (auth.type === "keyFile" || auth.type === "keyFileWithPassphrase") {
    return auth.path;
  }
  return "";
}

export function needsPassword(auth: AuthMethod): boolean {
  return auth.type === "password" || auth.type === "keyFileWithPassphrase";
}

export function hasKeyFile(auth: AuthMethod): boolean {
  return auth.type === "keyFile" || auth.type === "keyFileWithPassphrase";
}

/** Extract readable error message from Tauri invoke errors (which may be AppError objects). */
export function extractErrorMessage(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  if (typeof e === "object" && e !== null) {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === "string") return obj.message;
  }
  return "Unknown error";
}
