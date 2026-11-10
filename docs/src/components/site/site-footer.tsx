"use client";

import { ArrowUpRight, Github } from "lucide-react";
import { LogoMark } from "./logo-mark";
import { EnesIcon } from "./enes-icon";
import { Container } from "./section";
import { useI18n } from "@/lib/i18n";

export function SiteFooter() {
  const { t } = useI18n();

  return (
    <footer className="relative mt-auto bg-transparent text-foreground dark:text-slate-200">
      <div className="pointer-events-none absolute inset-x-0 -top-28 bottom-0 overflow-hidden">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_50%_100%,rgba(18,113,255,0.18),transparent_70%)] dark:bg-[radial-gradient(ellipse_at_50%_100%,rgba(18,113,255,0.25),transparent_70%)]" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_80%_80%,rgba(107,74,255,0.14),transparent_60%)] dark:bg-[radial-gradient(ellipse_at_80%_80%,rgba(107,74,255,0.20),transparent_60%)]" />
        <div className="absolute inset-0 [background-image:radial-gradient(circle,rgba(140,100,255,0.18)_1px,transparent_1px)] [background-size:24px_24px] opacity-30 dark:opacity-45 [mask-image:radial-gradient(ellipse_at_center,black_30%,transparent_80%)]" />
        <div className="gradients-container absolute inset-0 opacity-80 dark:opacity-95 [mask-image:linear-gradient(to_top,black_85%,transparent_100%)]">
          <div className="g1" />
          <div className="g2" />
          <div className="g3" />
          <div className="g4" />
          <div className="g5" />
        </div>
      </div>

      <Container className="relative z-10 py-12">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_minmax(0,1fr)]">
          <div>
            <div className="flex items-center gap-2.5">
              <LogoMark className="h-7 w-7 shrink-0" />
              <span className="font-heading font-heading-expanded font-bold text-[20px] text-foreground dark:text-white tracking-tight">
                konvert
              </span>
            </div>
            <p className="mt-3 max-w-[340px] text-[13px] leading-relaxed text-muted-foreground dark:text-slate-300 font-sans">
              {t.footer.tagline}
            </p>
            <div className="mt-4 flex flex-wrap gap-1.5 font-mono text-[10.5px]">
              {t.footer.chips.map((c) => (
                <span
                  key={c}
                  className="rounded-[2px] bg-muted/80 text-foreground dark:bg-white/10 dark:text-white/90 px-2 py-0.5 backdrop-blur-xs"
                >
                  {c}
                </span>
              ))}
            </div>
          </div>
          <div>
            <p className="font-mono text-[11px] font-semibold text-muted-foreground dark:text-white/70 uppercase tracking-wider">
              {t.footer.navTitle}
            </p>
            <ul className="mt-3 space-y-2 font-mono text-[12.5px]">
              {t.footer.links.map((link) => (
                <li key={link.label}>
                  <a
                    href={link.href}
                    className="text-muted-foreground hover:text-foreground dark:text-slate-300 dark:hover:text-white transition-colors"
                  >
                    {link.label}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <p className="font-mono text-[11px] font-semibold text-muted-foreground dark:text-white/70 uppercase tracking-wider">
              {t.footer.devTitle}
            </p>
            <ul className="mt-3 space-y-2.5 font-mono text-[12.5px]">
              <li>
                <a
                  href="https://github.com/enesehs/konvert"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 text-muted-foreground hover:text-foreground dark:text-slate-300 dark:hover:text-white transition-colors"
                >
                  <Github className="h-4 w-4 shrink-0" aria-hidden="true" />
                  <span>github.com/enesehs/konvert</span>
                  <ArrowUpRight className="h-3 w-3 opacity-60" aria-hidden="true" />
                </a>
              </li>
              <li>
                <a
                  href="https://enesehs.dev"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 text-muted-foreground hover:text-foreground dark:text-slate-300 dark:hover:text-white transition-colors"
                >
                  <EnesIcon className="h-4 w-4 shrink-0" />
                  <span>enesehs.dev</span>
                  <ArrowUpRight className="h-3 w-3 opacity-60" aria-hidden="true" />
                </a>
              </li>
            </ul>
          </div>
        </div>
        <div className="mt-8 flex flex-col items-start justify-between gap-2 pt-6 pb-20 md:flex-row md:items-center md:pb-0 font-mono text-[11px] text-muted-foreground dark:text-slate-400">
          <p>{t.footer.copyright}</p>
          <p className="text-muted-foreground/80 dark:text-slate-400/80">
            {t.footer.standards}
          </p>
        </div>
      </Container>
    </footer>
  );
}
