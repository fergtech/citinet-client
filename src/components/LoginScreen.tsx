import { useState } from "react";
import { useAppStore } from "../stores/appStore";
import { useAuthStore } from "../stores/authStore";
import { CitinetAPI } from "../api/tauri";
import { Cloud, Loader2 } from "lucide-react";
import { Button } from "./ui/Button";

export function LoginScreen() {
  const setPhase = useAppStore((s) => s.setPhase);
  const setCurrentUser = useAuthStore((s) => s.setCurrentUser);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleLogin = async () => {
    if (!username || !password) return;
    setLoading(true);
    setError(null);

    try {
      const user = await CitinetAPI.loginUser(username, password);
      setCurrentUser(user);
      setPhase("dashboard");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && username && password && !loading) {
      handleLogin();
    }
  };

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-[var(--bg-primary)] p-8">
      <div className="flex items-center gap-2.5 mb-10">
        <Cloud className="w-9 h-9 text-primary-500" />
        <span className="text-2xl font-bold text-[var(--text-primary)]">
          Citinet
        </span>
      </div>

      <div className="w-full max-w-sm bg-[var(--bg-card)] border border-[var(--border-color)] rounded-2xl p-8 shadow-lg">
        <h2 className="text-xl font-bold text-[var(--text-primary)] mb-2">
          Welcome back
        </h2>
        <p className="text-sm text-[var(--text-secondary)] mb-6">
          Sign in to your hub
        </p>

        <div className="space-y-4" onKeyDown={handleKeyDown}>
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
              Username
            </label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter your username"
              className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              autoComplete="username"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
              Password
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter your password"
              className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              autoComplete="current-password"
            />
          </div>

          {error && (
            <p className="text-sm text-red-500">{error}</p>
          )}

          <Button
            onClick={handleLogin}
            disabled={!username || !password || loading}
            className="w-full"
          >
            {loading ? (
              <Loader2 className="w-4 h-4 animate-spin mr-2" />
            ) : null}
            {loading ? "Signing in..." : "Sign In"}
          </Button>
        </div>
      </div>
    </div>
  );
}
