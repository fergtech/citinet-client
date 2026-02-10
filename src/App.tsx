import { useEffect } from "react";
import { useAppStore } from "./stores/appStore";
import { Wizard } from "./components/wizard/Wizard";
import { Onboarding } from "./components/onboarding/Onboarding";
import { Dashboard } from "./components/dashboard/Dashboard";

function App() {
  const { phase, theme } = useAppStore();

  // Apply theme class to document
  useEffect(() => {
    const root = document.documentElement;

    const applyTheme = (dark: boolean) => {
      if (dark) {
        root.classList.add("dark");
      } else {
        root.classList.remove("dark");
      }
    };

    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      applyTheme(mq.matches);
      const handler = (e: MediaQueryListEvent) => applyTheme(e.matches);
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    } else {
      applyTheme(theme === "dark");
    }
  }, [theme]);

  switch (phase) {
    case "wizard":
      return <Wizard />;
    case "onboarding":
      return <Onboarding />;
    case "dashboard":
      return <Dashboard />;
  }
}

export default App;
