import * as React from "react";

type Theme = "light" | "dark";

interface ThemeContextValue {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

const ThemeContext = React.createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setThemeState] = React.useState<Theme>(() => {
    const saved = localStorage.getItem("autoevaluacion-theme");
    return saved === "dark" ? "dark" : "light";
  });

  const setTheme = React.useCallback((nextTheme: Theme) => {
    document.documentElement.dataset.theme = nextTheme;
    localStorage.setItem("autoevaluacion-theme", nextTheme);
    setThemeState(nextTheme);
  }, []);

  React.useEffect(() => {
    setTheme(theme);
  }, [setTheme, theme]);

  const value = React.useMemo(
    () => ({
      theme,
      setTheme,
      toggleTheme: () => setTheme(theme === "dark" ? "light" : "dark"),
    }),
    [setTheme, theme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const context = React.useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within ThemeProvider");
  }
  return context;
}
