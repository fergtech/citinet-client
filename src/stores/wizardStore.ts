import { create } from "zustand";

export type NodeType = 'hub' | 'client' | 'personal';

interface WizardState {
  currentStep: number;
  licenseAccepted: boolean;
  privacyAccepted: boolean;
  installPath: string;
  autoStart: boolean;
  nodeType: NodeType;
  installProgress: number;
  installComplete: boolean;
  setStep: (step: number) => void;
  setNodeType: (type: NodeType) => void;
  nextStep: () => void;
  prevStep: () => void;
  setLicenseAccepted: (v: boolean) => void;
  setPrivacyAccepted: (v: boolean) => void;
  setInstallPath: (path: string) => void;
  setAutoStart: (v: boolean) => void;
  setInstallProgress: (v: number) => void;
  setInstallComplete: (v: boolean) => void;
  reset: () => void;
}

const TOTAL_STEPS = 6;

export const useWizardStore = create<WizardState>((set) => ({
  currentStep: 0,
  licenseAccepted: false,
  privacyAccepted: false,
  installPath: "C:\\Program Files\\Citinet",
  autoStart: true,
  nodeType: 'client',
  installProgress: 0,
  installComplete: false,
  setStep: (step) => set({ currentStep: Math.min(step, TOTAL_STEPS - 1) }),
  nextStep: () =>
    set((s) => ({ currentStep: Math.min(s.currentStep + 1, TOTAL_STEPS - 1) })),
  prevStep: () =>
    set((s) => ({ currentStep: Math.max(s.currentStep - 1, 0) })),
  setLicenseAccepted: (licenseAccepted) => set({ licenseAccepted }),
  setPrivacyAccepted: (privacyAccepted) => set({ privacyAccepted }),
  setInstallPath: (installPath) => set({ installPath }),
  setAutoStart: (autoStart) => set({ autoStart }),
  setNodeType: (nodeType) => set({ nodeType }),
  setInstallProgress: (installProgress) => set({ installProgress }),
  setInstallComplete: (installComplete) => set({ installComplete }),
  reset: () =>
    set({
      currentStep: 0,
      licenseAccepted: false,
      privacyAccepted: false,
      installPath: "C:\\Program Files\\Citinet",
      autoStart: true,
      nodeType: 'client',
      installProgress: 0,
      installComplete: false,
    }),
}));
