import React from "react";
import { useOnboardingStore } from "../../stores/onboardingStore";
import { Cloud } from "lucide-react";

const TOTAL_STEPS = 7;

interface OnboardingLayoutProps {
  children: React.ReactNode;
}

export function OnboardingLayout({ children }: OnboardingLayoutProps) {
  const currentStep = useOnboardingStore((s) => s.currentStep);

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-[var(--bg-primary)] p-8">
      <div className="w-full max-w-lg">
        {/* Logo */}
        <div className="flex items-center justify-center gap-2 mb-4">
          <Cloud className="w-6 h-6 text-primary-500" />
          <span className="text-lg font-bold text-[var(--text-primary)]">
            Citinet Setup
          </span>
        </div>

        {/* Dot indicator */}
        <div className="flex items-center justify-center gap-2 mb-8">
          {Array.from({ length: TOTAL_STEPS }).map((_, i) => (
            <div
              key={i}
              className={`h-1.5 rounded-full transition-all duration-300 ${
                i === currentStep
                  ? "w-6 bg-primary-500"
                  : i < currentStep
                  ? "w-1.5 bg-accent-500"
                  : "w-1.5 bg-surface-300 dark:bg-surface-700"
              }`}
            />
          ))}
        </div>

        {/* Content card */}
        <div className="bg-[var(--bg-card)] border border-[var(--border-color)] rounded-2xl p-8 shadow-lg transition-all duration-300">
          {children}
        </div>
      </div>
    </div>
  );
}
