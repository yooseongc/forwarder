import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import ConnectionStatusView from "../../components/ConnectionStatus";
import type { ConnectionProfile } from "../../types";
import { t } from "../../i18n";

vi.mock("@tauri-apps/api/core");
const mockInvoke = vi.mocked(invoke);

const baseProfile: ConnectionProfile = {
  id: "p1",
  name: "Test Server",
  host: "192.168.1.1",
  port: 22,
  username: "admin",
  authMethod: { type: "password" },
  forwardingRules: [
    { id: "r1", kind: "local", bindAddress: "127.0.0.1", bindPort: 8080, remoteHost: "db", remotePort: 5432, enabled: true },
    { id: "r2", kind: "dynamic", bindAddress: "127.0.0.1", bindPort: 1080, remoteHost: "", remotePort: 0, enabled: false },
  ],
  autoConnect: false,
  autoReconnect: true,
};

const noop = () => {};

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ConnectionStatusView", () => {
  it("renders profile name and SSH info", () => {
    render(
      <ConnectionStatusView profile={baseProfile} status="disconnected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    expect(screen.getByText("Test Server")).toBeInTheDocument();
    expect(screen.getByText(/admin@192\.168\.1\.1:22/)).toBeInTheDocument();
  });

  it("shows connect button when disconnected", () => {
    render(
      <ConnectionStatusView profile={baseProfile} status="disconnected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    expect(screen.getByRole("button", { name: new RegExp(t("action.connect")) })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: new RegExp(t("action.reconnect")) })).not.toBeInTheDocument();
  });

  it("shows disconnect and reconnect buttons when connected", () => {
    render(
      <ConnectionStatusView profile={baseProfile} status="connected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    expect(screen.getByRole("button", { name: new RegExp(t("action.disconnect")) })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: new RegExp(t("action.reconnect")) })).toBeInTheDocument();
  });

  it("shows cancel button when connecting", () => {
    render(
      <ConnectionStatusView profile={baseProfile} status="connecting" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    expect(screen.getByRole("button", { name: new RegExp(t("action.cancel")) })).toBeInTheDocument();
  });

  it("calls api.connect on button click", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const onRefresh = vi.fn();
    render(
      <ConnectionStatusView profile={baseProfile} status="disconnected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={onRefresh} />
    );
    fireEvent.click(screen.getByRole("button", { name: new RegExp(t("action.connect")) }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("connect_profile", { id: "p1" });
      expect(onRefresh).toHaveBeenCalled();
    });
  });

  it("displays error message on connection failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Auth failed"));
    render(
      <ConnectionStatusView profile={baseProfile} status="disconnected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    fireEvent.click(screen.getByRole("button", { name: new RegExp(t("action.connect")) }));
    await waitFor(() => {
      expect(screen.getByText("Auth failed")).toBeInTheDocument();
    });
  });

  it("renders forwarding rules with kind labels", () => {
    render(
      <ConnectionStatusView profile={baseProfile} status="connected" profileStatus={undefined} onEdit={noop} onDelete={noop} onRefresh={noop} />
    );
    expect(screen.getByText("L")).toBeInTheDocument();
    expect(screen.getByText("D")).toBeInTheDocument();
    expect(screen.getByText(t("forward.disabled"))).toBeInTheDocument();
  });
});
