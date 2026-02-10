import { useState } from "react";
import { useOnboardingStore } from "../../stores/onboardingStore";
import { Button } from "../ui/Button";
import { KeyRound, Shield, Download } from "lucide-react";

export function IdentityKeys() {
  const { keyPairGenerated, setKeyPairGenerated, nextStep, prevStep } =
    useOnboardingStore();
  const [generating, setGenerating] = useState(false);

  const handleGenerate = async () => {
    setGenerating(true);
    // Simulate key generation
    await new Promise((r) => setTimeout(r, 1500));
    setKeyPairGenerated(true);
    setGenerating(false);
  };

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Identity & Encryption
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Generate your cryptographic keypair for secure communication.
      </p>

      <div className="bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] rounded-lg p-4 mb-4">
        <div className="flex items-start gap-3">
          <Shield className="w-5 h-5 text-accent-500 mt-0.5" />
          <div className="text-sm text-[var(--text-secondary)]">
            <p className="font-medium text-[var(--text-primary)] mb-1">
              End-to-end encryption
            </p>
            <p>
              Your keys are generated locally and never leave your device.
              All data is encrypted before being shared on the network.
            </p>
          </div>
        </div>
      </div>

      {!keyPairGenerated ? (
        <Button
          onClick={handleGenerate}
          disabled={generating}
          className="w-full mb-4"
        >
          <KeyRound className="w-4 h-4 mr-2" />
          {generating ? "Generating keypair..." : "Generate Keypair"}
        </Button>
      ) : (
        <div className="space-y-3 mb-4">
          <div className="flex items-center gap-2 text-accent-500 justify-center">
            <KeyRound className="w-5 h-5" />
            <span className="text-sm font-medium">Keypair generated</span>
          </div>
          <Button variant="secondary" className="w-full" onClick={() => {}}>
            <Download className="w-4 h-4 mr-2" />
            Download Recovery Kit (optional)
          </Button>
        </div>
      )}

      <div className="flex gap-3 mt-6">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button
          onClick={nextStep}
          disabled={!keyPairGenerated}
          className="flex-1"
        >
          Continue
        </Button>
      </div>
    </div>
  );
}
