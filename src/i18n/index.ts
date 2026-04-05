import { ko } from "./ko";
import { en } from "./en";
import { useCallback, useState } from "react";

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
