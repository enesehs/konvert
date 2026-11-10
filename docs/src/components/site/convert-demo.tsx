"use client";

import * as React from "react";
import {
  BadgeCheck,
  Check,
  ChevronLeft,
  ChevronRight,
  FileArchive,
  FileAudio,
  FileImage,
  FileText,
  FileVideo,
  Folder,
  LayoutGrid,
  List,
  Loader2,
  Minus,
  MousePointer2,
  RotateCcw,
  Sparkles,
  Diamond,
  ChevronDown,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";

type FileItem = {
  name: string;
  ext: string;
  size: string;
  selected: boolean;
  desktop?: boolean;
  icon: React.ComponentType<{ className?: string }>;
};

const FILES: FileItem[] = [
  { name: "hero-banner", ext: "webp", size: "3,8 MB", selected: true, icon: FileImage },
  { name: "screenshot-arch", ext: "png", size: "2,4 MB", selected: true, icon: FileImage },
  { name: "screencast-demo", ext: "mp4", size: "18,2 MB", selected: false, desktop: true, icon: FileVideo },
  { name: "podcast-episode", ext: "wav", size: "12,6 MB", selected: false, desktop: true, icon: FileAudio },
  { name: "kernel-notes", ext: "md", size: "14 KB", selected: false, desktop: true, icon: FileText },
  { name: "backup-dotfiles", ext: "tar.gz", size: "48 MB", selected: false, desktop: true, icon: FileArchive },
];

type Target = "PNG" | "JPG" | "JXL" | "WEBP" | "AVIF";

const TARGETS: Record<
  Target,
  { sizes: [string, string]; total: string; saved: number; time: string }
> = {
  PNG: { sizes: ["1,2 MB", "1,8 MB"], total: "3,0 MB", saved: 51, time: "14ms" },
  JPG: { sizes: ["0,9 MB", "1,1 MB"], total: "2,0 MB", saved: 67, time: "11ms" },
  JXL: { sizes: ["0,5 MB", "0,7 MB"], total: "1,2 MB", saved: 81, time: "22ms" },
  WEBP: { sizes: ["0,6 MB", "0,8 MB"], total: "1,4 MB", saved: 77, time: "18ms" },
  AVIF: { sizes: ["0,4 MB", "0,6 MB"], total: "1,0 MB", saved: 83, time: "38ms" },
};

type Phase = "idle" | "converting" | "done";

export function ConvertDemo() {
  const { t } = useI18n();
  const [phase, setPhase] = React.useState<Phase>("idle");
  const [target, setTarget] = React.useState<Target>("JXL");
  const timer = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  React.useEffect(() => {
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
  }, []);

  const start = (fmt: Target) => {
    if (phase === "converting") return;
    setTarget(fmt);
    setPhase("converting");
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => setPhase("done"), reduce ? 120 : 950);
  };

  const reset = () => {
    if (timer.current) clearTimeout(timer.current);
    setPhase("idle");
  };

  const result = TARGETS[target];
  let selectedIdx = 0;

  return (
    <div className="w-full">
      <div className="mb-2.5 flex items-center justify-between">
        <div className="inline-flex items-center gap-2 rounded-none border border-emerald-500/40 bg-emerald-500/10 px-3 py-1 font-mono text-[11px] font-semibold text-emerald-600 dark:text-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.2)]">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
          </span>
          <span>{t.hero.interactiveBadge}</span>
        </div>
        <span className="font-mono text-[10.5px] text-muted-foreground hidden sm:inline">
          KDE Plasma · Dolphin 24.08
        </span>
      </div>
      <div className="overflow-hidden rounded-[2px] border border-border bg-card shadow-sm">
        <div className="flex h-9 items-center justify-between border-b border-border bg-muted/70 px-3 font-mono text-[11px] text-muted-foreground select-none">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-foreground">Dolphin</span>
            <span className="text-muted-foreground/60">—</span>
            <span className="text-muted-foreground">~/Downloads</span>
          </div>
          <div className="flex items-center gap-2.5 text-muted-foreground">
            <span className="cursor-default hover:text-foreground">
              <ChevronDown className="h-3 w-3" aria-hidden="true" />
            </span>
            <span className="cursor-default hover:text-foreground">
              <Diamond className="h-2.5 w-2.5" aria-hidden="true" />
            </span>
            <span className="cursor-default hover:text-red-400">
              <X className="h-3 w-3" aria-hidden="true" />
            </span>
          </div>
        </div>
        <div className="flex items-center gap-2 border-b border-border bg-card px-3.5 py-1.5 font-mono text-[11px]">
          <div className="flex items-center gap-1 text-muted-foreground" aria-hidden="true">
            <ChevronLeft className="h-3.5 w-3.5" />
            <ChevronRight className="h-3.5 w-3.5 opacity-40" />
          </div>
          <div className="flex min-w-0 items-center gap-1.5 text-muted-foreground">
            <Folder className="h-3.5 w-3.5 text-brand shrink-0" aria-hidden="true" />
            <span className="text-muted-foreground">/home/{t.demo.pathUser}/</span>
            <span className="font-semibold text-foreground">{t.demo.pathFolder}</span>
          </div>
          <div className="ml-auto hidden items-center gap-2 text-muted-foreground sm:flex" aria-hidden="true">
            <span className="text-[10px] text-muted-foreground/70">6 öğe (2 seçildi)</span>
            <span className="h-3 w-px bg-border" />
            <LayoutGrid className="h-3.5 w-3.5 opacity-40" />
            <List className="h-3.5 w-3.5 text-brand" />
          </div>
        </div>
        <div className="relative min-h-[250px] bg-background/50">
          {phase === "converting" && (
            <div
              className="bar-fill absolute inset-x-0 top-0 z-20 h-[2px] bg-brand"
              key={`bar-${target}`}
              aria-hidden="true"
            />
          )}
          <div className="flex items-center justify-between border-b border-border bg-muted/30 px-3.5 py-1 font-mono text-[10px] tracking-wider text-muted-foreground">
            <span>{t.demo.colName}</span>
            <span className="w-24 text-right">{t.demo.colSize}</span>
          </div>
          <ul className="divide-y divide-border/60">
            {FILES.map((file) => {
              const isSelected = file.selected;
              const i = isSelected ? selectedIdx++ : -1;
              const done = phase === "done" && isSelected;
              const converting = phase === "converting" && isSelected;
              const Icon = file.icon;

              return (
                <li
                  key={file.name}
                  className={cn(
                    "relative flex h-9 items-center gap-2.5 px-3.5 font-mono text-[12px] transition-colors",
                    file.desktop ? "hidden sm:flex" : "flex",
                    isSelected && "bg-brand/10 border-l-2 border-brand"
                  )}
                >
                  {converting ? (
                    <Loader2
                      className="h-3.5 w-3.5 shrink-0 animate-spin text-brand"
                      aria-hidden="true"
                    />
                  ) : (
                    <Icon
                      className={cn(
                        "h-3.5 w-3.5 shrink-0",
                        done ? "text-emerald-500" : isSelected ? "text-brand" : "text-muted-foreground"
                      )}
                      aria-hidden="true"
                    />
                  )}
                  <span className="min-w-0 truncate">
                    {file.name}
                    {done ? (
                      <span className="font-semibold text-emerald-500">
                        .{target.toLowerCase()}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">.{file.ext}</span>
                    )}
                  </span>
                  {done && (
                    <span className="ml-auto font-mono text-[10px] text-emerald-500 flex items-center gap-1">
                      <Check className="h-3 w-3" aria-hidden="true" />
                      <span>{result.time}</span>
                    </span>
                  )}
                  <span
                    className={cn(
                      "w-24 shrink-0 text-right font-mono text-[11px] tabular-nums",
                      done ? "ml-0" : "ml-auto"
                    )}
                  >
                    {done ? (
                      <span className="font-semibold text-emerald-500">
                        {result.sizes[i] || "0,8 MB"}
                      </span>
                    ) : converting ? (
                      <span className="text-muted-foreground">işleniyor...</span>
                    ) : (
                      <span className="text-muted-foreground">{file.size}</span>
                    )}
                  </span>
                </li>
              );
            })}
          </ul>
          {phase === "idle" && (
            <div className="relative z-10 border-t border-border bg-card p-1 sm:absolute sm:left-12 sm:top-6 sm:w-[204px] sm:rounded-[2px] sm:border sm:border-border sm:bg-popover sm:p-1 sm:shadow-lg">
              <MousePointer2
                className="hidden sm:block absolute -top-3.5 -left-2 h-4 w-4 fill-foreground text-foreground"
                aria-hidden="true"
              />
              <div className="hidden sm:block font-mono text-[11.5px] text-foreground/80 space-y-0.5">
                <div className="px-2 py-0.5 hover:bg-muted text-muted-foreground">{t.workflow.menuOpen}</div>
                <div className="px-2 py-0.5 hover:bg-muted text-muted-foreground">{t.workflow.menuCopy}</div>
                <div className="my-1 border-t border-border" />
              </div>
              <div className="bg-muted px-2 py-1 font-mono text-[11.5px] font-semibold text-foreground flex items-center justify-between border-l-2 border-brand">
                <span className="flex items-center gap-1.5">
                  <Sparkles className="h-3 w-3 text-brand" aria-hidden="true" />
                  Konvert
                </span>
                <ChevronRight className="h-3 w-3 text-muted-foreground" aria-hidden="true" />
              </div>
              <div className="mt-1 space-y-0.5 border-t border-border pt-1 font-mono text-[11px]">
                {(Object.keys(TARGETS) as Target[]).map((fmt) => (
                  <button
                    key={fmt}
                    type="button"
                    onClick={() => start(fmt)}
                    className="flex h-6 w-full items-center justify-between px-2 text-left hover:bg-brand hover:text-primary-foreground transition-colors group"
                  >
                    <span className="font-semibold">{fmt}</span>
                    <span className="text-[10px] text-muted-foreground group-hover:text-primary-foreground/90">
                      .{fmt.toLowerCase()}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          )}
          {phase === "done" && (
            <div
              role="status"
              aria-live="polite"
              className="toast-in absolute bottom-3 right-3 z-30 w-[290px] max-w-[calc(100%-24px)] rounded-[2px] border border-border bg-slate-900 dark:bg-[#0d131a] p-3 font-mono text-white shadow-xl"
            >
              <div className="flex items-start gap-2.5">
                <BadgeCheck className="mt-0.5 h-4 w-4 shrink-0 text-emerald-400" aria-hidden="true" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between text-[10px] text-slate-400">
                    <span>KONVERT · D-BUS</span>
                    <span className="text-emerald-400">0 MB RAM</span>
                  </div>
                  <p className="mt-0.5 text-[12px] font-semibold text-white">
                    {t.demo.notificationSuccess} ({result.time})
                  </p>
                  <p className="mt-1 text-[11px] leading-snug text-slate-300">
                    2 dosya başarıyla {target} formatına dönüştürüldü ({result.total}).
                  </p>
                  <button
                    type="button"
                    onClick={reset}
                    className="mt-2 inline-flex items-center gap-1 border border-white/20 bg-white/10 px-2 py-0.5 text-[10px] font-medium text-white hover:bg-white/20"
                  >
                    <RotateCcw className="h-3 w-3" aria-hidden="true" />
                    {t.demo.replay}
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
        <div className="flex h-8 items-center justify-between gap-2 border-t border-border bg-muted/40 px-3.5 font-mono text-[10.5px] text-muted-foreground">
          <span className="truncate">
            {phase === "idle" && "Hazır — Menüden format seçerek canlı dönüşümü test edin"}
            {phase === "converting" && `Rust Rayon iş parçacığı dönüştürüyor: ${target}...`}
            {phase === "done" && `Tamamlandı: %${result.saved} disk tasarrufu sağlandı.`}
          </span>
          {phase === "done" && (
            <button
              type="button"
              onClick={reset}
              className="font-semibold text-brand-strong hover:underline"
            >
              [ Sıfırla ]
            </button>
          )}
        </div>
      </div>
      <div className="mt-2.5 flex items-center justify-between font-mono text-[10.5px] text-muted-foreground px-1">
        <span>{t.demo.interactiveHint}</span>
        <span>JXL · WEBP · AVIF · PNG · JPG</span>
      </div>
    </div>
  );
}
