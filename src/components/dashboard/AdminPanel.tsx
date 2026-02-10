import { useState, useEffect, useCallback } from "react";
import { Card } from "../ui/Card";
import { CitinetAPI, DockerStatus, DockerContainer, CloudflaredStatus, TunnelStatus } from "../../api/tauri";
import {
  Container, Play, Square, RotateCcw, AlertCircle, CheckCircle2,
  Globe, Link, Loader2,
} from "lucide-react";

// --- Docker Section ---

function DockerSection() {
  const [status, setStatus] = useState<DockerStatus | null>(null);
  const [containers, setContainers] = useState<DockerContainer[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const refresh = useCallback(() => {
    CitinetAPI.checkDocker().then((s) => {
      setStatus(s);
      if (s.running) {
        CitinetAPI.listDockerContainers()
          .then(setContainers)
          .catch(console.error);
      }
      setLoading(false);
    }).catch((e) => {
      console.error(e);
      setLoading(false);
    });
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  const handleAction = async (id: string, action: "start" | "stop" | "restart") => {
    setActionLoading(`${id}-${action}`);
    try {
      if (action === "start") await CitinetAPI.startDockerContainer(id);
      else if (action === "stop") await CitinetAPI.stopDockerContainer(id);
      else await CitinetAPI.restartDockerContainer(id);
      refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setActionLoading(null);
    }
  };

  if (loading) {
    return (
      <Card>
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
          <span className="text-sm text-[var(--text-secondary)]">Checking Docker...</span>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <div className="flex items-center gap-2 mb-3">
          <Container className="w-5 h-5 text-primary-500" />
          <h3 className="text-sm font-medium text-[var(--text-primary)]">Docker</h3>
        </div>

        {!status?.installed ? (
          <div className="flex items-start gap-2 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
            <AlertCircle className="w-4 h-4 text-[var(--text-muted)] mt-0.5 shrink-0" />
            <div>
              <p className="text-sm text-[var(--text-primary)] font-medium">Docker Not Detected</p>
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Install Docker Desktop to manage containers from this panel.
              </p>
            </div>
          </div>
        ) : !status.running ? (
          <div className="flex items-start gap-2 p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/30">
            <AlertCircle className="w-4 h-4 text-yellow-500 mt-0.5 shrink-0" />
            <div>
              <p className="text-sm text-yellow-600 dark:text-yellow-400 font-medium">Docker Not Running</p>
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Start Docker Desktop to manage containers. {status.error}
              </p>
            </div>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-accent-500" />
            <span className="text-sm text-[var(--text-primary)]">
              Docker {status.version} — Running
            </span>
          </div>
        )}
      </Card>

      {status?.running && (
        <Card>
          <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">
            Containers ({containers.length})
          </h3>
          {containers.length === 0 ? (
            <p className="text-xs text-[var(--text-muted)]">No containers found.</p>
          ) : (
            <div className="space-y-2">
              {containers.map((c) => {
                const isRunning = c.state === "running";
                return (
                  <div
                    key={c.id}
                    className="flex items-center justify-between p-3 rounded-lg border border-[var(--border-color)]"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <div className={`w-2 h-2 rounded-full ${isRunning ? "bg-accent-500" : "bg-surface-400"}`} />
                        <span className="text-sm font-medium text-[var(--text-primary)] truncate">
                          {c.names}
                        </span>
                      </div>
                      <div className="flex gap-3 mt-1 text-xs text-[var(--text-muted)]">
                        <span>{c.image}</span>
                        <span>{c.status}</span>
                        {c.ports && <span>{c.ports}</span>}
                      </div>
                    </div>
                    <div className="flex items-center gap-1 ml-2 shrink-0">
                      {!isRunning && (
                        <button
                          onClick={() => handleAction(c.id, "start")}
                          disabled={actionLoading !== null}
                          className="p-1.5 rounded hover:bg-surface-100 dark:hover:bg-surface-800 text-accent-500"
                          title="Start"
                        >
                          <Play className="w-4 h-4" />
                        </button>
                      )}
                      {isRunning && (
                        <button
                          onClick={() => handleAction(c.id, "stop")}
                          disabled={actionLoading !== null}
                          className="p-1.5 rounded hover:bg-surface-100 dark:hover:bg-surface-800 text-red-500"
                          title="Stop"
                        >
                          <Square className="w-4 h-4" />
                        </button>
                      )}
                      <button
                        onClick={() => handleAction(c.id, "restart")}
                        disabled={actionLoading !== null}
                        className="p-1.5 rounded hover:bg-surface-100 dark:hover:bg-surface-800 text-[var(--text-secondary)]"
                        title="Restart"
                      >
                        <RotateCcw className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </Card>
      )}
    </div>
  );
}

// --- Tunnel Section ---

function TunnelSection() {
  const [cfStatus, setCfStatus] = useState<CloudflaredStatus | null>(null);
  const [tunnelStatus, setTunnelStatus] = useState<TunnelStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [setupMode, setSetupMode] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);

  // Setup form state
  const [apiToken, setApiToken] = useState("");
  const [tunnelName, setTunnelName] = useState("");
  const [localPort, setLocalPort] = useState(9090);

  const hostname = tunnelName ? `${tunnelName}.citinet.io` : "";

  const refresh = useCallback(() => {
    Promise.all([
      CitinetAPI.checkCloudflared(),
      CitinetAPI.getTunnelStatus(),
    ]).then(([cf, ts]) => {
      setCfStatus(cf);
      setTunnelStatus(ts);
      setLoading(false);
    }).catch((e) => {
      console.error(e);
      setLoading(false);
    });
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  const handleSetup = async () => {
    if (!apiToken || !tunnelName) return;
    setActionLoading(true);
    try {
      await CitinetAPI.setupTunnel(apiToken, tunnelName, hostname, localPort);
      setSetupMode(false);
      refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleStart = async () => {
    setActionLoading(true);
    try {
      await CitinetAPI.startTunnel();
      refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleStop = async () => {
    setActionLoading(true);
    try {
      await CitinetAPI.stopTunnel();
      refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setActionLoading(false);
    }
  };

  if (loading) {
    return (
      <Card>
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
          <span className="text-sm text-[var(--text-secondary)]">Checking cloudflared...</span>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <div className="flex items-center gap-2 mb-3">
          <Globe className="w-5 h-5 text-primary-500" />
          <h3 className="text-sm font-medium text-[var(--text-primary)]">Cloudflare Tunnel</h3>
        </div>

        {!cfStatus?.installed ? (
          <div className="flex items-start gap-2 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
            <AlertCircle className="w-4 h-4 text-[var(--text-muted)] mt-0.5 shrink-0" />
            <div>
              <p className="text-sm text-[var(--text-primary)] font-medium">cloudflared Not Detected</p>
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Install cloudflared to expose your hub node at a public URL.
              </p>
            </div>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-accent-500" />
            <span className="text-sm text-[var(--text-primary)]">
              cloudflared {cfStatus.version ?? ""} — Installed
            </span>
          </div>
        )}
      </Card>

      {cfStatus?.installed && !tunnelStatus?.configured && !setupMode && (
        <Card>
          <button
            onClick={() => setSetupMode(true)}
            className="w-full py-2 px-4 rounded-lg bg-primary-500 text-white text-sm font-medium hover:bg-primary-600 transition-colors"
          >
            Configure Tunnel
          </button>
        </Card>
      )}

      {setupMode && (
        <Card>
          <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">Setup Tunnel</h3>
          <div className="space-y-4">
            <div>
              <label htmlFor="tunnel-token" className="text-xs text-[var(--text-secondary)] block mb-1">
                Cloudflare API Token
              </label>
              <input
                id="tunnel-token"
                type="password"
                value={apiToken}
                onChange={(e) => setApiToken(e.target.value)}
                placeholder="Enter your CF API token"
                className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              />
            </div>
            <div>
              <label htmlFor="tunnel-name" className="text-xs text-[var(--text-secondary)] block mb-1">
                Tunnel Name
              </label>
              <input
                id="tunnel-name"
                type="text"
                value={tunnelName}
                onChange={(e) => setTunnelName(e.target.value)}
                placeholder="my-hub"
                className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              />
              {hostname && (
                <p className="text-xs text-[var(--text-muted)] mt-1">
                  Hostname: <span className="text-primary-500 font-medium">{hostname}</span>
                </p>
              )}
            </div>
            <div>
              <label htmlFor="tunnel-port" className="text-xs text-[var(--text-secondary)] block mb-1">
                Local Port
              </label>
              <input
                id="tunnel-port"
                type="number"
                value={localPort}
                onChange={(e) => setLocalPort(parseInt(e.target.value) || 9090)}
                className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleSetup}
                disabled={actionLoading || !apiToken || !tunnelName}
                className="flex-1 py-2 px-4 rounded-lg bg-primary-500 text-white text-sm font-medium hover:bg-primary-600 transition-colors disabled:opacity-50"
              >
                {actionLoading ? "Setting up..." : "Create Tunnel"}
              </button>
              <button
                onClick={() => setSetupMode(false)}
                className="py-2 px-4 rounded-lg border border-[var(--border-color)] text-sm text-[var(--text-secondary)] hover:bg-surface-100 dark:hover:bg-surface-800"
              >
                Cancel
              </button>
            </div>
          </div>
        </Card>
      )}

      {tunnelStatus?.configured && tunnelStatus.config && (
        <Card>
          <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">Tunnel Status</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-[var(--text-secondary)]">Status</span>
              <span className={`font-medium ${tunnelStatus.running ? "text-accent-500" : "text-[var(--text-muted)]"}`}>
                {tunnelStatus.running ? "Running" : "Stopped"}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--text-secondary)]">Hostname</span>
              <a
                href={`https://${tunnelStatus.config.hostname}`}
                target="_blank"
                rel="noopener noreferrer"
                className="font-medium text-primary-500 flex items-center gap-1 hover:underline"
              >
                {tunnelStatus.config.hostname}
                <Link className="w-3 h-3" />
              </a>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--text-secondary)]">Local Port</span>
              <span className="text-[var(--text-primary)] font-medium">{tunnelStatus.config.local_port}</span>
            </div>
          </div>
          <div className="flex gap-2 mt-4">
            {!tunnelStatus.running ? (
              <button
                onClick={handleStart}
                disabled={actionLoading}
                className="flex-1 py-2 px-4 rounded-lg bg-accent-500 text-white text-sm font-medium hover:bg-accent-600 transition-colors disabled:opacity-50"
              >
                Start Tunnel
              </button>
            ) : (
              <button
                onClick={handleStop}
                disabled={actionLoading}
                className="flex-1 py-2 px-4 rounded-lg bg-red-500 text-white text-sm font-medium hover:bg-red-600 transition-colors disabled:opacity-50"
              >
                Stop Tunnel
              </button>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}

// --- Main AdminPanel ---

export function AdminPanel() {
  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">Admin Panel</h2>
      <DockerSection />
      <TunnelSection />
    </div>
  );
}
