import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { FolderOpen } from "lucide-react";

export function LocationStep() {
  const { installPath, setInstallPath, nextStep, prevStep } = useWizardStore();

  const handleBrowse = () => {
    // In production, the Tauri dialog plugin would handle this.
    // For now, users can type the path manually.
    const path = prompt("Enter installation path:");
    if (path) {
      setInstallPath(path);
    }
  };

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

      <p className="text-xs text-[var(--text-muted)] mb-8">
        Approximately 50 MB required for base installation.
      </p>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} disabled={!installPath} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
