import { useEffect, useState } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { useConfigStore } from "../../stores/configStore";
import { ProgressBar } from "../ui/ProgressBar";
import { CheckCircle2, Loader2, AlertCircle } from "lucide-react";
import { CitinetAPI } from "../../api/tauri";

const INSTALL_STEPS = [
  "Creating directories...",
  "Initializing database...",
  "Saving node configuration...",
  "Creating admin account...",
  "Finalizing...",
];

export function ProgressStep() {
  const { installProgress, setInstallProgress, nextStep } = useWizardStore();
  const wizardState = useWizardStore();
  const configState = useConfigStore();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const run = async () => {
      try {
        // Step 0: Creating directories
        setInstallProgress(10);

        // Step 1-2: Real backend call — creates dirs, DB, saves config
        if (cancelled) return;
        setInstallProgress(20);

        const config = await CitinetAPI.initializeNode(
          wizardState.installPath,
          'hub',
          wizardState.nodeName || "hub-node",
          wizardState.storageContribution,
          configState.contribution.bandwidthMbps,
          configState.contribution.cpuPercent,
          wizardState.autoStart,
        );

        if (cancelled) return;
        setInstallProgress(60);

        // Step 3: Create admin account
        if (wizardState.adminUsername && wizardState.adminEmail && wizardState.adminPassword) {
          const adminUser = await CitinetAPI.createAdminUser(
            wizardState.adminUsername,
            wizardState.adminEmail,
            wizardState.adminPassword,
          );
          useWizardStore.setState({ createdUser: adminUser });
        }

        if (cancelled) return;
        setInstallProgress(80);

        // Step 4: Finalizing — sync config store with backend result
        configState.setNodeName(wizardState.nodeName || "hub-node");
        configState.setInstallPath(config.install_path);
        configState.setContribution({ diskSpaceGB: wizardState.storageContribution });
        configState.setConfigured(true);

        if (cancelled) return;
        setInstallProgress(100);

        setTimeout(() => {
          if (!cancelled) nextStep();
        }, 500);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    run();
    return () => {
      cancelled = true;
    };
  }, []);

  const stepIndex = Math.min(
    Math.floor(installProgress / (100 / INSTALL_STEPS.length)),
    INSTALL_STEPS.length - 1
  );

  if (error) {
    return (
      <div>
        <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
          Installation Failed
        </h2>
        <div className="flex items-start gap-2 p-4 rounded-lg bg-red-500/10 border border-red-500/30 mt-4">
          <AlertCircle className="w-5 h-5 text-red-500 mt-0.5 shrink-0" />
          <div>
            <p className="text-sm font-medium text-red-500">Error</p>
            <p className="text-sm text-[var(--text-secondary)] mt-1">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Installing Citinet
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-8">
        Please wait while we set things up...
      </p>

      <ProgressBar value={installProgress} />

      <div className="mt-6 space-y-2">
        {INSTALL_STEPS.map((step, i) => (
          <div key={step} className="flex items-center gap-2">
            {i < stepIndex ? (
              <CheckCircle2 className="w-4 h-4 text-accent-500" />
            ) : i === stepIndex ? (
              <Loader2 className="w-4 h-4 text-primary-500 animate-spin" />
            ) : (
              <div className="w-4 h-4 rounded-full border border-[var(--border-color)]" />
            )}
            <span
              className={`text-sm ${
                i <= stepIndex
                  ? "text-[var(--text-primary)]"
                  : "text-[var(--text-muted)]"
              }`}
            >
              {step}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
