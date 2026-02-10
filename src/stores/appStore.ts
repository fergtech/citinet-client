import { create } from "zustand";

export type AppPhase = "wizard" | "onboarding" | "dashboard";
export type ThemeMode = "light" | "dark" | "system";

interface AppState {
  phase: AppPhase;
  theme: ThemeMode;
  setPhase: (phase: AppPhase) => void;
  setTheme: (theme: ThemeMode) => void;
}

export const useAppStore = create<AppState>((set) => ({
  phase: (localStorage.getItem("citinet-phase") as AppPhase) || "wizard",
  theme: (localStorage.getItem("citinet-theme") as ThemeMode) || "dark",
  setPhase: (phase) => {
    localStorage.setItem("citinet-phase", phase);
    set({ phase });
  },
  setTheme: (theme) => {
    localStorage.setItem("citinet-theme", theme);
    set({ theme });
  },
}));
