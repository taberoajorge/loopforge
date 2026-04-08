import { create } from "zustand";

export type Theme = "dark" | "light";

interface ThemeState {
  theme: Theme;
  toggle: () => void;
  setTheme: (theme: Theme) => void;
}

const THEME_KEY = "loopforge-theme";
const THEME_CLASSES: Theme[] = ["light", "dark"];

function readSavedTheme(): Theme {
  if (typeof window === "undefined") {
    return "dark";
  }

  const savedTheme = window.localStorage.getItem(THEME_KEY);
  return savedTheme === "light" || savedTheme === "dark" ? savedTheme : "dark";
}

function syncRootTheme(theme: Theme) {
  if (typeof document === "undefined") {
    return;
  }

  const rootElement = document.documentElement;
  for (const themeClass of THEME_CLASSES) {
    rootElement.classList.toggle(themeClass, themeClass === theme);
  }
}

function applyTheme(theme: Theme) {
  syncRootTheme(theme);
  if (typeof window !== "undefined") {
    window.localStorage.setItem(THEME_KEY, theme);
  }
}

export function initializeTheme(): Theme {
  const theme = readSavedTheme();
  syncRootTheme(theme);
  return theme;
}

const initialTheme = readSavedTheme();
syncRootTheme(initialTheme);

export const useThemeStore = create<ThemeState>()((set) => ({
  theme: initialTheme,
  toggle: () =>
    set((state) => {
      const nextTheme: Theme = state.theme === "dark" ? "light" : "dark";
      applyTheme(nextTheme);
      return { theme: nextTheme };
    }),
  setTheme: (theme) => {
    applyTheme(theme);
    set({ theme });
  },
}));
