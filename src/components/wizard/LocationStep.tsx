import { useState, useEffect } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { FolderOpen, AlertCircle, CheckCircle2 } from "lucide-react";
import { CitinetAPI } from "../../api/tauri";

export function LocationStep() {
  const { installPath, setInstallPath, nextStep, prevStep } = useWizardStore();
  const [validating, setValidating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [validationSuccess, setValidationSuccess] = useState(false);

  // Load recommended path on mount
  useEffect(() => {
    const loadRecommendedPath = async () => {
      try {
        const recommended = await CitinetAPI.getRecommendedInstallPath();
        if (!installPath || installPath === "C:\\Program Files\\Citinet") {
          setInstallPath(recommended);
        }
      } catch (err) {
        console.error("Failed to get recommended path:", err);
      }
    };
    loadRecommendedPath();
  }, []);

  // Validate path when it changes
  useEffect(() => {
    const validatePath = async () => {
      if (!installPath) {
        setValidationError(null);
        setValidationSuccess(false);
        return;
      }

      setValidating(true);
      setValidationError(null);
      setValidationSuccess(false);

      try {
        await CitinetAPI.validateInstallPath(installPath);
        setValidationSuccess(true);
      } catch (err) {
        setValidationError(err instanceof Error ? err.message : String(err));
      } finally {
        setValidating(false);
      }
    };

    const timer = setTimeout(validatePath, 500);
    return () => clearTimeout(timer);
  }, [installPath]);

  const handleBrowse = () => {
    // In production, the Tauri dialog plugin would handle this.
    // For now, users can type the path manually.
    const path = prompt("Enter installation path:");
    if (path) {
      setInstallPath(path);
    }
  };

  const canContinue = installPath && validationSuccess && !validating;

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Install Location
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Choose where Citinet will store its data.
      </p>

      <div className="flex gap-2 mb-2">
        <input
          id="install-path"
          type="text"
          aria-label="Installation path"
          value={installPath}
          onChange={(e) => setInstallPath(e.target.value)}
          className="flex-1 px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)] focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
        <Button variant="secondary" onClick={handleBrowse}>
          <FolderOpen className="w-4 h-4" />
        </Button>
      </div>

      {validating && (
        <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)] mb-4">
          <div className="w-4 h-4 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
          <span>Validating path...</span>
        </div>
      )}

      {validationSuccess && !validating && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-green-500/10 border border-green-500/30 mb-4">
          <CheckCircle2 className="w-4 h-4 text-green-500 shrink-0" />
          <p className="text-sm text-green-600 dark:text-green-400">Path is valid and writable</p>
        </div>
      )}

      {validationError && !validating && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 mb-4">
          <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
          <div className="flex-1">
            <p className="text-sm font-medium text-red-600 dark:text-red-400">Invalid Path</p>
            <p className="text-xs text-red-600/80 dark:text-red-400/80 mt-1">{validationError}</p>
          </div>
        </div>
      )}

      <p className="text-xs text-[var(--text-muted)] mb-8">
        Approximately 50 MB required for base installation. The default location uses your user AppData folder, which doesn't require administrator privileges.
      </p>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} disabled={!canContinue} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
