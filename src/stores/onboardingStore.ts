import { create } from "zustand";

interface OnboardingState {
  currentStep: number;
  displayName: string;
  avatarUrl: string;
  storageContribution: number;
  keyPairGenerated: boolean;
  networkCheckPassed: boolean;
  setStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;
  setDisplayName: (name: string) => void;
  setAvatarUrl: (url: string) => void;
  setStorageContribution: (gb: number) => void;
  setKeyPairGenerated: (v: boolean) => void;
  setNetworkCheckPassed: (v: boolean) => void;
  reset: () => void;
}

const TOTAL_STEPS = 7;

export const useOnboardingStore = create<OnboardingState>((set) => ({
  currentStep: 0,
  displayName: "",
  avatarUrl: "",
  storageContribution: 5,
  keyPairGenerated: false,
  networkCheckPassed: false,
  setStep: (step) => set({ currentStep: Math.min(step, TOTAL_STEPS - 1) }),
  nextStep: () =>
    set((s) => ({ currentStep: Math.min(s.currentStep + 1, TOTAL_STEPS - 1) })),
  prevStep: () =>
    set((s) => ({ currentStep: Math.max(s.currentStep - 1, 0) })),
  setDisplayName: (displayName) => set({ displayName }),
  setAvatarUrl: (avatarUrl) => set({ avatarUrl }),
  setStorageContribution: (storageContribution) => set({ storageContribution }),
  setKeyPairGenerated: (keyPairGenerated) => set({ keyPairGenerated }),
  setNetworkCheckPassed: (networkCheckPassed) => set({ networkCheckPassed }),
  reset: () =>
    set({
      currentStep: 0,
      displayName: "",
      avatarUrl: "",
      storageContribution: 5,
      keyPairGenerated: false,
      networkCheckPassed: false,
    }),
}));
