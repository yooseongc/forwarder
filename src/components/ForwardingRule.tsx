import type { ForwardingRule as Rule, ForwardingKind } from "@/types";
import { KIND_OPTIONS } from "@/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/Select";
import { X } from "lucide-react";

interface Props {
  rule: Rule;
  onChange: (rule: Rule) => void;
  onRemove: () => void;
}

export default function ForwardingRuleRow({ rule, onChange, onRemove }: Props) {
  const isDynamic = rule.kind === "dynamic";
  const update = (partial: Partial<Rule>) => onChange({ ...rule, ...partial });

  return (
    <Card>
      <CardContent className="p-3 flex items-start gap-3">
        <div className="pt-1">
          <Switch checked={rule.enabled} onCheckedChange={(enabled) => update({ enabled })} />
        </div>
        <div className="flex-1 grid grid-cols-[110px_1fr_80px] gap-3 items-end">
          <div className="space-y-1.5">
            <Label className="text-xs">유형</Label>
            <Select value={rule.kind} onValueChange={(v) => update({ kind: v as ForwardingKind })}>
              <SelectTrigger className="w-full"><SelectValue /></SelectTrigger>
              <SelectContent>
                {KIND_OPTIONS.map((o) => <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs">바인드 주소</Label>
            <Input value={rule.bindAddress} onChange={(e) => update({ bindAddress: e.target.value })} placeholder="127.0.0.1" />
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs">포트</Label>
            <Input type="number" value={rule.bindPort || ""} onChange={(e) => update({ bindPort: parseInt(e.target.value) || 0 })} placeholder="8080" />
          </div>
          {!isDynamic && (
            <>
              <div />
              <div className="space-y-1.5">
                <Label className="text-xs">리모트 호스트</Label>
                <Input value={rule.remoteHost} onChange={(e) => update({ remoteHost: e.target.value })} placeholder="127.0.0.1" />
              </div>
              <div className="space-y-1.5">
                <Label className="text-xs">포트</Label>
                <Input type="number" value={rule.remotePort || ""} onChange={(e) => update({ remotePort: parseInt(e.target.value) || 0 })} placeholder="3306" />
              </div>
            </>
          )}
          {isDynamic && (
            <div className="col-span-2 text-xs text-muted-foreground self-center">SOCKS5 프록시</div>
          )}
        </div>
        <Button variant="ghost" size="icon-sm" onClick={onRemove} className="mt-5 shrink-0">
          <X className="size-3.5" />
        </Button>
      </CardContent>
    </Card>
  );
}
