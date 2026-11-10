"use client";

import * as React from "react";
import { Locale, Translations } from "./types";
import { tr } from "./tr";
import { en } from "./en";
import { de } from "./de";

export const LOCALES: Array<{ code: Locale; label: string; short: string }> = [
  { code: "tr", label: "Türkçe", short: "TR" },
  { code: "en", label: "English", short: "EN" },
  { code: "de", label: "Deutsch", short: "DE" },
];

const DICTIONARIES: Record<Locale, Translations> = {
  tr,
  en,
  de,
};

const STORAGE_KEY = "konvert_locale";

interface I18nContextType {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: Translations;
}

const I18nContext = React.createContext<I18nContextType>({
  locale: "en",
  setLocale: () => {},
  t: en,
});

function detectLocale(): Locale {
  try {
    const urlParams = new URLSearchParams(window.location.search);
    const urlLang = urlParams.get("lang")?.toLowerCase() as Locale | null;
    if (urlLang && (urlLang === "tr" || urlLang === "en" || urlLang === "de")) {
      return urlLang;
    }

    const stored = localStorage.getItem(STORAGE_KEY) as Locale | null;
    if (stored && (stored === "tr" || stored === "en" || stored === "de")) {
      return stored;
    }

    const browserLang = navigator.language?.toLowerCase() || "";
    if (browserLang.startsWith("tr")) return "tr";
    if (browserLang.startsWith("de")) return "de";
    return "en";
  } catch {
    return "en";
  }
}

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [locale, setLocaleState] = React.useState<Locale>("en");

  React.useEffect(() => {
    const detected = detectLocale();
    if (detected !== "en") {
      queueMicrotask(() => {
        setLocaleState(detected);
      });
    }
  }, []);

  const setLocale = React.useCallback((nextLocale: Locale) => {
    setLocaleState(nextLocale);
    try {
      localStorage.setItem(STORAGE_KEY, nextLocale);
      document.documentElement.lang = nextLocale;
      const url = new URL(window.location.href);
      url.searchParams.set("lang", nextLocale);
      window.history.replaceState({}, "", url.toString());
    } catch {
    }
  }, []);

  const t = DICTIONARIES[locale] || en;

  React.useEffect(() => {
    document.title = t.meta.title;
    document.documentElement.lang = locale;
    const metaDesc = document.querySelector('meta[name="description"]');
    if (metaDesc) {
      metaDesc.setAttribute("content", t.meta.description);
    }
  }, [locale, t]);

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n(): I18nContextType {
  return React.useContext(I18nContext);
}
