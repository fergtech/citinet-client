import { type ReactNode } from 'react';
import { useConfigStore, type NodeType } from '../stores/configStore';

export type FeatureFlag =
  | 'hub_mode'
  | 'file_sync'
  | 'discussions'
  | 'contribution'
  | 'admin_panel'
  | 'discover_tab'
  | 'local_storage'
  | 'auth'
  | 'diagnostics';

export const PROFILE_FLAGS: Record<NodeType, FeatureFlag[]> = {
  hub:      ['hub_mode', 'file_sync', 'discussions', 'admin_panel', 'discover_tab', 'local_storage', 'auth', 'diagnostics'],
  client:   ['file_sync', 'discussions', 'contribution', 'discover_tab', 'local_storage', 'auth', 'diagnostics'],
  personal: ['file_sync', 'discussions', 'discover_tab', 'local_storage', 'auth', 'diagnostics'],
};

export function resolveFlags(nodeType: NodeType): Record<FeatureFlag, boolean> {
  const enabled = PROFILE_FLAGS[nodeType];
  return {
    hub_mode: enabled.includes('hub_mode'),
    file_sync: enabled.includes('file_sync'),
    discussions: enabled.includes('discussions'),
    contribution: enabled.includes('contribution'),
    admin_panel: enabled.includes('admin_panel'),
    discover_tab: enabled.includes('discover_tab'),
    local_storage: enabled.includes('local_storage'),
    auth: enabled.includes('auth'),
    diagnostics: enabled.includes('diagnostics'),
  };
}

export function useFeatureFlags(): Record<FeatureFlag, boolean> {
  const nodeType = useConfigStore((s) => s.nodeType);
  return resolveFlags(nodeType);
}

export function useFeature(flag: FeatureFlag): boolean {
  const nodeType = useConfigStore((s) => s.nodeType);
  return PROFILE_FLAGS[nodeType].includes(flag);
}

interface FeatureGateProps {
  flag: FeatureFlag;
  children: ReactNode;
  fallback?: ReactNode;
}

export function FeatureGate({ flag, children, fallback = null }: FeatureGateProps) {
  const enabled = useFeature(flag);
  return enabled ? children : fallback;
}
