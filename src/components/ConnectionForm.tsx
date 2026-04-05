import { useEffect, useState } from "react";
import type { AuthMethod, ConnectionProfile, ForwardingRule as Rule } from "@/types";
import { AUTH_OPTIONS, extractErrorMessage, getKeyPath, hasKeyFile, needsPassword, newForwardingRule } from "@/types";
import { api } from "@/hooks/useTauri";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/Select";
import ForwardingRuleRow from "./ForwardingRule";
import { Eye, EyeOff, FolderOpen, Plus, Save, Trash2, X } from "lucide-react";
import { t } from "@/i18n";

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
      type === "password" ? { type: "password" }
      : type === "keyFile" ? { type: "keyFile", path: currentPath }
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
      <div className="max-w-2xl mx-auto p-6 space-y-5">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">
            {initial.name ? t("form.editConnection") : t("form.newConnection")}
          </h2>
          <div className="flex gap-1.5">
            <Button variant="outline" size="sm" onClick={onCancel}><X /> {t("action.cancel")}</Button>
            <Button size="sm" onClick={handleSave} disabled={!canSave}><Save /> {t("action.save")}</Button>
          </div>
        </div>

        {saveError && (
          <Card className="border-destructive/30 bg-destructive/10">
            <CardContent className="p-3 text-sm text-destructive">저장 실패: {saveError}</CardContent>
          </Card>
        )}

        {/* Server info */}
        <div className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">서버 정보</h3>
          <Card>
            <CardContent className="p-4 grid grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <Label htmlFor="name">이름</Label>
                <Input id="name" value={profile.name} onChange={(e) => update({ name: e.target.value })} placeholder="My Server" />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="username">사용자</Label>
                <Input id="username" value={profile.username} onChange={(e) => update({ username: e.target.value })} placeholder="root" />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="host">호스트</Label>
                <Input id="host" value={profile.host} onChange={(e) => update({ host: e.target.value })} placeholder="192.168.1.100" />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="port">포트</Label>
                <Input id="port" type="number" value={profile.port} onChange={(e) => update({ port: parseInt(e.target.value) || 22 })} />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Auth */}
        <div className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">인증</h3>
          <Card>
            <CardContent className="p-4 space-y-4">
              <div className="space-y-1.5">
                <Label>인증 방식</Label>
                <Select value={profile.authMethod.type} onValueChange={(v) => v && updateAuth(v)}>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {AUTH_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {showKey && (
                <div className="space-y-1.5">
                  <Label>키 파일 경로</Label>
                  <div className="flex gap-2">
                    <Input
                      className="flex-1"
                      value={getKeyPath(profile.authMethod)}
                      onChange={(e) => updateKeyPath(e.target.value)}
                      placeholder="C:\Users\...\.ssh\id_rsa"
                    />
                    <Button variant="outline" size="default" onClick={browseKeyFile}>
                      <FolderOpen /> 찾기
                    </Button>
                  </div>
                </div>
              )}

              {showPw && (
                <div className="space-y-2">
                  <Label>{profile.authMethod.type === "password" ? "비밀번호" : "키 파일 암호"}</Label>
                  <div className="relative">
                    <Input
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder={hasStoredPassword ? "저장됨 (변경 시 입력)" : "비밀번호 입력"}
                      className="pr-16"
                    />
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      className="absolute right-1.5 top-1/2 -translate-y-1/2"
                      onClick={() => setShowPassword(!showPassword)}
                    >
                      {showPassword ? <EyeOff /> : <Eye />}
                    </Button>
                  </div>
                  {hasStoredPassword && (
                    <Button
                      variant="destructive"
                      size="xs"
                      onClick={async () => { await api.deleteCredential(profile.id); setHasStoredPassword(false); }}
                    >
                      <Trash2 /> 저장된 비밀번호 삭제
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Forwarding rules */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">포워딩 규칙</h3>
            <Button variant="ghost" size="sm" onClick={() => update({ forwardingRules: [...profile.forwardingRules, newForwardingRule()] })}>
              <Plus /> 규칙 추가
            </Button>
          </div>
          {profile.forwardingRules.length === 0 ? (
            <Card className="border-dashed">
              <CardContent className="p-6 text-center text-sm text-muted-foreground">
                포워딩 규칙이 없습니다
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-2">
              {profile.forwardingRules.map((rule, i) => (
                <ForwardingRuleRow
                  key={rule.id}
                  rule={rule}
                  onChange={(r) => updateRule(i, r)}
                  onRemove={() => update({ forwardingRules: profile.forwardingRules.filter((_, j) => j !== i) })}
                />
              ))}
            </div>
          )}
        </div>

        {/* Options */}
        <div className="space-y-4">
          <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">옵션</h3>
          <Card>
            <CardContent className="p-4 flex items-center gap-3">
              <Switch checked={profile.autoConnect} onCheckedChange={(autoConnect) => update({ autoConnect })} />
              <Label>시작 시 자동 연결</Label>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
