import { useOnboardingStore } from "../../stores/onboardingStore";
import { Button } from "../ui/Button";
import { Heart, Users, Globe } from "lucide-react";

export function CommunityPanel() {
  const { nextStep, prevStep } = useOnboardingStore();

  return (
    <div className="text-center">
      <Heart className="w-12 h-12 text-red-400 mx-auto mb-4" />
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        You're Making a Difference
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-8">
        By joining Citinet, you're helping build a more equitable internet.
      </p>

      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="flex flex-col items-center gap-1">
          <Users className="w-6 h-6 text-primary-500 mb-1" />
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            12.4K
          </span>
          <span className="text-xs text-[var(--text-secondary)]">
            Active nodes
          </span>
        </div>
        <div className="flex flex-col items-center gap-1">
          <Globe className="w-6 h-6 text-accent-500 mb-1" />
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            42
          </span>
          <span className="text-xs text-[var(--text-secondary)]">
            Countries
          </span>
        </div>
        <div className="flex flex-col items-center gap-1">
          <Heart className="w-6 h-6 text-red-400 mb-1" />
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            1.2 PB
          </span>
          <span className="text-xs text-[var(--text-secondary)]">
            Data shared
          </span>
        </div>
      </div>

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
