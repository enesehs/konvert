"use client";

import * as React from "react";
import { useTheme } from "next-themes";
import { Menu, Moon, Sun, Star, Heart, Languages } from "lucide-react";
import { Sheet, SheetContent, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { LogoMark } from "./logo-mark";
import { cn } from "@/lib/utils";
import { useI18n, LOCALES } from "@/lib/i18n";

function LanguageToggle() {
  const { locale, setLocale } = useI18n();
  return (
    <div
      role="group"
      aria-label="Dil seçimi / Language selection"
      className="flex h-7 items-center border border-border bg-card p-0.5 font-mono text-[11px]"
    >
      {LOCALES.map((l) => (
        <button
          key={l.code}
          type="button"
          onClick={() => setLocale(l.code)}
          aria-label={l.label}
          className={cn(
            "h-full px-2 transition-colors flex items-center justify-center",
            locale === l.code
              ? "bg-brand font-semibold text-primary-foreground"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          {l.short}
        </button>
      ))}
    </div>
  );
}

function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const mounted = React.useSyncExternalStore(
    () => () => {},
    () => true,
    () => false
  );

  return (
    <button
      type="button"
      suppressHydrationWarning
      aria-label={
        mounted && resolvedTheme === "dark"
          ? "Aydınlık temaya geç"
          : "Karanlık temaya geç"
      }
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
      className="inline-flex h-7 w-7 items-center justify-center border border-border bg-card text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
    >
      {mounted ? (
        resolvedTheme === "dark" ? (
          <Sun className="h-3.5 w-3.5" aria-hidden="true" suppressHydrationWarning />
        ) : (
          <Moon className="h-3.5 w-3.5" aria-hidden="true" suppressHydrationWarning />
        )
      ) : (
        <span className="inline-block h-3.5 w-3.5" aria-hidden="true" />
      )}
    </button>
  );
}

function useKonvertVersion() {
  const [version, setVersion] = React.useState("v0.1-beta");

  React.useEffect(() => {
    let active = true;
    fetch("https://raw.githubusercontent.com/enesehs/konvert/refs/heads/main/version")
      .then((res) => {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.text();
      })
      .then((text) => {
        const trimmed = text.trim();
        if (active && trimmed && !trimmed.includes("404") && !trimmed.includes("<html")) {
          setVersion(trimmed.startsWith("v") ? trimmed : `v${trimmed}`);
        }
      })
      .catch(() => {
      });

    return () => {
      active = false;
    };
  }, []);

  return version;
}

export function SiteHeader() {
  const { t, locale, setLocale } = useI18n();
  const [scrolled, setScrolled] = React.useState(false);
  const [open, setOpen] = React.useState(false);
  const version = useKonvertVersion();

  React.useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const navItems = [
    { href: "#neden", label: t.nav.why },
    { href: "#metrikler", label: t.nav.metrics },
    { href: "#formatlar", label: t.nav.formats },
    { href: "#kurulum", label: t.nav.install },
    { href: "#sss", label: t.nav.faq },
  ];

  return (
    <header
      className={cn(
        "sticky top-0 z-50 transition-colors duration-150 backdrop-blur-md",
        scrolled
          ? "border-b border-border bg-background/95"
          : "border-b border-border/60 bg-background/90"
      )}
    >
      <div className="mx-auto flex h-14 w-full max-w-[1120px] items-center justify-between px-5 sm:px-8">
        <a
          href="#main"
          className="flex items-center gap-2.5 transition-opacity hover:opacity-90"
          aria-label="Konvert"
        >
          <LogoMark className="h-7 w-7 sm:h-8 sm:w-8 shrink-0" />
          <span className="font-heading font-heading-expanded font-bold text-[19px] sm:text-[21px] tracking-tight text-foreground">
            konvert
          </span>
          <span className="border-l border-border pl-2.5 font-mono text-[11px] text-muted-foreground select-none">
            {version}
          </span>
        </a>

        <nav aria-label="Ana menü" className="hidden items-center gap-1 lg:flex">
          {navItems.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="px-3 py-1 font-mono text-[12.5px] text-muted-foreground transition-colors hover:text-foreground"
            >
              {item.label}
            </a>
          ))}
        </nav>

        <div className="flex items-center gap-2">
          <div className="hidden sm:block">
            <LanguageToggle />
          </div>

          <a
            href="https://github.com/enesehs/konvert"
            target="_blank"
            rel="noopener noreferrer"
            aria-label="Star on GitHub"
            className="hidden h-7 items-center gap-1.5 border border-border bg-card px-2.5 font-mono text-[11.5px] font-medium text-foreground transition-colors hover:bg-muted sm:inline-flex"
          >
            <Star className="h-3.5 w-3.5 text-amber-500 fill-amber-500/20" aria-hidden="true" />
            <span>Star project</span>
          </a>

          <a
            href="https://github.com/sponsors/enesehs"
            target="_blank"
            rel="noopener noreferrer"
            aria-label="Support on GitHub Sponsors"
            className="hidden h-7 items-center gap-1.5 border border-border bg-card px-2.5 font-mono text-[11.5px] font-medium text-foreground transition-colors hover:bg-muted sm:inline-flex"
          >
            <Heart className="h-3.5 w-3.5 text-pink-500 fill-pink-500/20" aria-hidden="true" />
            <span>Support project</span>
          </a>

          <ThemeToggle />

          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger asChild>
              <button
                type="button"
                aria-label={t.nav.openMenu}
                className="inline-flex h-7 w-7 items-center justify-center border border-border text-foreground lg:hidden"
              >
                <Menu className="h-3.5 w-3.5" aria-hidden="true" />
              </button>
            </SheetTrigger>
            <SheetContent
              side="right"
              className="w-[280px] rounded-none border-border bg-background p-0"
            >
              <SheetTitle className="sr-only">{t.nav.mobileMenu}</SheetTitle>
              <div className="flex h-14 items-center justify-between border-b border-border px-5">
                <div className="flex items-center gap-2.5">
                  <LogoMark className="h-7 w-7 shrink-0" />
                  <span className="font-heading font-heading-expanded font-bold text-[20px]">konvert</span>
                  <span className="border-l border-border pl-2 font-mono text-[10.5px] text-muted-foreground select-none">
                    {version}
                  </span>
                </div>
              </div>

              <div className="border-b border-border px-5 py-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Languages className="h-4 w-4 text-brand" aria-hidden="true" />
                    <span className="font-mono text-[11px]">DİL</span>
                  </div>
                  <div className="flex gap-1 font-mono text-[11px]">
                    {LOCALES.map((l) => (
                      <button
                        key={l.code}
                        type="button"
                        onClick={() => setLocale(l.code)}
                        className={cn(
                          "px-2 py-0.5 border transition-colors",
                          locale === l.code
                            ? "border-brand bg-brand text-primary-foreground font-semibold"
                            : "border-border text-foreground hover:bg-muted"
                        )}
                      >
                        {l.short}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              <nav aria-label={t.nav.mobileMenu} className="flex flex-col px-3 py-3 space-y-1">
                {navItems.map((item) => (
                  <a
                    key={item.href}
                    href={item.href}
                    onClick={() => setOpen(false)}
                    className="flex h-10 items-center px-3 font-mono text-[13px] text-foreground transition-colors hover:bg-muted"
                  >
                    {item.label}
                  </a>
                ))}

                <div className="pt-3 mt-2 border-t border-border space-y-2">
                  <a
                    href="https://github.com/enesehs/konvert"
                    target="_blank"
                    rel="noopener noreferrer"
                    onClick={() => setOpen(false)}
                    className="flex h-9 items-center justify-center gap-2 border border-border bg-card px-4 font-mono text-[12.5px] text-foreground"
                  >
                    <Star className="h-3.5 w-3.5 text-amber-500 fill-amber-500/20" aria-hidden="true" />
                    <span>Star project</span>
                  </a>

                  <a
                    href="https://github.com/sponsors/enesehs"
                    target="_blank"
                    rel="noopener noreferrer"
                    onClick={() => setOpen(false)}
                    className="flex h-9 items-center justify-center gap-2 border border-border bg-card px-4 font-mono text-[12.5px] text-foreground"
                  >
                    <Heart className="h-3.5 w-3.5 text-pink-500 fill-pink-500/20" aria-hidden="true" />
                    <span>Support project</span>
                  </a>
                </div>
              </nav>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
