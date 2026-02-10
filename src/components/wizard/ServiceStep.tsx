import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { Toggle } from "../ui/Toggle";
import { Cpu } from "lucide-react";

export function ServiceStep() {
  const { autoStart, setAutoStart, nextStep, prevStep } = useWizardStore();

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        System Service
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Configure how Citinet runs on your system.
      </p>

      <div className="bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] rounded-lg p-4 mb-6">
        <div className="flex items-start gap-3">
          <Cpu className="w-5 h-5 text-primary-500 mt-0.5" />
          <div className="flex-1">
            <Toggle
              checked={autoStart}
              onChange={setAutoStart}
              label="Start Citinet on system boot"
              description="Citinet will run as a background service and start automatically when you log in."
            />
          </div>
        </div>
      </div>

      <p className="text-xs text-[var(--text-muted)] mb-8">
        You can change this setting later from the app's Settings page.
      </p>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} className="flex-1">
          Install
        </Button>
      </div>
    </div>
  );
}
