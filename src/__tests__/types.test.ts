import { describe, expect, it } from "vitest";
import {
  statusLabel,
  statusColor,
  ruleDescription,
  getKeyPath,
  needsPassword,
  hasKeyFile,
  newProfile,
  newForwardingRule,
  type ConnectionStatus,
  type ForwardingRule,
} from "../types";

describe("statusLabel", () => {
  it("returns 연결됨 for connected", () => {
    expect(statusLabel("connected")).toBe("연결됨");
  });
  it("returns 연결 중... for connecting", () => {
    expect(statusLabel("connecting")).toBe("연결 중...");
  });
  it("returns 연결 안됨 for disconnected", () => {
    expect(statusLabel("disconnected")).toBe("연결 안됨");
  });
  it("returns error message for error status", () => {
    const status: ConnectionStatus = { error: { message: "timeout" } };
    expect(statusLabel(status)).toBe("오류: timeout");
  });
});

describe("statusColor", () => {
  it("returns correct Tailwind classes", () => {
    expect(statusColor("connected")).toBe("bg-status-connected");
    expect(statusColor("connecting")).toBe("bg-status-connecting");
    expect(statusColor("disconnected")).toBe("bg-status-disconnected");
    expect(statusColor({ error: { message: "x" } })).toBe("bg-status-error");
  });
});

describe("ruleDescription", () => {
  it("formats local rule", () => {
    const rule: ForwardingRule = {
      id: "1", kind: "local", bindAddress: "127.0.0.1", bindPort: 8080,
      remoteHost: "db", remotePort: 5432, enabled: true,
    };
    expect(ruleDescription(rule)).toBe("127.0.0.1:8080 → db:5432");
  });
  it("formats dynamic rule as SOCKS5", () => {
    const rule: ForwardingRule = {
      id: "2", kind: "dynamic", bindAddress: "0.0.0.0", bindPort: 1080,
      remoteHost: "", remotePort: 0, enabled: true,
    };
    expect(ruleDescription(rule)).toBe("0.0.0.0:1080 (SOCKS5)");
  });
});

describe("getKeyPath", () => {
  it("returns empty for password", () => {
    expect(getKeyPath({ type: "password" })).toBe("");
  });
  it("returns path for keyFile", () => {
    expect(getKeyPath({ type: "keyFile", path: "/key" })).toBe("/key");
  });
  it("returns path for keyFileWithPassphrase", () => {
    expect(getKeyPath({ type: "keyFileWithPassphrase", path: "/key2" })).toBe("/key2");
  });
});

describe("needsPassword", () => {
  it("true for password", () => expect(needsPassword({ type: "password" })).toBe(true));
  it("false for keyFile", () => expect(needsPassword({ type: "keyFile", path: "" })).toBe(false));
  it("true for keyFileWithPassphrase", () => expect(needsPassword({ type: "keyFileWithPassphrase", path: "" })).toBe(true));
});

describe("hasKeyFile", () => {
  it("false for password", () => expect(hasKeyFile({ type: "password" })).toBe(false));
  it("true for keyFile", () => expect(hasKeyFile({ type: "keyFile", path: "" })).toBe(true));
  it("true for keyFileWithPassphrase", () => expect(hasKeyFile({ type: "keyFileWithPassphrase", path: "" })).toBe(true));
});

describe("newProfile", () => {
  it("creates profile with defaults", () => {
    const p = newProfile();
    expect(p.id).toBeTruthy();
    expect(p.port).toBe(22);
    expect(p.autoConnect).toBe(false);
    expect(p.authMethod.type).toBe("password");
    expect(p.forwardingRules).toEqual([]);
  });
  it("generates unique IDs", () => {
    const ids = new Set(Array.from({ length: 10 }, () => newProfile().id));
    expect(ids.size).toBe(10);
  });
});

describe("newForwardingRule", () => {
  it("creates rule with defaults", () => {
    const r = newForwardingRule();
    expect(r.kind).toBe("local");
    expect(r.bindAddress).toBe("127.0.0.1");
    expect(r.enabled).toBe(true);
  });
});
