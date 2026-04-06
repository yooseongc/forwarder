import { useEffect } from "react";
import Layout from "./components/Layout";
import { useTheme } from "./hooks/useTheme";
import { LocaleContext, useLocale } from "./i18n";

export const ThemeContext = { useTheme };

function App() {
  useTheme();
  const { locale, setLocale } = useLocale();

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  return (
    <LocaleContext.Provider value={{ locale, setLocale }}>
      <Layout />
    </LocaleContext.Provider>
  );
}

export default App;
