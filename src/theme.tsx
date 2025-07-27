import { createContext, useContext, createSignal, createEffect } from "solid-js";
import { JSXElement } from "solid-js";

interface ThemeContextType {
  theme: () => "light" | "dark";
  toggleTheme: () => void;
  accent: () => string;
  setAccent: (color: string) => void;
}

const ThemeContext = createContext<ThemeContextType>();

export function ThemeProvider(props: { children: JSXElement }) {
  const [theme, setTheme] = createSignal<"light" | "dark">(
    localStorage.getItem("theme") === "dark" ? "dark" : "light"
  );
  const [accent, setAccent] = createSignal<string>(
    localStorage.getItem("accent") || "#22c55e"
  );

  const toggleTheme = () => {
    setTheme((prev) => {
      const next = prev === "light" ? "dark" : "light";
      localStorage.setItem("theme", next);
      return next;
    });
  };

  const setAccentColor = (color: string) => {
    setAccent(color);
    localStorage.setItem("accent", color);
  };

  createEffect(() => {
    const isDark = theme() === "dark";
    document.documentElement.classList.toggle("dark", isDark);
    document.documentElement.style.setProperty("--accent", accent());
  });

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme, accent, setAccent: setAccentColor }}>
      {props.children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext)!;
}
