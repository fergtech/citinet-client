import React from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Cloud } from "lucide-react";

const STEP_LABELS = [
  "Welcome",
  "License",
  "Location",
  "Service",
  "Installing",
  "Complete",
];

interface WizardLayoutProps {
  children: React.ReactNode;
}

export function WizardLayout({ children }: WizardLayoutProps) {
  const currentStep = useWizardStore((s) => s.currentStep);

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-[var(--bg-primary)] p-8">
      <div className="w-full max-w-lg">
        {/* Logo */}
        <div className="flex items-center justify-center gap-2 mb-8">
          <Cloud className="w-8 h-8 text-primary-500" />
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            CitiNet
          </span>
        </div>

        {/* Step indicator */}
        <div className="flex items-center justify-between mb-8 px-4">
          {STEP_LABELS.map((label, i) => (
            <div key={label} className="flex items-center">
              <div className="flex flex-col items-center">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium transition-all duration-300 ${
                    i < currentStep
                      ? "bg-accent-500 text-white"
                      : i === currentStep
                      ? "bg-primary-500 text-white ring-4 ring-primary-200 dark:ring-primary-800"
                      : "bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)]"
                  }`}
                >
                  {i < currentStep ? "\u2713" : i + 1}
                </div>
                <span
                  className={`text-[10px] mt-1 ${
                    i === currentStep
                      ? "text-primary-500 font-medium"
                      : "text-[var(--text-muted)]"
                  }`}
                >
                  {label}
                </span>
              </div>
              {i < STEP_LABELS.length - 1 && (
                <div
                  className={`w-6 h-0.5 mx-1 mt-[-12px] transition-colors duration-300 ${
                    i < currentStep
                      ? "bg-accent-500"
                      : "bg-surface-200 dark:bg-surface-700"
                  }`}
                />
              )}
            </div>
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
