import { ko } from "./ko";
import { en } from "./en";
import { createContext, useCallback, useContext, useState } from "react";

export type Locale = "ko" | "en";

export type MessageKey = keyof typeof ko;
export type Messages = Record<MessageKey, string>;

const locales: Record<Locale, Messages> = { ko, en };

const STORAGE_KEY = "forwarder-locale";

function getStored(): Locale {
  const v = localStorage.getItem(STORAGE_KEY);
  return v === "en" ? "en" : "ko";
}

let current: Locale = getStored();

export function t(key: MessageKey): string {
  return locales[current][key];
}

export function useLocale() {
  const [locale, setLocaleState] = useState<Locale>(current);

  const setLocale = useCallback((l: Locale) => {
    current = l;
    localStorage.setItem(STORAGE_KEY, l);
    setLocaleState(l);
  }, []);

  return { locale, setLocale, t };
}

// Context-based locale for app-wide reactivity (re-renders entire tree on locale change)
const LocaleContext = createContext<{ locale: Locale; setLocale: (l: Locale) => void } | null>(null);

export { LocaleContext };

export function useLocaleContext() {
  const ctx = useContext(LocaleContext);
  if (!ctx) throw new Error("useLocaleContext must be used within LocaleProvider");
  return ctx;
}
