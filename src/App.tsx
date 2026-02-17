import { useEffect, useState } from "react";
import { useAppStore } from "./stores/appStore";
import { useAuthStore } from "./stores/authStore";
import { CitinetAPI } from "./api/tauri";
import { Wizard } from "./components/wizard/Wizard";
import { LoginScreen } from "./components/LoginScreen";
import { Dashboard } from "./components/dashboard/Dashboard";

function App() {
  const { phase, setPhase, theme } = useAppStore();
  const currentUser = useAuthStore((s) => s.currentUser);
  const [ready, setReady] = useState(false);

  // On startup, check if the node is already configured on the backend
  useEffect(() => {
    CitinetAPI.getNodeConfig()
      .then((config) => {
        if (config) {
          // Node exists — skip wizard, go to login or dashboard
          if (currentUser) {
            setPhase("dashboard");
          } else if (phase === "wizard") {
            setPhase("login");
          }
        } else {
          // No node configured — ensure wizard
          setPhase("wizard");
        }
      })
      .catch(() => {
        // Backend not ready yet, keep current phase
      })
      .finally(() => setReady(true));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

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

  if (!ready) {
    return (
      <div className="h-screen flex items-center justify-center bg-[var(--bg-primary)]">
        <div className="w-6 h-6 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

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
