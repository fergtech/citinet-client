import { useState, useEffect, useRef } from "react";
import { useAppStore, ThemeMode } from "../../stores/appStore";
import { useConfigStore } from "../../stores/configStore";
import { Card } from "../ui/Card";
import { Toggle } from "../ui/Toggle";
import { Sun, Moon, Monitor, HardDrive, Wifi, Cpu, AlertCircle, Activity } from "lucide-react";
import { CitinetAPI, HardwareInfo } from "../../api/tauri";
import { useFeatureFlags, PROFILE_FLAGS } from "../../lib/features";

export function SettingsPanel() {
  const { theme, setTheme } = useAppStore();
  const { contribution, setContribution, nodeType } = useConfigStore();
  const featureFlags = useFeatureFlags();
  const [hwInfo, setHwInfo] = useState<HardwareInfo | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    CitinetAPI.getHardwareInfo()
      .then(setHwInfo)
      .catch(console.error);
  }, []);

  // Debounced persist of resource limits to backend
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      CitinetAPI.updateResourceLimits(
        contribution.diskSpaceGB,
        contribution.bandwidthMbps,
        contribution.cpuPercent,
      ).catch(console.error);
    }, 500);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [contribution.diskSpaceGB, contribution.bandwidthMbps, contribution.cpuPercent]);

  const themes: { mode: ThemeMode; label: string; icon: React.ElementType }[] = [
    { mode: "light", label: "Light", icon: Sun },
    { mode: "dark", label: "Dark", icon: Moon },
    { mode: "system", label: "System", icon: Monitor },
  ];

  const maxDiskGB = hwInfo ? Math.floor(hwInfo.total_disk_gb * 0.5) : 100; // Max 50% of total disk
  const maxBandwidthMbps = 100; // Reasonable default
  const maxCpuPercent = 50; // Max 50% CPU

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">Settings</h2>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Appearance
        </h3>
        <div className="flex gap-3">
          {themes.map(({ mode, label, icon: Icon }) => (
            <button
              key={mode}
              onClick={() => setTheme(mode)}
              className={`flex-1 flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all duration-200 ${
                theme === mode
                  ? "border-primary-500 bg-primary-500/5"
                  : "border-[var(--border-color)] hover:border-surface-300 dark:hover:border-surface-600"
              }`}
            >
              <Icon
                className={`w-5 h-5 ${
                  theme === mode ? "text-primary-500" : "text-[var(--text-secondary)]"
                }`}
              />
              <span
                className={`text-sm ${
                  theme === mode
                    ? "text-primary-500 font-medium"
                    : "text-[var(--text-secondary)]"
                }`}
              >
                {label}
              </span>
            </button>
          ))}
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Resource Contribution
        </h3>
        <p className="text-xs text-[var(--text-muted)] mb-4">
          Configure how much of your device's resources to contribute to the Citinet network
        </p>
        
        <div className="space-y-6">
          {/* Disk Space */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <HardDrive className="w-4 h-4 text-accent-500" />
              <label htmlFor="settings-disk" className="text-sm font-medium text-[var(--text-primary)]">
                Storage Space
              </label>
            </div>
            <input
              id="settings-disk"
              type="range"
              min="5"
              max={maxDiskGB}
              step="5"
              value={contribution.diskSpaceGB}
              onChange={(e) => setContribution({ diskSpaceGB: parseInt(e.target.value) })}
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>5 GB</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.diskSpaceGB} GB
              </span>
              <span>{maxDiskGB} GB</span>
            </div>
          </div>

          {/* Bandwidth */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wifi className="w-4 h-4 text-primary-500" />
              <label htmlFor="settings-bandwidth" className="text-sm font-medium text-[var(--text-primary)]">
                Bandwidth Limit
              </label>
            </div>
            <input
              id="settings-bandwidth"
              type="range"
              min="1"
              max={maxBandwidthMbps}
              step="1"
              value={contribution.bandwidthMbps}
              onChange={(e) => setContribution({ bandwidthMbps: parseInt(e.target.value) })}
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>1 Mbps</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.bandwidthMbps} Mbps
              </span>
              <span>{maxBandwidthMbps} Mbps</span>
            </div>
          </div>

          {/* CPU */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Cpu className="w-4 h-4 text-purple-500" />
              <label htmlFor="settings-cpu" className="text-sm font-medium text-[var(--text-primary)]">
                CPU Usage
              </label>
            </div>
            <input
              id="settings-cpu"
              type="range"
              min="5"
              max={maxCpuPercent}
              step="5"
              value={contribution.cpuPercent}
              onChange={(e) => setContribution({ cpuPercent: parseInt(e.target.value) })}
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>5%</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.cpuPercent}%
              </span>
              <span>{maxCpuPercent}%</span>
            </div>
          </div>

          <div className="flex items-start gap-2 p-3 rounded-lg bg-primary-500/5 border border-primary-500/20">
            <AlertCircle className="w-4 h-4 text-primary-500 mt-0.5 shrink-0" />
            <p className="text-xs text-[var(--text-secondary)]">
              These limits protect your device. Citinet will never exceed these configured resources.
            </p>
          </div>
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          ISP Sharing
        </h3>
        <div className="space-y-4">
          <div className="flex items-start gap-2 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
            <AlertCircle className="w-4 h-4 text-[var(--text-muted)] mt-0.5 shrink-0" />
            <div>
              <p className="text-sm text-[var(--text-primary)] font-medium">Coming Soon</p>
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Internet bandwidth sharing will be available in Phase 3 after local mesh networking is established.
              </p>
            </div>
          </div>
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Node Configuration
        </h3>
        <div className="space-y-4">
          <Toggle
            checked={true}
            onChange={() => {}}
            label="Auto-start on boot"
            description="Launch Citinet when your system starts"
          />
          <Toggle
            checked={true}
            onChange={() => {}}
            label="Background mode"
            description="Keep the node running when the window is closed"
          />
          <Toggle
            checked={false}
            onChange={() => {}}
            label="Developer mode"
            description="Show advanced metrics and debug information"
          />
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          System Information
        </h3>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">Version</span>
            <span className="text-[var(--text-primary)] font-medium">0.1.0</span>
          </div>
          {hwInfo && (
            <>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">Hostname</span>
                <span className="text-[var(--text-primary)] font-medium">{hwInfo.hostname}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">OS</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.os_name} {hwInfo.os_version}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">CPU Cores</span>
                <span className="text-[var(--text-primary)] font-medium">{hwInfo.cpu_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">Total Memory</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.total_memory_gb.toFixed(1)} GB
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">Total Storage</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.total_disk_gb.toFixed(0)} GB
                </span>
              </div>
              {hwInfo.is_raspberry_pi && (
                <div className="flex items-center gap-2 mt-2 p-2 rounded bg-accent-500/10 border border-accent-500/20">
                  <span className="text-sm text-accent-500 font-medium">
                    🫐 Raspberry Pi Detected — Optimized for Hub Node
                  </span>
                </div>
              )}
            </>
          )}
        </div>
      </Card>

      <Card>
        <div className="flex items-center gap-2 mb-4">
          <Activity className="w-4 h-4 text-primary-500" />
          <h3 className="text-sm font-medium text-[var(--text-secondary)]">
            Diagnostics
          </h3>
        </div>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">Profile</span>
            <span className="text-[var(--text-primary)] font-medium capitalize">{nodeType}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">App Version</span>
            <span className="text-[var(--text-primary)] font-medium">0.1.0</span>
          </div>
          {hwInfo && (
            <div className="flex justify-between">
              <span className="text-[var(--text-secondary)]">Platform</span>
              <span className="text-[var(--text-primary)] font-medium">
                {hwInfo.os_name} {hwInfo.os_version}
              </span>
            </div>
          )}
          <div className="mt-3">
            <span className="text-[var(--text-secondary)] text-xs">Enabled Features</span>
            <div className="flex flex-wrap gap-1.5 mt-1.5">
              {(Object.entries(featureFlags) as [string, boolean][])
                .filter(([, enabled]) => enabled)
                .map(([flag]) => (
                  <span
                    key={flag}
                    className="px-2 py-0.5 text-xs rounded-full bg-primary-500/10 text-primary-500 font-medium"
                  >
                    {flag.replace(/_/g, ' ')}
                  </span>
                ))}
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
