import { useOnboardingStore } from "../../stores/onboardingStore";
import { Button } from "../ui/Button";
import { User } from "lucide-react";

export function CreateProfile() {
  const { displayName, setDisplayName, nextStep, prevStep } =
    useOnboardingStore();

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Create Your Profile
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Choose a display name for the Citinet network.
      </p>

      <div className="flex flex-col items-center mb-6">
        <div className="w-20 h-20 rounded-full bg-primary-100 dark:bg-primary-800 flex items-center justify-center mb-4">
          {displayName ? (
            <span className="text-2xl font-bold text-primary-500">
              {displayName.charAt(0).toUpperCase()}
            </span>
          ) : (
            <User className="w-8 h-8 text-primary-400" />
          )}
        </div>

        <input
          type="text"
          placeholder="Display name"
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
          className="w-full px-4 py-2.5 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)] text-center focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          maxLength={30}
        />
        <p className="text-xs text-[var(--text-muted)] mt-2">
          This is how others will see you on the network.
        </p>
      </div>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button
          onClick={nextStep}
          disabled={!displayName.trim()}
          className="flex-1"
        >
          Continue
        </Button>
      </div>
    </div>
  );
}
