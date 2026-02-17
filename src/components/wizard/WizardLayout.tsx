import React from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Cloud } from "lucide-react";

const TOTAL_STEPS = 10;

const STEP_LABELS = [
  "Welcome",
  "License",
  "Identity",
  "Location",
  "Contribute",
  "Service",
  "Admin",
  "Installing",
  "Tunnel",
  "Complete",
];

interface WizardLayoutProps {
  children: React.ReactNode;
}

export function WizardLayout({ children }: WizardLayoutProps) {
  const currentStep = useWizardStore((s) => s.currentStep);
  const progress = (currentStep / (TOTAL_STEPS - 1)) * 100;
  const isBookend = currentStep === 0 || currentStep === TOTAL_STEPS - 1;
  const progressContainerRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    if (progressContainerRef.current) {
      progressContainerRef.current.style.setProperty('--progress-width', `${progress}%`);
    }
  }, [progress]);

  return (
    <div className="min-h-screen flex flex-col bg-[var(--bg-primary)]">
      {/* Top bar: progress + step info */}
      {!isBookend && (
        <div className="shrink-0" ref={progressContainerRef}>
          {/* Thin progress bar */}
          <div className="h-1 bg-surface-200 dark:bg-surface-800">
            <div className="h-full bg-primary-500 transition-all duration-500 ease-out progress-bar" />
          </div>

          {/* Step label bar */}
          <div className="flex items-center justify-between px-6 py-3">
            <div className="flex items-center gap-2">
              <Cloud className="w-5 h-5 text-primary-500" />
              <span className="text-sm font-semibold text-[var(--text-primary)]">
                Citinet
              </span>
            </div>
            <span className="text-xs text-[var(--text-muted)]">
              {STEP_LABELS[currentStep]}
              <span className="ml-2 text-[var(--text-muted)]/60">
                {currentStep + 1}/{TOTAL_STEPS}
              </span>
            </span>
          </div>
        </div>
      )}

      {/* Content area */}
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="w-full max-w-md">
          {isBookend ? (
            // Welcome & Complete: no card wrapper, more breathing room
            <div>
              {currentStep === 0 && (
                <div className="flex items-center justify-center gap-2.5 mb-10">
                  <Cloud className="w-9 h-9 text-primary-500" />
                  <span className="text-2xl font-bold text-[var(--text-primary)]">
                    Citinet
                  </span>
                </div>
              )}
              {children}
            </div>
          ) : (
            // Middle steps: card wrapper
            <div className="bg-[var(--bg-card)] border border-[var(--border-color)] rounded-2xl p-8 shadow-lg">
              {children}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
