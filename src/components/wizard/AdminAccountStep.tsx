import { useState } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { Mail, Lock, User as UserIcon, AlertCircle } from "lucide-react";

export function AdminAccountStep() {
  const { nextStep, prevStep } = useWizardStore();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const isValid = 
    username.length >= 3 &&
    email.includes("@") &&
    password.length >= 8 &&
    password === confirmPassword;

  const handleContinue = () => {
    if (!isValid) {
      setError("Please fill in all fields correctly");
      return;
    }

    // Store credentials in wizardStore temporarily
    // They'll be used in ProgressStep to create the admin user
    useWizardStore.setState({
      adminUsername: username,
      adminEmail: email,
      adminPassword: password,
    });

    nextStep();
  };

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Create Admin Account
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Set up your administrator account to manage this hub
      </p>

      <div className="space-y-4 mb-6">
        <div>
          <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
            <UserIcon className="w-4 h-4 inline mr-1.5" />
            Username
          </label>
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="admin"
            className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
            autoComplete="username"
          />
          {username && username.length < 3 && (
            <p className="text-xs text-red-500 mt-1">Username must be at least 3 characters</p>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
            <Mail className="w-4 h-4 inline mr-1.5" />
            Email
          </label>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="admin@example.com"
            className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
            autoComplete="email"
          />
          {email && !email.includes("@") && (
            <p className="text-xs text-red-500 mt-1">Please enter a valid email</p>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
            <Lock className="w-4 h-4 inline mr-1.5" />
            Password
          </label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="••••••••"
            className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
            autoComplete="new-password"
          />
          {password && password.length < 8 && (
            <p className="text-xs text-red-500 mt-1">Password must be at least 8 characters</p>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-[var(--text-primary)] mb-1.5">
            <Lock className="w-4 h-4 inline mr-1.5" />
            Confirm Password
          </label>
          <input
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder="••••••••"
            className="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
            autoComplete="new-password"
          />
          {confirmPassword && password !== confirmPassword && (
            <p className="text-xs text-red-500 mt-1">Passwords do not match</p>
          )}
        </div>

        {error && (
          <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30">
            <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
          </div>
        )}

        <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/30">
          <p className="text-xs text-blue-600 dark:text-blue-400">
            This admin account will have full access to manage the hub, including creating additional users and configuring settings.
          </p>
        </div>
      </div>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={handleContinue} disabled={!isValid} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
