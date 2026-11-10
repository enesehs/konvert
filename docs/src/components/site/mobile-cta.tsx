"use client";

import * as React from "react";
import { Terminal, X } from "lucide-react";
import { LogoMark } from "./logo-mark";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";

export function MobileCta() {
  const { t } = useI18n();
  const [visible, setVisible] = React.useState(false);
  const [dismissed, setDismissed] = React.useState(false);

  React.useEffect(() => {
    const onScroll = () => {
      setVisible(window.scrollY > 480);
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const show = visible && !dismissed;

  return (
    <div
      className={cn(
        "fixed inset-x-0 bottom-0 z-40 transition-transform duration-200 md:hidden",
        show ? "translate-y-0" : "translate-y-full"
      )}
      aria-hidden={!show}
    >
      <div className="flex items-center justify-between gap-3 border-t border-border bg-background/95 backdrop-blur-md px-4 py-2.5 pb-[max(0.625rem,env(safe-area-inset-bottom))] shadow-lg">
        <div className="flex min-w-0 items-center gap-2.5">
          <LogoMark className="h-5 w-5 shrink-0" />
          <div className="min-w-0 font-mono text-[11px]">
            <p className="font-bold text-foreground">konvert</p>
            <p className="truncate text-muted-foreground">{t.mobileCta.tagline}</p>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-1.5">
          <a
            href="#kurulum"
            className="inline-flex h-8 items-center gap-1.5 bg-brand px-3 font-mono text-[11.5px] font-semibold text-primary-foreground transition-colors hover:bg-brand-strong"
          >
            <Terminal className="h-3 w-3" aria-hidden="true" />
            {t.mobileCta.download}
          </a>
          <button
            type="button"
            aria-label={t.mobileCta.dismiss}
            onClick={() => setDismissed(true)}
            className="inline-flex h-8 w-8 items-center justify-center text-muted-foreground hover:text-foreground"
          >
            <X className="h-4 w-4" aria-hidden="true" />
          </button>
        </div>
      </div>
    </div>
  );
}
