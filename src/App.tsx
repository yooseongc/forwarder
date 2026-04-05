import Layout from "./components/Layout";
import { useTheme } from "./hooks/useTheme";

export const ThemeContext = { useTheme };

function App() {
  // Initialize theme on mount
  useTheme();
  return <Layout />;
}

export default App;
