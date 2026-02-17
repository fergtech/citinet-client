import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface ResourceContribution {
  diskSpaceGB: number;
  bandwidthMbps: number;
  cpuPercent: number;
}

export interface NodeConfig {
  nodeName: string;
  installPath: string;
  autoStart: boolean;
  contribution: ResourceContribution;
  isConfigured: boolean;
}

interface ConfigState extends NodeConfig {
  nodeType: 'hub'; // Read-only, always hub for now
  setNodeName: (name: string) => void;
  setInstallPath: (path: string) => void;
  setAutoStart: (enabled: boolean) => void;
  setContribution: (contribution: Partial<ResourceContribution>) => void;
  setConfigured: (configured: boolean) => void;
  reset: () => void;
}

const defaultConfig: NodeConfig = {
  nodeName: '',
  installPath: '',  // Will be set during installation
  autoStart: true,
  contribution: {
    diskSpaceGB: 10,
    bandwidthMbps: 5,
    cpuPercent: 25,
  },
  isConfigured: false,
};

export const useConfigStore = create<ConfigState>()(
  persist(
    (set) => ({
      ...defaultConfig,
      nodeType: 'hub' as const,
      setNodeName: (nodeName) => set({ nodeName }),
      setInstallPath: (installPath) => set({ installPath }),
      setAutoStart: (autoStart) => set({ autoStart }),
      setContribution: (contribution) =>
        set((state) => ({
          contribution: { ...state.contribution, ...contribution },
        })),
      setConfigured: (isConfigured) => set({ isConfigured }),
      reset: () => set(defaultConfig),
    }),
    {
      name: 'citinet-config',
    }
  )
);
