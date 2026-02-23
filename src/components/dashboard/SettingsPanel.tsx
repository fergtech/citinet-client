import { useState, useEffect, useRef } from "react";
import { useAppStore, ThemeMode } from "../../stores/appStore";
import { useConfigStore } from "../../stores/configStore";
import { Card } from "../ui/Card";
import { ProgressBar } from "../ui/ProgressBar";
import { Toggle } from "../ui/Toggle";
import {
  Sun,
  Moon,
  Monitor,
  HardDrive,
  Wifi,
  Cpu,
  AlertCircle,
  Activity,
  FolderOpen,
  ArrowRight,
  Loader2,
  CheckCircle,
  RefreshCw,
  Download,
  Trash2,
} from "lucide-react";
import { Button } from "../ui/Button";
import {
  CitinetAPI,
  HardwareInfo,
  DriveSpace,
  StorageStatus,
  SystemMetrics,
} from "../../api/tauri";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const APP_VERSION = "0.1.0";

function formatSpeed(mbps: number): string {
  if (mbps < 0.1) return `${(mbps * 1000).toFixed(0)} Kbps`;
  if (mbps < 1) return `${mbps.toFixed(2)} Mbps`;
  return `${mbps.toFixed(1)} Mbps`;
}

export function SettingsPanel() {
  const { theme, setTheme } = useAppStore();
  const { contribution, setContribution, nodeType, autoStart, setAutoStart, backgroundMode, setBackgroundMode } = useConfigStore();
  const [hwInfo, setHwInfo] = useState<HardwareInfo | null>(null);
  const [driveSpace, setDriveSpace] = useState<DriveSpace | null>(null);
  const [storageStatus, setStorageStatus] = useState<StorageStatus | null>(
    null
  );
  const [liveMetrics, setLiveMetrics] = useState<SystemMetrics | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Relocation state
  const [newPath, setNewPath] = useState("");
  const [relocating, setRelocating] = useState(false);
  const [relocateResult, setRelocateResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);

  // Update state
  type UpdateStatus = "idle" | "checking" | "up-to-date" | "available" | "downloading" | "error";
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>("idle");
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [updateProgress, setUpdateProgress] = useState(0);

  // Factory reset state
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    CitinetAPI.getHardwareInfo().then(setHwInfo).catch(console.error);
    CitinetAPI.getInstallDriveSpace().then(setDriveSpace).catch(console.error);
    CitinetAPI.getStorageStatus().then(setStorageStatus).catch(console.error);
  }, []);

  // Poll live system metrics every 3s
  useEffect(() => {
    const fetch = () =>
      CitinetAPI.getSystemMetrics().then(setLiveMetrics).catch(() => {});
    fetch();
    const interval = setInterval(fetch, 3000);
    return () => clearInterval(interval);
  }, []);

  // Debounced persist of resource limits to backend
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      CitinetAPI.updateResourceLimits(
        contribution.diskSpaceGB,
        contribution.bandwidthMbps,
        contribution.cpuPercent
      ).catch(console.error);
    }, 500);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [
    contribution.diskSpaceGB,
    contribution.bandwidthMbps,
    contribution.cpuPercent,
  ]);

  const themes: {
    mode: ThemeMode;
    label: string;
    icon: React.ElementType;
  }[] = [
    { mode: "light", label: "Light", icon: Sun },
    { mode: "dark", label: "Dark", icon: Moon },
    { mode: "system", label: "System", icon: Monitor },
  ];

  const maxDiskGB = driveSpace
    ? Math.floor(driveSpace.total_gb * 0.5)
    : hwInfo
      ? Math.floor(hwInfo.total_disk_gb * 0.5)
      : 100;
  const maxBandwidthMbps = 100;
  const maxCpuPercent = 50;

  const storageUsedPercent =
    storageStatus && contribution.diskSpaceGB > 0
      ? Math.min(
          100,
          Math.round((storageStatus.used_gb / contribution.diskSpaceGB) * 100)
        )
      : 0;

  const handleCheckForUpdates = async () => {
    setUpdateStatus("checking");
    setUpdateVersion(null);
    try {
      const update = await check();
      if (update?.available) {
        setUpdateStatus("available");
        setUpdateVersion(update.version);
      } else {
        setUpdateStatus("up-to-date");
      }
    } catch {
      setUpdateStatus("error");
    }
  };

  const handleInstallUpdate = async () => {
    setUpdateStatus("downloading");
    setUpdateProgress(0);
    try {
      const update = await check();
      if (!update?.available) return;
      let downloaded = 0;
      let total = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setUpdateProgress(total > 0 ? Math.round((downloaded / total) * 100) : 0);
        }
      });
      await relaunch();
    } catch {
      setUpdateStatus("error");
    }
  };

  const handleFactoryReset = async () => {
    if (!confirm(
      "Factory Reset will permanently delete all Citinet data:\n\n" +
      "• All stored files\n• All users and accounts\n• Node configuration\n• Tunnel settings\n\n" +
      "The app will restart and run the setup wizard fresh.\n\n" +
      "Your data directory will NOT be deleted from disk — only the Citinet database and stored files are cleared.\n\n" +
      "This cannot be undone. Continue?"
    )) return;

    setResetting(true);
    try {
      await CitinetAPI.factoryReset();
    } catch (err) {
      console.error("Factory reset failed:", err);
      setResetting(false);
    }
  };

  const handleRelocate = async () => {
    if (!newPath.trim()) return;
    if (
      !confirm(
        `This will copy all Citinet data to:\n${newPath}\n\nThe tunnel will be temporarily stopped. This may take a while for large storage.\n\nContinue?`
      )
    )
      return;

    setRelocating(true);
    setRelocateResult(null);

    try {
      const oldPath = await CitinetAPI.relocateStorage(newPath.trim());
      // Refresh storage info
      const [newDriveSpace, newStorageStatus] = await Promise.all([
        CitinetAPI.getInstallDriveSpace(),
        CitinetAPI.getStorageStatus(),
      ]);
      setDriveSpace(newDriveSpace);
      setStorageStatus(newStorageStatus);
      setNewPath("");
      setRelocateResult({
        success: true,
        message: `Migration complete. Old data backed up at: ${oldPath}`,
      });
    } catch (err) {
      setRelocateResult({
        success: false,
        message: err instanceof Error ? err.message : "Migration failed",
      });
    } finally {
      setRelocating(false);
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">
        Settings
      </h2>

      {/* Appearance */}
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
                  theme === mode
                    ? "text-primary-500"
                    : "text-[var(--text-secondary)]"
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

      {/* Resource Contribution */}
      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Resource Contribution
        </h3>
        <p className="text-xs text-[var(--text-muted)] mb-4">
          Configure how much of your device's resources to contribute to the
          Citinet network
        </p>

        <div className="space-y-6">
          {/* Disk Space */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <HardDrive className="w-4 h-4 text-accent-500" />
              <label
                htmlFor="settings-disk"
                className="text-sm font-medium text-[var(--text-primary)]"
              >
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
              onChange={(e) =>
                setContribution({ diskSpaceGB: parseInt(e.target.value) })
              }
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>5 GB</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.diskSpaceGB} GB
              </span>
              <span>{maxDiskGB} GB</span>
            </div>
            {/* Real Citinet usage */}
            {storageStatus && (
              <div className="mt-2 p-2.5 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
                <div className="flex justify-between text-xs mb-1.5">
                  <span className="text-[var(--text-secondary)]">
                    Used: {storageStatus.used_gb.toFixed(2)} GB of{" "}
                    {contribution.diskSpaceGB} GB
                  </span>
                  <span className="text-[var(--text-muted)]">
                    {storageStatus.file_count} file
                    {storageStatus.file_count !== 1 ? "s" : ""}
                  </span>
                </div>
                <ProgressBar
                  value={storageUsedPercent}
                  showPercent={false}
                  color="accent"
                />
                <div className="text-[10px] text-[var(--text-muted)] mt-1">
                  {storageUsedPercent}% of allocated storage used
                </div>
              </div>
            )}
          </div>

          {/* Bandwidth */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wifi className="w-4 h-4 text-primary-500" />
              <label
                htmlFor="settings-bandwidth"
                className="text-sm font-medium text-[var(--text-primary)]"
              >
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
              onChange={(e) =>
                setContribution({ bandwidthMbps: parseInt(e.target.value) })
              }
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>1 Mbps</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.bandwidthMbps} Mbps
              </span>
              <span>{maxBandwidthMbps} Mbps</span>
            </div>
            {/* Live system bandwidth */}
            {liveMetrics && (
              <div className="mt-2 flex items-center gap-3 text-xs text-[var(--text-muted)]">
                <span className="text-[var(--text-secondary)]">System:</span>
                <span>
                  &uarr; {formatSpeed(liveMetrics.network_up_mbps)}
                </span>
                <span>
                  &darr; {formatSpeed(liveMetrics.network_down_mbps)}
                </span>
              </div>
            )}
          </div>

          {/* CPU */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Cpu className="w-4 h-4 text-purple-500" />
              <label
                htmlFor="settings-cpu"
                className="text-sm font-medium text-[var(--text-primary)]"
              >
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
              onChange={(e) =>
                setContribution({ cpuPercent: parseInt(e.target.value) })
              }
              className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-[var(--text-muted)] mt-1">
              <span>5%</span>
              <span className="font-medium text-[var(--text-primary)]">
                {contribution.cpuPercent}%
              </span>
              <span>{maxCpuPercent}%</span>
            </div>
            {/* Live system CPU */}
            {liveMetrics && (
              <div className="mt-2 flex items-center gap-2 text-xs text-[var(--text-muted)]">
                <span className="text-[var(--text-secondary)]">System:</span>
                <span>{Math.round(liveMetrics.cpu_usage)}% current</span>
              </div>
            )}
          </div>

          <div className="flex items-start gap-2 p-3 rounded-lg bg-primary-500/5 border border-primary-500/20">
            <AlertCircle className="w-4 h-4 text-primary-500 mt-0.5 shrink-0" />
            <p className="text-xs text-[var(--text-secondary)]">
              These limits protect your device. Citinet will never exceed these
              configured resources.
            </p>
          </div>
        </div>
      </Card>

      {/* Storage Location */}
      <Card>
        <div className="flex items-center gap-2 mb-4">
          <FolderOpen className="w-4 h-4 text-accent-500" />
          <h3 className="text-sm font-medium text-[var(--text-secondary)]">
            Storage Location
          </h3>
        </div>

        {storageStatus && (
          <div className="space-y-3">
            <div className="p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
              <div className="text-xs text-[var(--text-muted)] mb-1">
                Current path
              </div>
              <div className="text-sm font-medium text-[var(--text-primary)] break-all">
                {storageStatus.data_path.replace(/[/\\]storage$/, "")}
              </div>
              {driveSpace && (
                <div className="text-xs text-[var(--text-muted)] mt-2">
                  Drive: {driveSpace.available_gb.toFixed(0)} GB free of{" "}
                  {driveSpace.total_gb.toFixed(0)} GB
                </div>
              )}
            </div>

            <div className="space-y-2">
              <label className="text-xs text-[var(--text-secondary)]">
                Move storage to a new location
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newPath}
                  onChange={(e) => setNewPath(e.target.value)}
                  placeholder="Enter new path, e.g. D:\CitinetData"
                  disabled={relocating}
                  className="flex-1 px-3 py-2 text-sm rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-primary-500 focus:outline-none disabled:opacity-50"
                />
                <Button
                  size="sm"
                  onClick={handleRelocate}
                  disabled={relocating || !newPath.trim()}
                >
                  {relocating ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-1.5 animate-spin" />
                      Migrating...
                    </>
                  ) : (
                    <>
                      <ArrowRight className="w-4 h-4 mr-1.5" />
                      Relocate
                    </>
                  )}
                </Button>
              </div>
            </div>

            {relocateResult && (
              <div
                className={`flex items-start gap-2 p-3 rounded-lg border ${
                  relocateResult.success
                    ? "bg-accent-500/10 border-accent-500/30"
                    : "bg-red-500/10 border-red-500/30"
                }`}
              >
                {relocateResult.success ? (
                  <CheckCircle className="w-4 h-4 text-accent-500 mt-0.5 shrink-0" />
                ) : (
                  <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
                )}
                <p
                  className={`text-xs ${relocateResult.success ? "text-accent-500" : "text-red-500"}`}
                >
                  {relocateResult.message}
                </p>
              </div>
            )}

            <div className="flex items-start gap-2 p-2.5 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
              <AlertCircle className="w-3.5 h-3.5 text-[var(--text-muted)] mt-0.5 shrink-0" />
              <p className="text-[10px] text-[var(--text-muted)]">
                Relocation copies all data (files, database, binaries) to the
                new path. Your old data is preserved as a backup. The tunnel will
                be temporarily stopped during migration.
              </p>
            </div>
          </div>
        )}
      </Card>

      {/* ISP Sharing */}
      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          ISP Sharing
        </h3>
        <div className="space-y-4">
          <div className="flex items-start gap-2 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
            <AlertCircle className="w-4 h-4 text-[var(--text-muted)] mt-0.5 shrink-0" />
            <div>
              <p className="text-sm text-[var(--text-primary)] font-medium">
                Coming Soon
              </p>
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Internet bandwidth sharing will be available in Phase 3 after
                local mesh networking is established.
              </p>
            </div>
          </div>
        </div>
      </Card>

      {/* Node Configuration */}
      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Node Configuration
        </h3>
        <div className="space-y-4">
          <Toggle
            checked={autoStart}
            onChange={async (val) => {
              setAutoStart(val);
              try { await CitinetAPI.setAutoStart(val); } catch (e) { console.error("Failed to set auto-start:", e); }
            }}
            label="Auto-start on boot"
            description="Launch Citinet when your system starts"
          />
          <Toggle
            checked={backgroundMode}
            onChange={async (val) => {
              setBackgroundMode(val);
              try { await CitinetAPI.setBackgroundMode(val); } catch (e) { console.error("Failed to set background mode:", e); }
            }}
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

      {/* System Information */}
      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          System Information
        </h3>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">Version</span>
            <span className="text-[var(--text-primary)] font-medium">
              0.1.0
            </span>
          </div>
          {hwInfo && (
            <>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">Hostname</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.hostname}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">OS</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.os_name} {hwInfo.os_version}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">CPU Cores</span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.cpu_count}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">
                  Total Memory
                </span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.total_memory_gb.toFixed(1)} GB
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--text-secondary)]">
                  Total Storage
                </span>
                <span className="text-[var(--text-primary)] font-medium">
                  {hwInfo.total_disk_gb.toFixed(0)} GB
                </span>
              </div>
              {hwInfo.is_raspberry_pi && (
                <div className="flex items-center gap-2 mt-2 p-2 rounded bg-accent-500/10 border border-accent-500/20">
                  <span className="text-sm text-accent-500 font-medium">
                    Raspberry Pi Detected
                  </span>
                </div>
              )}
            </>
          )}
        </div>
      </Card>

      {/* Diagnostics */}
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
            <span className="text-[var(--text-primary)] font-medium capitalize">
              {nodeType}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">App Version</span>
            <span className="text-[var(--text-primary)] font-medium">
              {APP_VERSION}
            </span>
          </div>
          {hwInfo && (
            <div className="flex justify-between">
              <span className="text-[var(--text-secondary)]">Platform</span>
              <span className="text-[var(--text-primary)] font-medium">
                {hwInfo.os_name} {hwInfo.os_version}
              </span>
            </div>
          )}
        </div>
      </Card>

      {/* Updates */}
      <Card>
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <RefreshCw className="w-4 h-4 text-primary-500" />
            <h3 className="text-sm font-medium text-[var(--text-secondary)]">Updates</h3>
          </div>
          <span className="text-xs text-[var(--text-muted)]">v{APP_VERSION}</span>
        </div>

        <div className="space-y-3">
          {updateStatus === "idle" && (
            <Button size="sm" variant="secondary" onClick={handleCheckForUpdates} className="w-full">
              <RefreshCw className="w-4 h-4 mr-2" />
              Check for Updates
            </Button>
          )}

          {updateStatus === "checking" && (
            <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
              <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
              Checking for updates…
            </div>
          )}

          {updateStatus === "up-to-date" && (
            <div className="flex items-center gap-2 text-sm">
              <CheckCircle className="w-4 h-4 text-accent-500" />
              <span className="text-[var(--text-primary)]">You're up to date</span>
              <button onClick={() => setUpdateStatus("idle")} className="ml-auto text-xs text-[var(--text-muted)] hover:text-[var(--text-secondary)]">
                Check again
              </button>
            </div>
          )}

          {updateStatus === "available" && updateVersion && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Download className="w-4 h-4 text-primary-500" />
                <span className="text-[var(--text-primary)]">
                  Update available: <span className="font-medium text-primary-500">v{updateVersion}</span>
                </span>
              </div>
              <Button size="sm" onClick={handleInstallUpdate} className="w-full">
                <Download className="w-4 h-4 mr-2" />
                Install & Restart
              </Button>
            </div>
          )}

          {updateStatus === "downloading" && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
                <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
                Downloading update… {updateProgress > 0 ? `${updateProgress}%` : ""}
              </div>
              {updateProgress > 0 && (
                <ProgressBar value={updateProgress} showPercent={false} color="primary" />
              )}
            </div>
          )}

          {updateStatus === "error" && (
            <div className="flex items-center gap-2 text-sm">
              <AlertCircle className="w-4 h-4 text-red-500" />
              <span className="text-red-500">Could not check for updates</span>
              <button onClick={() => setUpdateStatus("idle")} className="ml-auto text-xs text-[var(--text-muted)] hover:text-[var(--text-secondary)]">
                Retry
              </button>
            </div>
          )}
        </div>
      </Card>

      {/* Danger Zone */}
      <Card className="border-red-500/30 dark:border-red-500/20">
        <div className="flex items-center gap-2 mb-4">
          <Trash2 className="w-4 h-4 text-red-500" />
          <h3 className="text-sm font-medium text-red-500">Danger Zone</h3>
        </div>
        <div className="space-y-3">
          <div className="p-3 rounded-lg bg-red-500/5 border border-red-500/20">
            <p className="text-sm font-medium text-[var(--text-primary)] mb-1">Factory Reset</p>
            <p className="text-xs text-[var(--text-muted)] mb-3">
              Deletes all users, files, and configuration. The setup wizard will run on next launch.
              Your data folder is not removed from disk.
            </p>
            <Button
              size="sm"
              onClick={handleFactoryReset}
              disabled={resetting}
              className="w-full !bg-red-600 hover:!bg-red-700 !text-white"
            >
              {resetting ? (
                <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Resetting…</>
              ) : (
                <><Trash2 className="w-4 h-4 mr-2" />Factory Reset</>
              )}
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
}
