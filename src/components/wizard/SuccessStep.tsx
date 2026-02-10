import { Button } from "../ui/Button";
import { CheckCircle2, PartyPopper } from "lucide-react";
import { useAppStore } from "../../stores/appStore";

export function SuccessStep() {
  const setPhase = useAppStore((s) => s.setPhase);

  return (
    <div className="text-center">
      <div className="flex justify-center mb-4">
        <div className="relative">
          <CheckCircle2 className="w-16 h-16 text-accent-500 animate-bounce" />
          <PartyPopper className="w-6 h-6 text-primary-400 absolute -top-1 -right-2" />
        </div>
      </div>

      <h2 className="text-2xl font-bold mb-2 text-[var(--text-primary)]">
        Installation Complete!
      </h2>
      <p className="text-[var(--text-secondary)] mb-8">
        Citinet has been installed successfully. Let's get you set up.
      </p>

      <Button onClick={() => setPhase("onboarding")} size="lg" className="w-full">
        Launch Citinet
      </Button>
    </div>
  );
}
