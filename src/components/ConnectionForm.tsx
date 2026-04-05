import { useEffect, useState } from "react";
import type { AuthMethod, ConnectionProfile, ForwardingRule as Rule } from "../types";
import { AUTH_OPTIONS, extractErrorMessage, getKeyPath, hasKeyFile, needsPassword, newForwardingRule } from "../types";
import { api } from "../hooks/useTauri";
import Button from "./ui/Button";
import Input from "./ui/Input";
import Select from "./ui/Select";
import Toggle from "./ui/Toggle";
import ForwardingRuleRow from "./ForwardingRule";

interface Props {
  profile: ConnectionProfile;
  onSave: (profile: ConnectionProfile) => void;
  onCancel: () => void;
}

export default function ConnectionForm({ profile: initial, onSave, onCancel }: Props) {
  const [profile, setProfile] = useState<ConnectionProfile>(initial);
  const [password, setPassword] = useState("");
  const [hasStoredPassword, setHasStoredPassword] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    setProfile(initial);
    setPassword("");
    setSaveError(null);
    api.hasCredential(initial.id).then(setHasStoredPassword).catch(() => {});
  }, [initial.id]);

  const update = (partial: Partial<ConnectionProfile>) =>
    setProfile((p) => ({ ...p, ...partial }));

  const updateAuth = (type: string) => {
    const currentPath = getKeyPath(profile.authMethod);
    const method: AuthMethod =
      type === "password"
        ? { type: "password" }
        : type === "keyFile"
          ? { type: "keyFile", path: currentPath }
          : { type: "keyFileWithPassphrase", path: currentPath };
    update({ authMethod: method });
    if (type === "keyFile") setPassword("");
  };

  const updateKeyPath = (path: string) => {
    const { type } = profile.authMethod;
    if (type === "keyFile") update({ authMethod: { type: "keyFile", path } });
    else if (type === "keyFileWithPassphrase") update({ authMethod: { type: "keyFileWithPassphrase", path } });
  };

  const browseKeyFile = async () => {
    const path = await api.openKeyFileDialog();
    if (path) updateKeyPath(path);
  };

  const updateRule = (idx: number, rule: Rule) => {
    const rules = [...profile.forwardingRules];
    rules[idx] = rule;
    update({ forwardingRules: rules });
  };

  const removeRule = (idx: number) => {
    update({ forwardingRules: profile.forwardingRules.filter((_, i) => i !== idx) });
  };

  const handleSave = async () => {
    try {
      setSaveError(null);
      await api.saveProfile(profile);
      if (password && needsPassword(profile.authMethod)) {
        await api.saveCredential(profile.id, password);
      }
      onSave(profile);
    } catch (e) {
      setSaveError(extractErrorMessage(e));
    }
  };

  const showPw = needsPassword(profile.authMethod);
  const showKey = hasKeyFile(profile.authMethod);
  const canSave = profile.name && profile.host && profile.username;

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-2xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-foreground">
            {initial.name ? "연결 편집" : "새 연결"}
          </h2>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={onCancel}>취소</Button>
            <Button size="sm" onClick={handleSave} disabled={!canSave}>저장</Button>
          </div>
        </div>

        {saveError && (
          <div className="px-4 py-3 rounded-lg border border-destructive/30 bg-destructive/10 text-sm text-destructive">
            저장 실패: {saveError}
          </div>
        )}

        {/* Server info */}
        <section className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">서버 정보</h3>
          <div className="grid grid-cols-2 gap-4">
            <Input label="이름" value={profile.name} onChange={(e) => update({ name: e.target.value })} placeholder="My Server" />
            <Input label="사용자" value={profile.username} onChange={(e) => update({ username: e.target.value })} placeholder="root" />
            <Input label="호스트" value={profile.host} onChange={(e) => update({ host: e.target.value })} placeholder="192.168.1.100" />
            <Input label="포트" type="number" value={profile.port} onChange={(e) => update({ port: parseInt(e.target.value) || 22 })} />
          </div>
        </section>

        {/* Auth */}
        <section className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">인증</h3>
          <Select
            label="인증 방식"
            options={[...AUTH_OPTIONS]}
            value={profile.authMethod.type}
            onChange={(e) => updateAuth(e.target.value)}
          />
          {showKey && (
            <div className="flex gap-2 items-end">
              <div className="flex-1">
                <Input
                  label="키 파일 경로"
                  value={getKeyPath(profile.authMethod)}
                  onChange={(e) => updateKeyPath(e.target.value)}
                  placeholder="C:\Users\...\.ssh\id_rsa"
                />
              </div>
              <Button variant="outline" size="sm" onClick={browseKeyFile}>찾기</Button>
            </div>
          )}
          {showPw && (
            <div className="space-y-2">
              <div className="relative">
                <Input
                  label={profile.authMethod.type === "password" ? "비밀번호" : "키 파일 암호"}
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder={hasStoredPassword ? "저장됨 (변경 시 입력)" : "비밀번호 입력"}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-7 text-xs text-muted-foreground hover:text-foreground cursor-pointer"
                >
                  {showPassword ? "숨김" : "표시"}
                </button>
              </div>
              {hasStoredPassword && (
                <button
                  className="text-xs text-destructive hover:text-destructive/80 cursor-pointer"
                  onClick={async () => {
                    await api.deleteCredential(profile.id);
                    setHasStoredPassword(false);
                  }}
                >
                  저장된 비밀번호 삭제
                </button>
              )}
            </div>
          )}
        </section>

        {/* Forwarding rules */}
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">포워딩 규칙</h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => update({ forwardingRules: [...profile.forwardingRules, newForwardingRule()] })}
            >
              + 규칙 추가
            </Button>
          </div>
          {profile.forwardingRules.length === 0 && (
            <p className="text-sm text-muted-foreground py-4 text-center border border-dashed border-border rounded-lg">
              포워딩 규칙이 없습니다
            </p>
          )}
          <div className="space-y-2">
            {profile.forwardingRules.map((rule, i) => (
              <ForwardingRuleRow
                key={rule.id}
                rule={rule}
                onChange={(r) => updateRule(i, r)}
                onRemove={() => removeRule(i)}
              />
            ))}
          </div>
        </section>

        {/* Options */}
        <section className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">옵션</h3>
          <div className="p-4 rounded-lg border border-border bg-card">
            <Toggle label="시작 시 자동 연결" checked={profile.autoConnect} onChange={(autoConnect) => update({ autoConnect })} />
          </div>
        </section>
      </div>
    </div>
  );
}
