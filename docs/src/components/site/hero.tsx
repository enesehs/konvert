"use client";

import * as React from "react";
import { Check, Copy, Terminal, ExternalLink, ArrowRight, Github } from "lucide-react";
import { Container } from "./section";
import { ConvertDemo } from "./convert-demo";
import { Reveal } from "./reveal";
import { useI18n } from "@/lib/i18n";

function highlightManagers(text: string) {
  const parts = text.split(/(Dolphin|Nautilus|Nemo|Thunar)/g);
  return parts.map((part, i) => {
    if (["Dolphin", "Nautilus", "Nemo", "Thunar"].includes(part)) {
      return (
        <span
          key={i}
          className="font-semibold text-foreground dark:text-slate-100"
        >
          {part}
        </span>
      );
    }
    return part;
  });
}

export function Hero() {
  const { t } = useI18n();
  const [copied, setCopied] = React.useState(false);

  const copyCommand = async () => {
    const cmd = t.hero.oneLinerCommand;
    let ok = false;
    try {
      await navigator.clipboard.writeText(cmd);
      ok = true;
    } catch {
      try {
        const ta = document.createElement("textarea");
        ta.value = cmd;
        ta.setAttribute("readonly", "");
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        ok = document.execCommand("copy");
        document.body.removeChild(ta);
      } catch {
        ok = false;
      }
    }
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <section className="relative overflow-hidden border-b border-border bg-background pt-10 pb-16 sm:pt-14 sm:pb-20 lg:pt-16 lg:pb-24">
      <div className="dot-grid pointer-events-none absolute inset-0 opacity-40" aria-hidden="true" />

      <Container className="relative">
        <div className="grid grid-cols-1 items-center gap-12 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.05fr)] lg:gap-14">
          <div>
            <Reveal>
              <div className="inline-flex items-center gap-2 border border-border bg-muted/60 px-3 py-1 font-mono text-[11px] text-muted-foreground">
                <span className="h-1.5 w-1.5 bg-brand" aria-hidden="true" />
                <h1 className="font-heading font-heading-expanded font-semibold tracking-wider text-foreground uppercase">
                  {t.hero.seoH1}
                </h1>
              </div>
            </Reveal>
            <Reveal delay={0.05}>
              <div
                aria-hidden="true"
                className="mt-6 font-heading font-heading-expanded text-[2.75rem] font-bold leading-[0.98] tracking-[-0.03em] sm:text-[3.5rem] lg:text-[4rem] text-foreground"
              >
                {t.hero.displayTitle1}
                <br />
                <span className="text-brand-strong">{t.hero.displayTitle2}</span>
              </div>
            </Reveal>
            <Reveal delay={0.1}>
              <p className="mt-5 max-w-[540px] text-[15.5px] sm:text-[16.5px] leading-relaxed text-muted-foreground">
                {highlightManagers(t.hero.valueProposition)}
              </p>
            </Reveal>
            <Reveal delay={0.15}>
              <div className="mt-7 max-w-[540px]">
                <div className="flex items-center pb-2 font-mono text-[10.5px] tracking-wider text-muted-foreground uppercase">
                  <span className="flex items-center gap-1.5 font-semibold text-foreground">
                    <Terminal className="h-3.5 w-3.5 text-brand" aria-hidden="true" />
                    <span>{t.hero.oneLinerLabel}</span>
                  </span>
                </div>

                <div className="group relative flex items-center justify-between rounded-[2px] border border-border bg-slate-100 text-slate-900 dark:bg-[#0d131a] dark:text-[#ccd6df] p-1.5 pl-3.5 font-mono text-[13px] shadow-xs transition-all focus-within:border-brand">
                  <div className="flex items-center gap-2.5 overflow-x-auto scroll-thin py-1">
                    <span className="select-none font-bold text-brand-strong">$</span>
                    <code className="whitespace-nowrap font-mono">{t.hero.oneLinerCommand}</code>
                  </div>

                  <button
                    type="button"
                    onClick={copyCommand}
                    aria-label={copied ? t.hero.copied : t.hero.copy}
                    className="ml-3 inline-flex h-9 shrink-0 items-center gap-1.5 rounded-[2px] bg-brand text-primary-foreground hover:bg-brand-strong px-4 font-mono text-[12px] font-semibold transition-all cursor-pointer shadow-xs"
                  >
                    {copied ? (
                      <>
                        <Check className="h-3.5 w-3.5 text-emerald-300" aria-hidden="true" />
                        <span className="font-bold">{t.hero.copied}</span>
                      </>
                    ) : (
                      <>
                        <Copy className="h-3.5 w-3.5" aria-hidden="true" />
                        <span>{t.hero.copy}</span>
                      </>
                    )}
                  </button>
                </div>
                <div className="mt-2.5 flex items-center justify-between px-0.5 font-mono text-[11.5px] text-muted-foreground">
                  <a
                    href="#kurulum"
                    className="inline-flex items-center gap-1 text-muted-foreground hover:text-brand transition-colors"
                  >
                    <span>{t.hero.allInstallOptions}</span>
                    <ArrowRight className="h-3 w-3" aria-hidden="true" />
                  </a>

                  <a
                    href="https://github.com/enesehs/konvert"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <Github className="h-3 w-3" aria-hidden="true" />
                    <span>GitHub</span>
                    <ExternalLink className="h-2.5 w-2.5 opacity-60" aria-hidden="true" />
                  </a>
                </div>
              </div>
            </Reveal>
            <Reveal delay={0.2}>
              <div className="mt-8 max-w-[540px] border-t border-border pt-4">
                <div className="flex items-center justify-between font-mono text-[10.5px] font-semibold tracking-wider text-muted-foreground uppercase">
                  <span>{t.hero.supportedManagersTitle}</span>
                  <span>ENESEHS.DEV</span>
                </div>

                <div className="mt-2.5 grid grid-cols-2 gap-2 sm:grid-cols-4 font-mono text-[12px]">
                  {t.hero.supportedDesktops.map((d) => (
                    <div
                      key={d}
                      className="flex items-center gap-2 rounded-[2px] border border-border bg-card px-2.5 py-1.5 font-medium text-foreground shadow-xs"
                    >
                      <span className="h-1.5 w-1.5 rounded-full bg-brand shrink-0" />
                      <span className="truncate">{d}</span>
                    </div>
                  ))}
                </div>
              </div>
            </Reveal>
          </div>
          <Reveal delay={0.12} className="lg:pl-2">
            <ConvertDemo />
          </Reveal>
        </div>
      </Container>
    </section>
  );
}
