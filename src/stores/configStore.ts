import { create } from "zustand";
import { persist } from "zustand/middleware";

export type NodeType = 'hub' | 'client' | 'personal';

export interface ResourceContribution {
  diskSpaceGB: number;
  bandwidthMbps: number;
  cpuPercent: number;
}

export interface NodeConfig {
  nodeType: NodeType;
  nodeName: string;
  installPath: string;
  autoStart: boolean;
  contribution: ResourceContribution;
  isConfigured: boolean;
}

interface ConfigState extends NodeConfig {
  setNodeType: (type: NodeType) => void;
  setNodeName: (name: string) => void;
  setInstallPath: (path: string) => void;
  setAutoStart: (enabled: boolean) => void;
  setContribution: (contribution: Partial<ResourceContribution>) => void;
  setConfigured: (configured: boolean) => void;
  reset: () => void;
}

const defaultConfig: NodeConfig = {
  nodeType: 'client',
  nodeName: '',
  installPath: 'C:\\Program Files\\Citinet',
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
      setNodeType: (nodeType) => set({ nodeType }),
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
