import { describe, expect, it, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { api, onStatusChange } from "../../hooks/useTauri";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/api/event");

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("api", () => {
  it("getProfiles calls correct command", async () => {
    mockInvoke.mockResolvedValue([]);
    await api.getProfiles();
    expect(mockInvoke).toHaveBeenCalledWith("get_profiles");
  });

  it("saveProfile passes profile argument", async () => {
    const profile = { id: "1", name: "test" } as Parameters<typeof api.saveProfile>[0];
    mockInvoke.mockResolvedValue(undefined);
    await api.saveProfile(profile);
    expect(mockInvoke).toHaveBeenCalledWith("save_profile", { profile });
  });

  it("connect passes id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.connect("abc");
    expect(mockInvoke).toHaveBeenCalledWith("connect_profile", { id: "abc" });
  });

  it("disconnect passes id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.disconnect("abc");
    expect(mockInvoke).toHaveBeenCalledWith("disconnect_profile", { id: "abc" });
  });

  it("reconnect passes id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.reconnect("abc");
    expect(mockInvoke).toHaveBeenCalledWith("reconnect_profile", { id: "abc" });
  });

  it("saveCredential passes profileId and password", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.saveCredential("p1", "secret");
    expect(mockInvoke).toHaveBeenCalledWith("save_credential", { profileId: "p1", password: "secret" });
  });

  it("hasCredential returns boolean", async () => {
    mockInvoke.mockResolvedValue(true);
    const result = await api.hasCredential("p1");
    expect(result).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("has_credential", { profileId: "p1" });
  });

  it("exportConfig returns JSON string", async () => {
    mockInvoke.mockResolvedValue('{"profiles":[]}');
    const result = await api.exportConfig();
    expect(result).toBe('{"profiles":[]}');
    expect(mockInvoke).toHaveBeenCalledWith("export_config");
  });

  it("importConfig passes json string", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.importConfig('{"profiles":[]}');
    expect(mockInvoke).toHaveBeenCalledWith("import_config", { json: '{"profiles":[]}' });
  });
});

describe("onStatusChange", () => {
  it("listens to correct event name", async () => {
    const unlisten = vi.fn();
    mockListen.mockResolvedValue(unlisten);

    const callback = vi.fn();
    const result = await onStatusChange(callback);

    expect(mockListen).toHaveBeenCalledWith("connection-status-changed", expect.any(Function));
    expect(result).toBe(unlisten);
  });
});
