"use client";

import * as React from "react";
import {
  Check,
  Copy,
  Terminal,
  RefreshCw,
  ExternalLink,
  ShieldCheck,
  Clock,
} from "lucide-react";
import { Container, Section } from "./section";
import { Reveal } from "./reveal";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useI18n } from "@/lib/i18n";

type InstallTabKey = "script" | "arch" | "debian" | "fedora" | "uninstall";

export function Install() {
  const { t } = useI18n();
  const [copiedKey, setCopiedKey] = React.useState<string | null>(null);

  const copyText = async (text: string, key: string) => {
    let ok = false;
    try {
      await navigator.clipboard.writeText(text);
      ok = true;
    } catch {
      try {
        const ta = document.createElement("textarea");
        ta.value = text;
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
      setCopiedKey(key);
      setTimeout(() => setCopiedKey(null), 2000);
    }
  };

  const tabs: Array<{ key: InstallTabKey; label: string; isUpcoming?: boolean }> = [
    { key: "script", label: t.install.tabs.script },
    { key: "arch", label: t.install.tabs.arch },
    { key: "debian", label: t.install.tabs.debian, isUpcoming: true },
    { key: "fedora", label: t.install.tabs.fedora, isUpcoming: true },
    { key: "uninstall", label: t.install.tabs.uninstall },
  ];

  return (
    <Section id="kurulum" className="relative border-b border-border bg-transparent">
      <Container>
        <Reveal>
          <div className="max-w-[720px]">
            <p className="font-mono text-[11px] font-semibold tracking-widest text-brand-strong uppercase">
              04 / {t.install.badge}
            </p>
            <h2 className="mt-3 font-heading font-heading-expanded text-[1.95rem] font-bold tracking-tight text-foreground sm:text-[2.35rem]">
              {t.install.title}
            </h2>
            <p className="mt-3 text-[15px] leading-relaxed text-muted-foreground sm:text-base">
              {t.install.subtitle}
            </p>
          </div>
        </Reveal>
        <div className="mt-10">
          <Tabs defaultValue="script" className="w-full">
            <Reveal delay={0.05}>
              <TabsList className="flex h-auto w-full flex-wrap justify-start gap-1 rounded-[2px] border border-border bg-muted/40 p-1">
                {tabs.map((tab) => (
                  <TabsTrigger
                    key={tab.key}
                    value={tab.key}
                    className="flex items-center gap-1.5 rounded-[2px] px-3.5 py-1.5 font-mono text-[12px] font-medium transition-colors data-[state=active]:bg-brand data-[state=active]:text-primary-foreground"
                  >
                    <Terminal className="h-3.5 w-3.5" aria-hidden="true" />
                    <span>{tab.label}</span>
                    {tab.isUpcoming && (
                      <span className="ml-1 rounded-[2px] border border-amber-500/40 bg-amber-500/15 px-1 py-0.2 font-mono text-[9px] font-bold text-amber-600 dark:text-amber-400">
                        {t.install.comingSoonBadge}
                      </span>
                    )}
                  </TabsTrigger>
                ))}
              </TabsList>
            </Reveal>

            {tabs.map((tab) => {
              const cmd = t.install.commands[tab.key];
              const caption = t.install.captions[tab.key];
              const isCopied = copiedKey === tab.key;

              return (
                <TabsContent key={tab.key} value={tab.key} className="mt-4 focus-visible:outline-none">
                  <div className="overflow-hidden rounded-[2px] border border-border bg-slate-100 text-slate-900 dark:bg-[#0d131a] dark:text-[#ccd6df] transition-colors">
                    <div className="flex h-8 items-center justify-between border-b border-border bg-slate-200/60 dark:bg-black/40 px-3.5 font-mono text-[11px] text-slate-700 dark:text-muted-foreground">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-slate-900 dark:text-white">bash</span>
                        <span>—</span>
                        <span>{tab.label}</span>
                        {tab.isUpcoming && (
                          <span className="flex items-center gap-1 text-[10px] text-amber-600 dark:text-amber-400 font-semibold">
                            <Clock className="h-3 w-3" aria-hidden="true" />
                            <span>{t.install.comingSoonBadge}</span>
                          </span>
                        )}
                      </div>

                      <button
                        type="button"
                        onClick={() => copyText(cmd, tab.key)}
                        aria-label="Komutu kopyala"
                        className="inline-flex items-center gap-1.5 border border-border bg-white text-slate-800 hover:bg-slate-50 dark:border-white/20 dark:bg-white/10 dark:text-white dark:hover:bg-white/20 px-2 py-0.5 font-mono text-[10.5px] transition-colors"
                      >
                        {isCopied ? (
                          <>
                            <Check className="h-3 w-3 text-emerald-500" aria-hidden="true" />
                            <span className="text-emerald-600 dark:text-emerald-400 font-semibold">Kopyalandı</span>
                          </>
                        ) : (
                          <>
                            <Copy className="h-3 w-3" aria-hidden="true" />
                            <span>Kopyala</span>
                          </>
                        )}
                      </button>
                    </div>
                    <div className="p-4">
                      <pre className="scroll-thin overflow-x-auto font-mono text-[12.5px] leading-relaxed">
                        <code>{cmd}</code>
                      </pre>
                    </div>
                    {caption && (
                      <div className="flex items-center gap-2 border-t border-border bg-slate-200/40 dark:bg-black/20 px-3.5 py-2 font-mono text-[11px] text-slate-600 dark:text-muted-foreground">
                        <ShieldCheck className="h-3.5 w-3.5 text-brand shrink-0" aria-hidden="true" />
                        <span>{caption}</span>
                      </div>
                    )}
                  </div>
                </TabsContent>
              );
            })}
          </Tabs>
        </div>
        <div className="mt-8 grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="flex flex-col justify-between rounded-[2px] border border-border bg-card p-5">
            <div>
              <div className="flex items-center gap-2 font-mono text-[11px] font-semibold text-brand-strong uppercase">
                <RefreshCw className="h-3.5 w-3.5" aria-hidden="true" />
                <span>{t.install.integrationTitle}</span>
              </div>
              <p className="mt-2 font-mono text-[12.5px] leading-relaxed text-muted-foreground">
                {t.install.integrationDesc}
              </p>
            </div>

            <div className="mt-4 flex items-center justify-between rounded-[2px] border border-border bg-slate-100 text-slate-900 dark:bg-[#0d131a] dark:text-[#ccd6df] px-3 py-2 font-mono text-[12px] transition-colors">
              <code>{t.install.integrationCmd}</code>
              <button
                type="button"
                onClick={() => copyText(t.install.integrationCmd, "integrate")}
                aria-label="Entegrasyon komutunu kopyala"
                className="inline-flex items-center gap-1 font-mono text-[11px] text-muted-foreground hover:text-foreground"
              >
                {copiedKey === "integrate" ? (
                  <Check className="h-3 w-3 text-emerald-500" aria-hidden="true" />
                ) : (
                  <Copy className="h-3 w-3" aria-hidden="true" />
                )}
              </button>
            </div>
          </div>
          <div className="flex flex-col justify-between rounded-[2px] border border-border bg-card p-5">
            <div>
              <div className="flex items-center justify-between font-mono text-[11px] font-semibold text-muted-foreground uppercase">
                <span>{t.install.sourceContribTitle}</span>
                <span>{t.install.licenseTag}</span>
              </div>
              <p className="mt-2 font-mono text-[12.5px] leading-relaxed text-muted-foreground">
                {t.install.sourceContribDesc}
              </p>
            </div>

            <div className="mt-4">
              <a
                href="https://github.com/enesehs/konvert"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 rounded-[2px] border border-border bg-muted/60 px-4 py-2 font-mono text-[12px] font-medium text-foreground hover:bg-muted transition-colors"
              >
                <span>{t.install.viewRepo}</span>
                <ExternalLink className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
              </a>
            </div>
          </div>
        </div>
      </Container>
    </Section>
  );
}
