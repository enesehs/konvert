"use client";

import * as React from "react";
import {
  Terminal,
  FileImage,
  Sparkles,
  ChevronRight,
  Check,
  RotateCcw,
  Sparkle,
} from "lucide-react";
import { Container, Section } from "./section";
import { Reveal } from "./reveal";
import { useI18n } from "@/lib/i18n";
import { cn } from "@/lib/utils";

function RightClickEmulation() {
  const { t } = useI18n();

  const [step, setStep] = React.useState<number>(0);
  const [inView, setInView] = React.useState<boolean>(false);
  const containerRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setInView(true);
        }
      },
      { threshold: 0.25 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  React.useEffect(() => {
    if (!inView) return;

    const timings = [1100, 1000, 1200, 900, 700, 3800];
    const timer = setTimeout(() => {
      setStep((prev) => (prev + 1) % 6);
    }, timings[step]);

    return () => clearTimeout(timer);
  }, [step, inView]);

  const isConverting = step === 4;
  const isDone = step === 5;
  const showMainMenu = step >= 1 && step <= 3;
  const showSubMenu = step >= 2 && step <= 3;

  const cursorCoords: Record<number, { x: number; y: number; opacity: number; clicking: boolean }> = {
    0: { x: 35, y: 100, opacity: 0.8, clicking: false },
    1: { x: 135, y: 64, opacity: 1, clicking: true },
    2: { x: 170, y: 135, opacity: 1, clicking: false },
    3: { x: 325, y: 182, opacity: 1, clicking: true },
    4: { x: 360, y: 220, opacity: 0, clicking: false },
    5: { x: 360, y: 220, opacity: 0, clicking: false },
  };

  return (
    <div
      ref={containerRef}
      onClick={() => setStep(0)}
      className="relative flex h-full min-h-[460px] flex-col justify-between overflow-hidden rounded-[2px] border border-border bg-slate-50 dark:bg-[#0c111c] p-5 font-mono select-none transition-colors"
    >
      <div className="w-full overflow-x-auto sm:overflow-visible">
        <div className="relative min-w-[420px] flex-1 pb-4">
          <div className="mb-3 flex h-7 items-center justify-between border-b border-border pb-2 text-[11px] text-muted-foreground">
            <span>{t.workflow.folder}</span>
            <span className="text-[10px]">
              {isDone ? t.workflow.fileCount2 : t.workflow.fileCount1}
            </span>
          </div>

          <div className="space-y-2.5">
            <div
              className={cn(
                "flex h-[58px] items-center justify-between rounded-[2px] border px-3.5 transition-all duration-200",
                step >= 1 && step <= 4
                  ? "border-rose-500/50 bg-rose-500/10 shadow-xs"
                  : isDone
                    ? "border-rose-500/30 bg-rose-500/5 text-muted-foreground opacity-85"
                    : "border-border bg-card shadow-xs"
              )}
            >
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-[2px] border border-rose-500/30 bg-rose-500/10 text-rose-500">
                  <FileImage className="h-4 w-4" aria-hidden="true" />
                </div>
                <div>
                  <div className="flex items-center gap-2 text-[12.5px] font-semibold text-foreground">
                    <span>screenshot-01.png</span>
                    <span className="rounded-[2px] border border-rose-500/30 bg-rose-500/15 px-1 py-0.2 text-[9.5px] font-bold text-rose-600 dark:text-rose-400 uppercase">
                      {t.workflow.sourceTag}
                    </span>
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    33 MB · PNG Image (2560x1440)
                  </div>
                </div>
              </div>

              {isConverting && (
                <div className="flex items-center gap-1.5 text-[11px] text-brand-strong animate-pulse">
                  <Sparkle className="h-3 w-3 animate-spin" />
                  <span>{t.workflow.converting}</span>
                </div>
              )}
            </div>

            {isDone && (
              <div className="animate-in fade-in slide-in-from-top-3 duration-300 flex h-[58px] items-center justify-between rounded-[2px] border border-emerald-500/50 bg-emerald-500/10 px-3.5 shadow-xs">
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-[2px] border border-emerald-500/40 bg-emerald-500/20 text-emerald-500">
                    <FileImage className="h-4 w-4" aria-hidden="true" />
                  </div>
                  <div>
                    <div className="flex items-center gap-2 text-[12.5px] font-semibold text-foreground">
                      <span className="text-emerald-600 dark:text-emerald-400">screenshot-01.jxl</span>
                      <span className="rounded-[2px] border border-emerald-500/40 bg-emerald-500/20 px-1.5 py-0.2 text-[9.5px] font-bold text-emerald-600 dark:text-emerald-400">
                        {t.workflow.newTag}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 text-[11px] text-emerald-600/80 dark:text-emerald-400/80">
                      <span className="font-bold">26 MB</span>
                      <span>·</span>
                      <span>JPEG XL (2560x1440)</span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          {showMainMenu && (
            <div className="animate-in fade-in zoom-in-95 duration-150 absolute left-[105px] top-[60px] z-20 w-[176px] rounded-[2px] border border-border bg-card p-1 text-[11.5px] shadow-2xl">
              <div className="flex h-[26px] items-center px-2.5 text-muted-foreground hover:bg-muted">
                {t.workflow.menuOpen}
              </div>
              <div className="flex h-[26px] items-center px-2.5 text-muted-foreground hover:bg-muted">
                {t.workflow.menuCopy}
              </div>
              <div className="my-1 border-t border-border" />

              <div
                className={cn(
                  "relative flex h-[28px] items-center justify-between px-2.5 transition-colors",
                  step >= 2
                    ? "bg-brand text-primary-foreground font-semibold"
                    : "bg-muted/80 text-foreground"
                )}
              >
                <span className="flex items-center gap-1.5">
                  <Sparkles className="h-3 w-3 text-brand-strong group-hover:text-white" aria-hidden="true" />
                  <span>Konvert</span>
                </span>
                <ChevronRight className="h-3 w-3" aria-hidden="true" />

                {showSubMenu && (
                  <div className="animate-in fade-in zoom-in-95 duration-150 absolute left-[calc(100%-2px)] top-0 z-30 w-[140px] rounded-[2px] border border-border bg-card p-1 text-[11px] text-foreground shadow-2xl">
                    <div className="flex h-[24px] items-center justify-between px-2 text-muted-foreground hover:bg-muted">
                      <span>PNG</span>
                      <span className="text-[10px] opacity-70">.png</span>
                    </div>
                    <div className="flex h-[24px] items-center justify-between px-2 text-muted-foreground hover:bg-muted">
                      <span>JPG</span>
                      <span className="text-[10px] opacity-70">.jpg</span>
                    </div>

                    <div
                      className={cn(
                        "flex h-[26px] items-center justify-between px-2 transition-colors",
                        step === 3
                          ? "bg-brand text-primary-foreground font-bold shadow-xs"
                          : "bg-muted font-semibold text-brand-strong"
                      )}
                    >
                      <span>JXL</span>
                      <span className="text-[10px] font-mono">.jxl</span>
                    </div>

                    <div className="flex h-[24px] items-center justify-between px-2 text-muted-foreground hover:bg-muted">
                      <span>WEBP</span>
                      <span className="text-[10px] opacity-70">.jxl</span>
                    </div>
                    <div className="flex h-[24px] items-center justify-between px-2 text-muted-foreground hover:bg-muted">
                      <span>AVIF</span>
                      <span className="text-[10px] opacity-70">.avif</span>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {isDone && (
            <div className="animate-in fade-in slide-in-from-bottom-2 duration-200 absolute right-1 bottom-1 z-20 flex items-center gap-2.5 rounded-[2px] border border-border bg-slate-900 dark:bg-[#0d131a] px-3 py-2 text-[11px] text-white shadow-2xl">
              <Check className="h-3.5 w-3.5 text-emerald-400 shrink-0" aria-hidden="true" />
              <div>
                <p className="font-bold text-white">{t.workflow.doneToastTitle}</p>
                <p className="text-[10px] text-slate-300">{t.workflow.doneToastDetail}</p>
              </div>
            </div>
          )}

          <div
            style={{
              transform: `translate3d(${cursorCoords[step].x}px, ${cursorCoords[step].y}px, 0)`,
              opacity: cursorCoords[step].opacity,
            }}
            className="pointer-events-none absolute left-0 top-0 z-[100] transition-[transform,opacity] duration-600 ease-[cubic-bezier(0.22,1,0.36,1)]"
          >
            {cursorCoords[step].clicking && (
              <span
                key={`click-${step}`}
                className="animate-ping absolute -left-2 -top-2 h-6 w-6 rounded-full border border-brand bg-brand/30"
              />
            )}

            <svg
              className={cn(
                "h-5 w-5 drop-shadow-[0_2px_8px_rgba(0,0,0,0.55)] transition-transform duration-150 origin-top-left",
                cursorCoords[step].clicking ? "scale-85" : "scale-100"
              )}
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              aria-hidden="true"
            >
              <path
                d="M3 3L10.07 20.97L12.58 13.58L19.97 11.07L3 3Z"
                className={cn(
                  "transition-colors duration-200",
                  cursorCoords[step].clicking
                    ? "fill-brand stroke-brand-strong"
                    : "fill-white dark:fill-slate-900 stroke-slate-900 dark:stroke-white"
                )}
                strokeWidth="1.75"
                strokeLinejoin="round"
              />
            </svg>
          </div>
        </div>
      </div>

      {isDone && (
        <div className="animate-in fade-in-50 zoom-in-98 slide-in-from-top-3 duration-300 mt-3 rounded-none border border-emerald-500/40 bg-gradient-to-b from-emerald-500/10 via-card/90 to-card p-3 font-mono text-[11px] shadow-[0_0_25px_-5px_rgba(16,185,129,0.18)]">
          <div className="flex items-center justify-between pb-2 border-b border-border/60 text-[10.5px]">
            <div className="flex items-center gap-2">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-80" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
              </span>
              <span className="font-heading font-bold uppercase tracking-wider text-emerald-600 dark:text-emerald-400 text-[11px]">
                {t.workflow.summaryTitle}
              </span>
            </div>
            <span className="rounded-none border border-emerald-500/40 bg-emerald-500/15 px-2 py-0.5 text-[9.5px] font-bold text-emerald-600 dark:text-emerald-400 tracking-wider">
              240MS · LIBJXL
            </span>
          </div>

          <div className="mt-2.5 space-y-1.5">
            {t.workflow.summaryItems.map((item, idx) => {
              const [label, badge] = item.split(" · ");
              return (
                <div
                  key={idx}
                  style={{ animationDelay: `${idx * 160 + 80}ms` }}
                  className="row-in flex items-center justify-between rounded-none border border-border/60 bg-background/85 hover:border-emerald-500/40 hover:bg-emerald-500/5 px-2.5 py-1.5 text-[11px] transition-all shadow-2xs"
                >
                  <div className="flex items-center gap-2.5 min-w-0">
                    <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-none border border-emerald-500/40 bg-emerald-500/15 text-emerald-600 dark:text-emerald-400">
                      <Check className="h-2.5 w-2.5" strokeWidth={2.5} aria-hidden="true" />
                    </span>
                    <span className="truncate font-semibold text-foreground">
                      {label}
                    </span>
                  </div>
                  {badge && (
                    <span className="shrink-0 pl-2 text-[10px] font-mono font-medium text-emerald-600 dark:text-emerald-400">
                      {badge}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      <div className="mt-4 flex items-center justify-between border-t border-border pt-3 font-mono text-[10.5px] text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <span className="h-1.5 w-1.5 bg-brand" />
          <span>{t.workflow.bottomStatus}</span>
        </span>
        <button
          type="button"
          onClick={() => setStep(0)}
          className="flex items-center gap-1 text-[10px] hover:text-foreground transition-colors cursor-pointer"
        >
          <RotateCcw className="h-3 w-3" aria-hidden="true" />
          <span>{t.workflow.replay}</span>
        </button>
      </div>
    </div>
  );
}

export function Problem() {
  const { t } = useI18n();

  return (
    <Section id="neden" className="relative border-b border-border bg-transparent">
      <Container>
        <Reveal>
          <div className="max-w-[720px]">
            <p className="font-mono text-[11px] font-semibold tracking-widest text-brand-strong uppercase">
              01 / {t.comparison.badge}
            </p>
            <h2 className="mt-3 font-heading font-heading-expanded text-[1.95rem] font-bold tracking-tight text-foreground sm:text-[2.35rem]">
              {t.comparison.title}
            </h2>
            <p className="mt-3 text-[15px] leading-relaxed text-muted-foreground sm:text-base">
              {t.comparison.subtitle}
            </p>
          </div>
        </Reveal>

        <div className="mt-10 grid grid-cols-1 gap-6 lg:grid-cols-2">
          <Reveal delay={0.05}>
            <div className="flex h-full flex-col justify-between rounded-[2px] border border-border bg-card p-6 sm:p-7">
              <div>
                <div className="flex items-center justify-between border-b border-border pb-3.5">
                  <div className="flex items-center gap-2">
                    <Terminal className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                    <h3 className="font-mono text-[14px] font-semibold text-foreground">
                      {t.comparison.oldWayTitle}
                    </h3>
                  </div>
                  <span className="font-mono text-[10px] text-muted-foreground/70 uppercase">
                    {t.comparison.oldWayTag}
                  </span>
                </div>

                <div className="mt-4 overflow-hidden rounded-[2px] border border-border bg-slate-100 dark:bg-[#0d131a] p-3 font-mono text-[11.5px] leading-relaxed text-slate-800 dark:text-[#ccd6df]">
                  <p className="text-red-600 dark:text-red-400">
                    $ ffmpeg -i screencast.mp4 -vf &quot;fps=15,scale=720:-1&quot; -c:v gif out.gif
                  </p>
                </div>

                <ul className="mt-5 space-y-3 font-mono text-[12.5px] text-muted-foreground">
                  {t.comparison.oldWayPoints.map((point) => (
                    <li key={point} className="flex items-start gap-2.5">
                      <span className="select-none font-bold text-muted-foreground/50">−</span>
                      <span className="leading-snug">{point}</span>
                    </li>
                  ))}
                </ul>
              </div>

              <div className="mt-6 border-t border-border pt-3 font-mono text-[10.5px] text-muted-foreground/60">
                {t.comparison.oldWayResult}
              </div>
            </div>
          </Reveal>

          <Reveal delay={0.1}>
            <div className="flex h-full flex-col">
              <RightClickEmulation />
            </div>
          </Reveal>
        </div>
      </Container>
    </Section>
  );
}
