import { useOnboardingStore } from "../../stores/onboardingStore";
import { Button } from "../ui/Button";
import { Sparkles } from "lucide-react";

export function WelcomePanel() {
  const nextStep = useOnboardingStore((s) => s.nextStep);

  return (
    <div className="text-center">
      <Sparkles className="w-12 h-12 text-primary-500 mx-auto mb-4" />
      <h1 className="text-2xl font-bold mb-2 text-[var(--text-primary)]">
        Welcome to the Network
      </h1>
      <p className="text-[var(--text-secondary)] mb-4">
        You're about to join a community of people building the internet's
        future — together.
      </p>
      <p className="text-sm text-[var(--text-muted)] mb-8">
        Let's set up your node in a few quick steps.
      </p>

      <Button onClick={nextStep} size="lg" className="w-full">
        Let's Go
      </Button>
    </div>
  );
}
