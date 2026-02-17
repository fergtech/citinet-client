import { useWizardStore } from "../../stores/wizardStore";
import { useConfigStore } from "../../stores/configStore";
import { Button } from "../ui/Button";
import { HardDrive } from "lucide-react";

export function ContributionSlider() {
  const { storageContribution, setStorageContribution, nextStep, prevStep } =
    useWizardStore();
  const setContribution = useConfigStore((s) => s.setContribution);

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Choose Your Contribution
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        How much storage would you like to share with the network?
      </p>

      <div className="flex flex-col items-center mb-6">
        <div className="w-20 h-20 rounded-full bg-accent-500/10 flex items-center justify-center mb-3">
          <HardDrive className="w-8 h-8 text-accent-500" />
        </div>
        <span className="text-4xl font-bold text-[var(--text-primary)]">
          {storageContribution} GB
        </span>
        <span className="text-sm text-[var(--text-secondary)]">
          shared storage
        </span>
      </div>

      <label htmlFor="wizard-storage" className="sr-only">Storage contribution</label>
      <input
        id="wizard-storage"
        type="range"
        min={1}
        max={50}
        value={storageContribution}
        onChange={(e) => {
          const val = Number(e.target.value);
          setStorageContribution(val);
          setContribution({ diskSpaceGB: val });
        }}
        className="w-full mb-2 accent-primary-500"
      />
      <div className="flex justify-between text-xs text-[var(--text-muted)] mb-6">
        <span>1 GB</span>
        <span>25 GB</span>
        <span>50 GB</span>
      </div>

      <p className="text-xs text-[var(--text-muted)] text-center mb-6">
        You can adjust this anytime from your dashboard.
      </p>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
