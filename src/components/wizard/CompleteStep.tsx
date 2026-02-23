import { useState, useEffect } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { useAppStore } from "../../stores/appStore";
import { useAuthStore } from "../../stores/authStore";
import { Button } from "../ui/Button";
import { CitinetAPI, TunnelStatus } from "../../api/tauri";
import { CheckCircle2, Server, HardDrive, Globe, Wifi, Link } from "lucide-react";

export function CompleteStep() {
  const { nodeName, nodeSlug, storageContribution, tunnelSkipped } =
    useWizardStore();
  const setPhase = useAppStore((s) => s.setPhase);
  const setCurrentUser = useAuthStore((s) => s.setCurrentUser);
  const createdUser = useWizardStore((s) => s.createdUser);
  const [tunnelStatus, setTunnelStatus] = useState<TunnelStatus | null>(null);

  useEffect(() => {
    if (!tunnelSkipped) {
      CitinetAPI.getTunnelStatus().then(setTunnelStatus).catch(console.error);
    }
  }, [tunnelSkipped]);

  const tunnelConfigured = !tunnelSkipped && tunnelStatus?.configured;
  const webUrl = tunnelConfigured ? `${nodeSlug}.citinet.cloud` : null;
  const tunnelUrl = tunnelConfigured && tunnelStatus?.config?.hostname
    ? tunnelStatus.config.hostname
    : null;

  return (
    <div className="text-center">
      <CheckCircle2 className="w-16 h-16 text-accent-500 mx-auto mb-4" />

      <h2 className="text-2xl font-bold mb-2 text-[var(--text-primary)]">
        Hub Ready!
      </h2>
      <p className="text-[var(--text-secondary)] mb-8">
        Your Citinet hub node is configured and ready to go.
      </p>

      <div className="grid grid-cols-2 gap-4 mb-8 text-left">
        <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
          <Server className="w-5 h-5 text-primary-500 mt-0.5 shrink-0" />
          <div>
            <p className="text-xs text-[var(--text-muted)]">Node Name</p>
            <p className="text-sm font-medium text-[var(--text-primary)]">
              {nodeName || "Hub Node"}
            </p>
          </div>
        </div>

        <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
          <HardDrive className="w-5 h-5 text-accent-500 mt-0.5 shrink-0" />
          <div>
            <p className="text-xs text-[var(--text-muted)]">Storage</p>
            <p className="text-sm font-medium text-[var(--text-primary)]">
              {storageContribution} GB
            </p>
          </div>
        </div>

        {webUrl ? (
          <>
            {/* Web address — where community members go */}
            <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] col-span-2">
              <Globe className="w-5 h-5 text-primary-500 mt-0.5 shrink-0" />
              <div>
                <p className="text-xs text-[var(--text-muted)]">Web Address</p>
                <p className="text-sm font-medium text-primary-500">
                  {webUrl}
                </p>
                <p className="text-xs text-[var(--text-muted)] mt-0.5">
                  Share this with your community
                </p>
              </div>
            </div>

            {/* Tunnel URL — the underlying API endpoint */}
            {tunnelUrl && (
              <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] col-span-2">
                <Link className="w-5 h-5 text-[var(--text-muted)] mt-0.5 shrink-0" />
                <div>
                  <p className="text-xs text-[var(--text-muted)]">Tunnel Endpoint</p>
                  <p className="text-sm font-mono text-[var(--text-secondary)] break-all">
                    {tunnelUrl}
                  </p>
                </div>
              </div>
            )}
          </>
        ) : (
          <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)] col-span-2">
            <Wifi className="w-5 h-5 text-[var(--text-muted)] mt-0.5 shrink-0" />
            <div>
              <p className="text-xs text-[var(--text-muted)]">Access</p>
              <p className="text-sm font-medium text-[var(--text-primary)]">
                Local only
              </p>
              <p className="text-xs text-[var(--text-muted)]">
                Set up a tunnel later from the Admin panel
              </p>
            </div>
          </div>
        )}
      </div>

      <Button
        onClick={() => {
          if (createdUser) setCurrentUser(createdUser);
          setPhase("dashboard");
        }}
        size="lg"
        className="w-full"
      >
        Open Dashboard
      </Button>
    </div>
  );
}
