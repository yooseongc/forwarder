import type { ForwardingRule as Rule, ForwardingKind } from "@/types";
import { KIND_OPTIONS } from "@/types";
import { t } from "@/i18n";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/Select";
import { ArrowLeftRight, X } from "lucide-react";

interface Props {
  rule: Rule;
  onChange: (rule: Rule) => void;
  onRemove: () => void;
}

export default function ForwardingRuleRow({ rule, onChange, onRemove }: Props) {
  const isDynamic = rule.kind === "dynamic";
  const isRemote = rule.kind === "remote";
  const update = (partial: Partial<Rule>) => onChange({ ...rule, ...partial });

  // Labels change based on direction
  // Local:  app host bind → SSH tunnel → remote target
  // Remote: SSH server bind → SSH tunnel → app host target
  const leftLabel = isRemote ? t("forward.serverBind") : t("forward.localBind");
  const leftHint = isRemote ? t("forward.serverBindHint") : t("forward.localBindHint");
  const rightLabel = isRemote ? t("forward.localTarget") : t("forward.remoteTarget");
  const rightHint = isRemote ? t("forward.localTargetHint") : t("forward.remoteTargetHint");

  const swap = () => {
    update({
      bindAddress: rule.remoteHost || "127.0.0.1",
      bindPort: rule.remotePort,
      remoteHost: rule.bindAddress || "127.0.0.1",
      remotePort: rule.bindPort,
    });
  };

  return (
    <Card>
      <CardContent className="p-3 space-y-3">
        {/* Top row: enable + kind + remove */}
        <div className="flex items-center gap-3">
          <Switch checked={rule.enabled} onCheckedChange={(enabled) => update({ enabled })} />
          <div className="flex-1">
            <Select value={rule.kind} onValueChange={(v) => v && update({ kind: v as ForwardingKind })}>
              <SelectTrigger className="h-7 w-36"><SelectValue /></SelectTrigger>
              <SelectContent>
                {KIND_OPTIONS.map((o) => <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
          {!isDynamic && (
            <Button variant="ghost" size="icon-sm" onClick={swap} title={t("forward.swap")}>
              <ArrowLeftRight className="size-3.5" />
            </Button>
          )}
          <Button variant="ghost" size="icon-sm" onClick={onRemove} title={t("forward.remove")}>
            <X className="size-3.5" />
          </Button>
        </div>

        {/* Bind + Remote fields */}
        {isDynamic ? (
          <div className="grid grid-cols-[1fr_100px] gap-2">
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">{t("forward.socks5Bind")}</Label>
              <Input value={rule.bindAddress} onChange={(e) => update({ bindAddress: e.target.value })} placeholder="127.0.0.1" />
            </div>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">{t("forward.port")}</Label>
              <Input type="number" value={rule.bindPort || ""} onChange={(e) => update({ bindPort: parseInt(e.target.value) || 0 })} placeholder="1080" />
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-[1fr_auto_1fr] gap-2 items-end">
            {/* Left: bind */}
            <div className="space-y-1">
              <div className="flex items-baseline justify-between">
                <Label className="text-xs text-muted-foreground">{leftLabel}</Label>
                <span className="text-[10px] text-muted-foreground/60">{leftHint}</span>
              </div>
              <div className="grid grid-cols-[1fr_80px] gap-1.5">
                <Input value={rule.bindAddress} onChange={(e) => update({ bindAddress: e.target.value })} placeholder="127.0.0.1" />
                <Input type="number" value={rule.bindPort || ""} onChange={(e) => update({ bindPort: parseInt(e.target.value) || 0 })} placeholder="8080" />
              </div>
            </div>

            {/* Arrow */}
            <div className="text-muted-foreground pb-2 text-xs">→</div>

            {/* Right: remote */}
            <div className="space-y-1">
              <div className="flex items-baseline justify-between">
                <Label className="text-xs text-muted-foreground">{rightLabel}</Label>
                <span className="text-[10px] text-muted-foreground/60">{rightHint}</span>
              </div>
              <div className="grid grid-cols-[1fr_80px] gap-1.5">
                <Input value={rule.remoteHost} onChange={(e) => update({ remoteHost: e.target.value })} placeholder="127.0.0.1" />
                <Input type="number" value={rule.remotePort || ""} onChange={(e) => update({ remotePort: parseInt(e.target.value) || 0 })} placeholder="3306" />
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
