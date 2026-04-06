import Layout from "./components/Layout";
import { useTheme } from "./hooks/useTheme";
import { LocaleContext, useLocale } from "./i18n";

export const ThemeContext = { useTheme };

function App() {
  // Initialize theme on mount
  useTheme();
  const { locale, setLocale } = useLocale();
  return (
    <LocaleContext.Provider value={{ locale, setLocale }}>
      <Layout />
    </LocaleContext.Provider>
  );
}

export default App;
