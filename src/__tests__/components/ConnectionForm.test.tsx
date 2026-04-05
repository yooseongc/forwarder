import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import ConnectionForm from "../../components/ConnectionForm";
import { newProfile } from "../../types";
import type { ConnectionProfile } from "../../types";

vi.mock("@tauri-apps/api/core");
const mockInvoke = vi.mocked(invoke);

const emptyProfile = newProfile();

const filledProfile: ConnectionProfile = {
  id: "p1",
  name: "Test Server",
  host: "192.168.1.1",
  port: 22,
  username: "admin",
  authMethod: { type: "password" },
  forwardingRules: [],
  autoConnect: false,
};

const noop = () => {};

beforeEach(() => {
  vi.clearAllMocks();
  mockInvoke.mockResolvedValue(false); // hasCredential default
});

describe("ConnectionForm", () => {
  it("renders empty form for new profile", () => {
    render(<ConnectionForm profile={emptyProfile} onSave={noop} onCancel={noop} />);
    expect(screen.getByText("새 연결")).toBeInTheDocument();
  });

  it("renders edit form with profile name", () => {
    render(<ConnectionForm profile={filledProfile} onSave={noop} onCancel={noop} />);
    expect(screen.getByText("연결 편집")).toBeInTheDocument();
  });

  it("save button is disabled without required fields", () => {
    render(<ConnectionForm profile={emptyProfile} onSave={noop} onCancel={noop} />);
    const saveBtn = screen.getByText("저장");
    expect(saveBtn).toBeDisabled();
  });

  it("save button is enabled with required fields", () => {
    render(<ConnectionForm profile={filledProfile} onSave={noop} onCancel={noop} />);
    const saveBtn = screen.getByText("저장");
    expect(saveBtn).not.toBeDisabled();
  });

  it("cancel button calls onCancel", () => {
    const onCancel = vi.fn();
    render(<ConnectionForm profile={filledProfile} onSave={noop} onCancel={onCancel} />);
    fireEvent.click(screen.getByText("취소"));
    expect(onCancel).toHaveBeenCalled();
  });

  it("save calls api.saveProfile and onSave", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const onSave = vi.fn();
    render(<ConnectionForm profile={filledProfile} onSave={onSave} onCancel={noop} />);
    fireEvent.click(screen.getByText("저장"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_profile", expect.any(Object));
      expect(onSave).toHaveBeenCalled();
    });
  });

  it("shows error message when save fails", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "save_profile") throw new Error("Disk full");
      return false;
    });
    render(<ConnectionForm profile={filledProfile} onSave={noop} onCancel={noop} />);
    fireEvent.click(screen.getByText("저장"));
    await waitFor(() => {
      expect(screen.getByText(/Disk full/)).toBeInTheDocument();
    });
  });

  it("shows add rule button", () => {
    render(<ConnectionForm profile={filledProfile} onSave={noop} onCancel={noop} />);
    expect(screen.getByText("+ 규칙 추가")).toBeInTheDocument();
  });
});
