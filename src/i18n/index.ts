import { ko } from "./ko";
import { en } from "./en";

export type Locale = "ko" | "en";

export type MessageKey = keyof typeof ko;
export type Messages = Record<MessageKey, string>;

const locales: Record<Locale, Messages> = { ko, en };

let currentLocale: Locale = "ko";

export function setLocale(locale: Locale) {
  currentLocale = locale;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function t(key: MessageKey): string {
  return locales[currentLocale][key];
}
