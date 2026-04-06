import { t } from "@/i18n";
import { Button } from "@/components/ui/Button";
import { Card, CardContent } from "@/components/ui/card";
import { X, Monitor, ArrowRightLeft, Shield } from "lucide-react";

interface Props {
  onClose: () => void;
}

function Section({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <h3 className="text-sm font-semibold flex items-center gap-2">
        {icon}
        {title}
      </h3>
      <Card>
        <CardContent className="p-4 text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
          {children}
        </CardContent>
      </Card>
    </div>
  );
}

function ForwardTag({ label, color }: { label: string; color: string }) {
  return (
    <span className={`inline-block text-xs font-medium px-1.5 py-0.5 rounded ${color}`}>
      {label}
    </span>
  );
}

export default function HelpDialog({ onClose }: Props) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="bg-background border border-border rounded-lg shadow-xl w-[560px] max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <h2 className="text-base font-semibold">{t("help.title")}</h2>
          <Button variant="ghost" size="icon-sm" onClick={onClose}>
            <X className="size-4" />
          </Button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
          {/* Getting Started */}
          <Section icon={<Monitor className="size-4" />} title={t("help.gettingStarted")}>
            {t("help.gettingStartedContent")}
          </Section>

          {/* Auth Methods */}
          <Section icon={<Shield className="size-4" />} title={t("help.authMethods")}>
            {t("help.authMethodsContent")}
          </Section>

          {/* Forwarding Types */}
          <div className="space-y-2">
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <ArrowRightLeft className="size-4" />
              {t("help.forwardingTypes")}
            </h3>

            <Card>
              <CardContent className="p-4 space-y-4">
                <div className="space-y-1.5">
                  <ForwardTag label={t("help.forwardingLocal")} color="bg-blue-500/15 text-blue-400" />
                  <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
                    {t("help.forwardingLocalDesc")}
                  </p>
                </div>
                <div className="space-y-1.5">
                  <ForwardTag label={t("help.forwardingRemote")} color="bg-orange-500/15 text-orange-400" />
                  <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
                    {t("help.forwardingRemoteDesc")}
                  </p>
                </div>
                <div className="space-y-1.5">
                  <ForwardTag label={t("help.forwardingDynamic")} color="bg-purple-500/15 text-purple-400" />
                  <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
                    {t("help.forwardingDynamicDesc")}
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-border flex justify-end">
          <Button variant="outline" size="sm" onClick={onClose}>
            {t("help.close")}
          </Button>
        </div>
      </div>
    </div>
  );
}
