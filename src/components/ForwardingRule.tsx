import type { ForwardingRule as Rule, ForwardingKind } from "../types";
import { KIND_OPTIONS } from "../types";
import Button from "./ui/Button";
import Input from "./ui/Input";
import Select from "./ui/Select";
import Toggle from "./ui/Toggle";

interface Props {
  rule: Rule;
  onChange: (rule: Rule) => void;
  onRemove: () => void;
}

export default function ForwardingRuleRow({ rule, onChange, onRemove }: Props) {
  const isDynamic = rule.kind === "dynamic";
  const update = (partial: Partial<Rule>) => onChange({ ...rule, ...partial });

  return (
    <div className="flex items-start gap-3 p-3 rounded-lg border border-border bg-card">
      <div className="pt-1">
        <Toggle checked={rule.enabled} onChange={(enabled) => update({ enabled })} />
      </div>
      <div className="flex-1 grid grid-cols-[100px_1fr_80px] gap-3 items-end">
        <Select
          label="유형"
          options={[...KIND_OPTIONS]}
          value={rule.kind}
          onChange={(e) => update({ kind: e.target.value as ForwardingKind })}
        />
        <Input
          label="바인드 주소"
          value={rule.bindAddress}
          onChange={(e) => update({ bindAddress: e.target.value })}
          placeholder="127.0.0.1"
        />
        <Input
          label="포트"
          type="number"
          value={rule.bindPort || ""}
          onChange={(e) => update({ bindPort: parseInt(e.target.value) || 0 })}
          placeholder="8080"
        />
        {!isDynamic && (
          <>
            <div /> {/* spacer for grid alignment */}
            <Input
              label="리모트 호스트"
              value={rule.remoteHost}
              onChange={(e) => update({ remoteHost: e.target.value })}
              placeholder="127.0.0.1"
            />
            <Input
              label="포트"
              type="number"
              value={rule.remotePort || ""}
              onChange={(e) => update({ remotePort: parseInt(e.target.value) || 0 })}
              placeholder="3306"
            />
          </>
        )}
        {isDynamic && (
          <div className="col-span-2 text-xs text-muted-foreground self-center pl-1">
            SOCKS5 프록시
          </div>
        )}
      </div>
      <Button variant="ghost" size="icon" onClick={onRemove} className="h-7 w-7 mt-5 shrink-0">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
          <line x1="2" y1="2" x2="10" y2="10" />
          <line x1="10" y1="2" x2="2" y2="10" />
        </svg>
      </Button>
    </div>
  );
}
