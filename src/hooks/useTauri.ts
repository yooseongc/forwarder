import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConnectionProfile,
  ProfileStatus,
  StatusChangeEvent,
} from "../types";

export const api = {
  getProfiles: () => invoke<ConnectionProfile[]>("get_profiles"),
  saveProfile: (profile: ConnectionProfile) =>
    invoke("save_profile", { profile }),
  deleteProfile: (id: string) => invoke("delete_profile", { id }),
  connect: (id: string) => invoke("connect_profile", { id }),
  disconnect: (id: string) => invoke("disconnect_profile", { id }),
  getAllStatus: () => invoke<ProfileStatus[]>("get_all_status"),
  saveCredential: (profileId: string, password: string) =>
    invoke("save_credential", { profileId, password }),
  deleteCredential: (profileId: string) =>
    invoke("delete_credential", { profileId }),
  hasCredential: (profileId: string) =>
    invoke<boolean>("has_credential", { profileId }),
  getAutostartEnabled: () => invoke<boolean>("get_autostart_enabled"),
  setAutostartEnabled: (enabled: boolean) =>
    invoke("set_autostart_enabled", { enabled }),
  openKeyFileDialog: () => invoke<string | null>("open_key_file_dialog"),
  reconnect: (id: string) => invoke("reconnect_profile", { id }),
  enableTunnel: (profileId: string, ruleId: string) =>
    invoke("enable_tunnel", { profileId, ruleId }),
  exportConfig: () => invoke<string>("export_config"),
  importConfig: (json: string) => invoke("import_config", { json }),
};

export function onStatusChange(
  callback: (event: StatusChangeEvent) => void,
): Promise<UnlistenFn> {
  return listen<StatusChangeEvent>("connection-status-changed", (e) =>
    callback(e.payload),
  );
}
