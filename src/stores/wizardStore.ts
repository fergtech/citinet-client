import { create } from "zustand";

interface WizardState {
  currentStep: number;
  licenseAccepted: boolean;
  privacyAccepted: boolean;
  installPath: string;
  autoStart: boolean;
  installProgress: number;
  installComplete: boolean;
  nodeName: string;
  nodeSlug: string;
  cfApiToken: string;
  tunnelSkipped: boolean;
  storageContribution: number;
  adminUsername: string;
  adminEmail: string;
  adminPassword: string;
  createdUser: { user_id: string; username: string; email: string; is_admin: boolean; created_at: string; updated_at: string } | null;
  setStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;
  setLicenseAccepted: (v: boolean) => void;
  setPrivacyAccepted: (v: boolean) => void;
  setInstallPath: (path: string) => void;
  setAutoStart: (v: boolean) => void;
  setInstallProgress: (v: number) => void;
  setInstallComplete: (v: boolean) => void;
  setNodeName: (name: string) => void;
  setNodeSlug: (slug: string) => void;
  setCfApiToken: (token: string) => void;
  setTunnelSkipped: (v: boolean) => void;
  setStorageContribution: (gb: number) => void;
  reset: () => void;
}

const TOTAL_STEPS = 10;

export const useWizardStore = create<WizardState>((set) => ({
  currentStep: 0,
  licenseAccepted: false,
  privacyAccepted: false,
  installPath: "",  // Will be set by LocationStep from Tauri
  autoStart: true,
  installProgress: 0,
  installComplete: false,
  nodeName: "",
  nodeSlug: "",
  cfApiToken: "",
  tunnelSkipped: false,
  storageContribution: 10,
  adminUsername: "",
  adminEmail: "",
  adminPassword: "",
  createdUser: null,
  setStep: (step) => set({ currentStep: Math.min(step, TOTAL_STEPS - 1) }),
  nextStep: () =>
    set((s) => ({ currentStep: Math.min(s.currentStep + 1, TOTAL_STEPS - 1) })),
  prevStep: () =>
    set((s) => ({ currentStep: Math.max(s.currentStep - 1, 0) })),
  setLicenseAccepted: (licenseAccepted) => set({ licenseAccepted }),
  setPrivacyAccepted: (privacyAccepted) => set({ privacyAccepted }),
  setInstallPath: (installPath) => set({ installPath }),
  setAutoStart: (autoStart) => set({ autoStart }),
  setInstallProgress: (installProgress) => set({ installProgress }),
  setInstallComplete: (installComplete) => set({ installComplete }),
  setNodeName: (nodeName) => set({ nodeName }),
  setNodeSlug: (nodeSlug) => set({ nodeSlug }),
  setCfApiToken: (cfApiToken) => set({ cfApiToken }),
  setTunnelSkipped: (tunnelSkipped) => set({ tunnelSkipped }),
  setStorageContribution: (storageContribution) => set({ storageContribution }),
  reset: () =>
    set({
      currentStep: 0,
      licenseAccepted: false,
      privacyAccepted: false,
      installPath: "",  // Will be set by LocationStep from Tauri
      autoStart: true,
      installProgress: 0,
      installComplete: false,
      nodeName: "",
      nodeSlug: "",
      cfApiToken: "",
      tunnelSkipped: false,
      storageContribution: 10,
      adminUsername: "",
      adminEmail: "",
      adminPassword: "",
      createdUser: null,
    }),
}));
