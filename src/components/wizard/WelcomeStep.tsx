import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { Globe, Shield, Users } from "lucide-react";

export function WelcomeStep() {
  const nextStep = useWizardStore((s) => s.nextStep);

  return (
    <div className="text-center">
      <h1 className="text-2xl font-bold mb-2 text-[var(--text-primary)]">
        Welcome to Citinet
      </h1>
      <p className="text-[var(--text-secondary)] mb-8">
        The People-Powered Cloud. Share a little, gain a lot.
      </p>

      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="flex flex-col items-center gap-2 p-3">
          <Globe className="w-8 h-8 text-primary-500" />
          <span className="text-xs text-[var(--text-secondary)]">
            Decentralized
          </span>
        </div>
        <div className="flex flex-col items-center gap-2 p-3">
          <Shield className="w-8 h-8 text-accent-500" />
          <span className="text-xs text-[var(--text-secondary)]">
            Privacy-First
          </span>
        </div>
        <div className="flex flex-col items-center gap-2 p-3">
          <Users className="w-8 h-8 text-primary-400" />
          <span className="text-xs text-[var(--text-secondary)]">
            Community-Owned
          </span>
        </div>
      </div>

      <Button onClick={nextStep} size="lg" className="w-full">
        Get Started
      </Button>
    </div>
  );
}
