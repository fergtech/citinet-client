import { useEffect } from "react";
import { useAppStore } from "./stores/appStore";
import { useAuthStore } from "./stores/authStore";
import { Wizard } from "./components/wizard/Wizard";
import { LoginScreen } from "./components/LoginScreen";
import { Dashboard } from "./components/dashboard/Dashboard";

function App() {
  const { phase, setPhase, theme } = useAppStore();
  const currentUser = useAuthStore((s) => s.currentUser);

  // Redirect to login if dashboard but no auth
  useEffect(() => {
    if (phase === "dashboard" && !currentUser) {
      setPhase("login");
    }
  }, [phase, currentUser, setPhase]);

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
    case "login":
      return <LoginScreen />;
    case "dashboard":
      return currentUser ? <Dashboard /> : <LoginScreen />;
  }
}

export default App;
