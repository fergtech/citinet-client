import { useState, useEffect, useCallback } from "react";
import { Card } from "../ui/Card";
import { CitinetAPI, TailscaleStatus, TunnelStatus, User } from "../../api/tauri";
import { useConfigStore } from "../../stores/configStore";
import {
  Globe, Link, Loader2, CheckCircle2, AlertCircle, Copy, Check,
  Users, Shield, ShieldOff, Trash2, Share2, Mail, BookOpen,
} from "lucide-react";

// --- Tunnel Section ---

type TunnelMode = "choose" | "quick" | "custom" | "tailscale";
type SetupPhase = "idle" | "connecting" | "done" | "error";

interface SetupStep {
  label: string;
  status: "pending" | "running" | "done" | "failed";
  error?: string;
}

function generateSlug(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 63);
}

function TunnelSection() {
  const [tunnelStatus, setTunnelStatus] = useState<TunnelStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [mode, setMode] = useState<TunnelMode>("choose");
  const [setupPhase, setSetupPhase] = useState<SetupPhase>("idle");
  const [publicUrl, setPublicUrl] = useState<string | null>(null);

  // Custom domain form state
  const [apiToken, setApiToken] = useState("");
  const [customHostname, setCustomHostname] = useState("");
  const [steps, setSteps] = useState<SetupStep[]>([]);
  const [copied, setCopied] = useState(false);

  // Tailscale-specific state (used only during setup; after setup mode="tailscale" is stored)
  const [tsStatus, setTsStatus] = useState<TailscaleStatus | null>(null);
  const [tsLoginUrl, setTsLoginUrl] = useState<string | null>(null);

  const nodeName = useConfigStore((s) => s.nodeName);
  const tunnelSlug = generateSlug(nodeName || "citinet-hub");

  const shareUrl = async (url: string) => {
    if (navigator.share) {
      try {
        await navigator.share({ title: "Citinet Hub", text: "Join my Citinet hub:", url });
      } catch { /* user cancelled */ }
    } else {
      // Fallback: copy to clipboard
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const emailUrl = (url: string) => {
    const subject = encodeURIComponent("Join my Citinet Hub");
    const body = encodeURIComponent(`Hey,\n\nI'd like to invite you to join my Citinet hub. You can access it here:\n\n${url}\n\nSee you there!`);
    window.open(`mailto:?subject=${subject}&body=${body}`, "_self");
  };

  const refresh = useCallback(() => {
    CitinetAPI.getTunnelStatus().then((ts) => {
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

  const updateStep = (
    index: number,
    update: Partial<SetupStep>,
    prev: SetupStep[]
  ): SetupStep[] => {
    const next = [...prev];
    next[index] = { ...next[index], ...update };
    return next;
  };

  // Quick Connect: install + quick tunnel in one click
  const handleQuickConnect = async () => {
    setMode("quick");
    setSetupPhase("connecting");

    let currentSteps: SetupStep[] = [
      { label: "Installing cloudflared", status: "pending" },
      { label: "Starting tunnel", status: "pending" },
    ];
    setSteps(currentSteps);

    // Step 1: Install
    currentSteps = updateStep(0, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      await CitinetAPI.installCloudflared();
      currentSteps = updateStep(0, { status: "done" }, currentSteps);
      setSteps(currentSteps);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(0, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
      return;
    }

    // Step 2: Start quick tunnel
    currentSteps = updateStep(1, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      const url = await CitinetAPI.startQuickTunnel(9090);
      currentSteps = updateStep(1, { status: "done" }, currentSteps);
      setSteps(currentSteps);
      setPublicUrl(url);
      setSetupPhase("done");
      refresh();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(1, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
    }
  };

  // Custom Domain: install + API tunnel + DNS
  const handleCustomConnect = async () => {
    if (!apiToken || !customHostname) return;
    setSetupPhase("connecting");

    let currentSteps: SetupStep[] = [
      { label: "Installing cloudflared", status: "pending" },
      { label: "Creating tunnel", status: "pending" },
      { label: "Starting tunnel", status: "pending" },
    ];
    setSteps(currentSteps);

    // Step 1: Install
    currentSteps = updateStep(0, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      await CitinetAPI.installCloudflared();
      currentSteps = updateStep(0, { status: "done" }, currentSteps);
      setSteps(currentSteps);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(0, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
      return;
    }

    // Step 2: Setup tunnel via API
    currentSteps = updateStep(1, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      await CitinetAPI.setupTunnel(apiToken, tunnelSlug, customHostname, 9090);
      currentSteps = updateStep(1, { status: "done" }, currentSteps);
      setSteps(currentSteps);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(1, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
      return;
    }

    // Step 3: Start tunnel
    currentSteps = updateStep(2, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      await CitinetAPI.startTunnel();
      currentSteps = updateStep(2, { status: "done" }, currentSteps);
      setSteps(currentSteps);
      setPublicUrl(`https://${customHostname}`);
      setSetupPhase("done");
      refresh();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(2, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
    }
  };

  // Tailscale Funnel: check install, login if needed, enable funnel
  const handleTailscaleConnect = async () => {
    setMode("tailscale");
    setSetupPhase("connecting");

    let currentSteps: SetupStep[] = [
      { label: "Checking Tailscale installation", status: "pending" },
      { label: "Sign in to Tailscale", status: "pending" },
      { label: "Enabling Tailscale Funnel", status: "pending" },
    ];
    setSteps(currentSteps);

    // Step 1: Check / install Tailscale
    currentSteps = updateStep(0, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    let ts: TailscaleStatus;
    try {
      ts = await CitinetAPI.checkTailscale();
      setTsStatus(ts);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(0, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
      return;
    }

    if (!ts.installed) {
      currentSteps = updateStep(0, { label: "Installing Tailscale...", status: "running" }, currentSteps);
      setSteps(currentSteps);
      try {
        await CitinetAPI.installTailscale();
        ts = await CitinetAPI.checkTailscale();
        setTsStatus(ts);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        currentSteps = updateStep(0, { status: "failed", error: msg }, currentSteps);
        setSteps(currentSteps);
        setSetupPhase("error");
        return;
      }
    }
    currentSteps = updateStep(0, { label: "Tailscale installed", status: "done" }, currentSteps);
    setSteps(currentSteps);

    // Step 2: Login if not already authenticated
    if (!ts.logged_in) {
      // Show "check browser" immediately — do NOT await startTailscaleLogin.
      // The Tailscale tray often opens the browser directly; we poll BackendState
      // independently so detection is never blocked on CLI output.
      currentSteps = updateStep(1, { label: "Check your browser or Tailscale taskbar icon", status: "running" }, currentSteps);
      setSteps(currentSteps);

      // Fire-and-forget: kicks off tailscale up + login, captures URL if CLI prints it
      CitinetAPI.startTailscaleLogin()
        .then((url) => { if (url) setTsLoginUrl(url); })
        .catch(() => {});

      // Poll every 2 s for up to 3 minutes (90 attempts)
      let loggedIn = false;
      for (let i = 0; i < 90; i++) {
        await new Promise<void>((resolve) => setTimeout(resolve, 2000));
        loggedIn = await CitinetAPI.pollTailscaleLogin();
        if (loggedIn) break;
      }

      if (!loggedIn) {
        currentSteps = updateStep(1, { status: "failed", error: "Sign-in timed out. Please try again." }, currentSteps);
        setSteps(currentSteps);
        setSetupPhase("error");
        return;
      }

      setTsLoginUrl(null); // clear fallback URL — sign-in complete
      // Refresh status after successful login to get the machine DNS name
      ts = await CitinetAPI.checkTailscale();
      setTsStatus(ts);
    }
    currentSteps = updateStep(1, { label: "Signed in to Tailscale", status: "done" }, currentSteps);
    setSteps(currentSteps);

    // Step 3: Enable funnel
    currentSteps = updateStep(2, { status: "running" }, currentSteps);
    setSteps(currentSteps);
    try {
      const url = await CitinetAPI.startTailscaleFunnel(9090);
      currentSteps = updateStep(2, { status: "done" }, currentSteps);
      setSteps(currentSteps);
      setPublicUrl(url);
      setSetupPhase("done");
      refresh();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      currentSteps = updateStep(2, { status: "failed", error: msg }, currentSteps);
      setSteps(currentSteps);
      setSetupPhase("error");
    }
  };

  const handleStart = async () => {
    setActionLoading(true);
    try {
      if (tunnelStatus?.config?.mode === "tailscale") {
        await CitinetAPI.startTailscaleFunnel(tunnelStatus.config.local_port);
      } else {
        await CitinetAPI.startTunnel();
      }
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
      if (tunnelStatus?.config?.mode === "tailscale") {
        await CitinetAPI.stopTailscaleFunnel();
      } else {
        await CitinetAPI.stopTunnel();
      }
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
          <span className="text-sm text-[var(--text-secondary)]">Checking tunnel status...</span>
        </div>
      </Card>
    );
  }

  // Already configured — show status + controls
  if (tunnelStatus?.configured && tunnelStatus.config) {
    const isQuick = tunnelStatus.config.mode === "quick";
    const isTailscale = tunnelStatus.config.mode === "tailscale";
    return (
      <Card>
        <div className="flex items-center gap-2 mb-3">
          <Globe className="w-5 h-5 text-primary-500" />
          <h3 className="text-sm font-medium text-[var(--text-primary)]">Public Access</h3>
          {isTailscale && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-accent-500/20 text-accent-500 font-medium">
              Tailscale Funnel
            </span>
          )}
          {isQuick && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)]">
              Quick Tunnel
            </span>
          )}
        </div>

        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-[var(--text-secondary)]">Status</span>
            <span className={`font-medium ${tunnelStatus.running ? "text-accent-500" : "text-[var(--text-muted)]"}`}>
              {tunnelStatus.running ? "Running" : "Stopped"}
            </span>
          </div>
          <div>
            <span className="text-[var(--text-secondary)] text-sm">Public URL</span>
            <div className="flex items-center gap-2 mt-1 p-2 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
              <a
                href={`https://${tunnelStatus.config.hostname}`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex-1 text-sm font-medium text-primary-500 hover:underline break-all"
              >
                {tunnelStatus.config.hostname}
              </a>
              <button
                onClick={() => {
                  navigator.clipboard.writeText(`https://${tunnelStatus.config!.hostname}`);
                  setCopied(true);
                  setTimeout(() => setCopied(false), 2000);
                }}
                className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors shrink-0"
                title="Copy URL"
              >
                {copied ? (
                  <Check className="w-4 h-4 text-accent-500" />
                ) : (
                  <Copy className="w-4 h-4 text-[var(--text-muted)]" />
                )}
              </button>
              <button
                onClick={() => shareUrl(`https://${tunnelStatus.config!.hostname}`)}
                className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors shrink-0"
                title="Share link"
              >
                <Share2 className="w-4 h-4 text-[var(--text-muted)]" />
              </button>
              <button
                onClick={() => emailUrl(`https://${tunnelStatus.config!.hostname}`)}
                className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors shrink-0"
                title="Email link"
              >
                <Mail className="w-4 h-4 text-[var(--text-muted)]" />
              </button>
            </div>
          </div>
        </div>

        {isTailscale && (
          <p className="text-xs text-accent-500 mt-3">
            Stable URL — persists across restarts via Tailscale.
            {!tunnelStatus.running && " Funnel may need to be re-enabled."}
          </p>
        )}
        {isQuick && (
          <p className="text-xs text-[var(--text-muted)] mt-3">
            Quick tunnel URL changes on restart. Set up a custom domain for a permanent URL.
          </p>
        )}

        <div className="flex gap-2 mt-4">
          {!tunnelStatus.running ? (
            <button
              onClick={handleStart}
              disabled={actionLoading}
              className="flex-1 py-2 px-4 rounded-lg bg-accent-500 text-white text-sm font-medium hover:bg-accent-600 transition-colors disabled:opacity-50"
            >
              {isTailscale ? "Re-enable Funnel" : "Start Tunnel"}
            </button>
          ) : (
            <button
              onClick={handleStop}
              disabled={actionLoading}
              className="flex-1 py-2 px-4 rounded-lg bg-red-500 text-white text-sm font-medium hover:bg-red-600 transition-colors disabled:opacity-50"
            >
              {isTailscale ? "Disable Funnel" : "Stop Tunnel"}
            </button>
          )}
        </div>
      </Card>
    );
  }

  // Not configured — show setup options
  return (
    <div className="space-y-4">
      <Card>
        <div className="flex items-center gap-2 mb-3">
          <Globe className="w-5 h-5 text-primary-500" />
          <h3 className="text-sm font-medium text-[var(--text-primary)]">Public Access</h3>
        </div>
        <p className="text-xs text-[var(--text-muted)] mb-4">
          Make your hub discoverable so others can find and join your network.
        </p>

        {/* Mode chooser */}
        {mode === "choose" && setupPhase === "idle" && (
          <div className="space-y-3">
            <button
              onClick={handleTailscaleConnect}
              className="w-full p-4 rounded-lg border border-[var(--border-color)] hover:border-primary-500 transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-[var(--text-primary)]">Tailscale Funnel</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-accent-500/20 text-accent-500 font-medium">
                  Recommended
                </span>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                Stable, persistent URL that never changes. Works on IPv4 and IPv6 via Tailscale relay. Free account required.
              </p>
            </button>

            <button
              onClick={handleQuickConnect}
              className="w-full p-4 rounded-lg border border-[var(--border-color)] hover:border-primary-500 transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-[var(--text-primary)]">Quick Connect</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)] text-[10px]">
                  cloudflared
                </span>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                Instant public URL — no account needed. URL is temporary and changes on restart.
              </p>
            </button>

            <button
              onClick={() => setMode("custom")}
              className="w-full p-4 rounded-lg border border-[var(--border-color)] hover:border-primary-500 transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-[var(--text-primary)]">Custom Domain</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)] text-[10px]">
                  cloudflared
                </span>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                Permanent URL with your own domain. Requires a Cloudflare account.
              </p>
            </button>
          </div>
        )}

        {/* Custom domain form */}
        {mode === "custom" && setupPhase === "idle" && (
          <div className="space-y-4">
            <div>
              <label htmlFor="tunnel-hostname" className="text-xs text-[var(--text-secondary)] block mb-1">
                Hostname
              </label>
              <input
                id="tunnel-hostname"
                type="text"
                value={customHostname}
                onChange={(e) => setCustomHostname(e.target.value)}
                placeholder="hub.yourdomain.com"
                className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              />
            </div>
            <div>
              <label htmlFor="tunnel-token" className="text-xs text-[var(--text-secondary)] block mb-1">
                Cloudflare API Token
              </label>
              <input
                id="tunnel-token"
                type="password"
                value={apiToken}
                onChange={(e) => setApiToken(e.target.value)}
                placeholder="Enter your Cloudflare API token"
                className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
              />
              <p className="text-xs text-[var(--text-muted)] mt-1">
                Needs Account, Tunnel, and DNS permissions
              </p>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setMode("choose")}
                className="py-2 px-4 rounded-lg border border-[var(--border-color)] text-sm text-[var(--text-secondary)] hover:bg-surface-100 dark:hover:bg-surface-800"
              >
                Back
              </button>
              <button
                onClick={handleCustomConnect}
                disabled={!apiToken || !customHostname}
                className="flex-1 py-2.5 px-4 rounded-lg bg-primary-500 text-white text-sm font-medium hover:bg-primary-600 transition-colors disabled:opacity-50"
              >
                Connect
              </button>
            </div>
          </div>
        )}

        {/* Progress steps (both modes) */}
        {setupPhase !== "idle" && (
          <div className="space-y-3">
            {steps.map((step) => (
              <div
                key={step.label}
                className="flex items-center gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]"
              >
                <div className="flex-1">
                  <span className="text-sm text-[var(--text-primary)] block">{step.label}</span>
                  {step.status === "failed" && step.error && (
                    <span className="text-xs text-red-500 block mt-1">{step.error}</span>
                  )}
                </div>
                {step.status === "pending" && (
                  <div className="w-5 h-5 rounded-full border-2 border-surface-300 dark:border-surface-600" />
                )}
                {step.status === "running" && (
                  <Loader2 className="w-5 h-5 text-primary-500 animate-spin" />
                )}
                {step.status === "done" && (
                  <CheckCircle2 className="w-5 h-5 text-accent-500" />
                )}
                {step.status === "failed" && (
                  <AlertCircle className="w-5 h-5 text-red-500" />
                )}
              </div>
            ))}

            {/* Fallback login URL — shown when browser didn't open automatically */}
            {tsLoginUrl && setupPhase === "connecting" && (
              <div className="p-3 rounded-lg bg-primary-500/10 border border-primary-500/30">
                <p className="text-xs text-[var(--text-secondary)] mb-1.5">
                  Browser didn't open? Visit this URL to sign in:
                </p>
                <div className="flex items-center gap-2">
                  <a
                    href={tsLoginUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex-1 text-xs text-primary-500 hover:underline break-all font-medium"
                  >
                    {tsLoginUrl}
                  </a>
                  <button
                    onClick={() => { navigator.clipboard.writeText(tsLoginUrl); setCopied(true); setTimeout(() => setCopied(false), 2000); }}
                    className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors shrink-0"
                    title="Copy URL"
                  >
                    {copied ? <Check className="w-3.5 h-3.5 text-accent-500" /> : <Copy className="w-3.5 h-3.5 text-[var(--text-muted)]" />}
                  </button>
                </div>
              </div>
            )}

            {setupPhase === "done" && publicUrl && (
              <div className="flex flex-col items-center gap-2 py-3">
                <CheckCircle2 className="w-8 h-8 text-accent-500" />
                <p className="text-sm text-[var(--text-primary)] font-medium">Hub is live!</p>
                <a
                  href={publicUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary-500 text-sm font-medium flex items-center gap-1 hover:underline"
                >
                  {publicUrl.replace("https://", "")}
                  <Link className="w-3 h-3" />
                </a>
                <div className="flex items-center gap-2 mt-1">
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(publicUrl!);
                      setCopied(true);
                      setTimeout(() => setCopied(false), 2000);
                    }}
                    className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors"
                    title="Copy URL"
                  >
                    {copied ? (
                      <Check className="w-4 h-4 text-accent-500" />
                    ) : (
                      <Copy className="w-4 h-4 text-[var(--text-muted)]" />
                    )}
                  </button>
                  <button
                    onClick={() => shareUrl(publicUrl!)}
                    className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors"
                    title="Share link"
                  >
                    <Share2 className="w-4 h-4 text-[var(--text-muted)]" />
                  </button>
                  <button
                    onClick={() => emailUrl(publicUrl!)}
                    className="p-1.5 rounded-md hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors"
                    title="Email link"
                  >
                    <Mail className="w-4 h-4 text-[var(--text-muted)]" />
                  </button>
                </div>
              </div>
            )}

            {setupPhase === "error" && (
              <button
                onClick={() => { setSetupPhase("idle"); setMode("choose"); }}
                className="w-full py-2 px-4 rounded-lg border border-[var(--border-color)] text-sm text-[var(--text-secondary)] hover:bg-surface-100 dark:hover:bg-surface-800"
              >
                Try Again
              </button>
            )}
          </div>
        )}
      </Card>
    </div>
  );
}

// --- Users Section ---

function UsersSection() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    CitinetAPI.listUsers()
      .then((u) => { setUsers(u); setLoading(false); })
      .catch((e) => { setError(String(e)); setLoading(false); });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const handleToggleAdmin = async (user: User) => {
    try {
      await CitinetAPI.updateUserRole(user.user_id, !user.is_admin);
      refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleDelete = async (user: User) => {
    if (!confirm(`Remove user "${user.username}"? Their files will be deleted.`)) return;
    try {
      await CitinetAPI.deleteUser(user.user_id);
      refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  if (loading) {
    return (
      <Card>
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
          <span className="text-sm text-[var(--text-secondary)]">Loading users...</span>
        </div>
      </Card>
    );
  }

  return (
    <Card>
      <div className="flex items-center gap-2 mb-4">
        <Users className="w-5 h-5 text-primary-500" />
        <h3 className="text-sm font-medium text-[var(--text-primary)]">Users</h3>
        <span className="text-xs text-[var(--text-muted)] ml-auto">{users.length} total</span>
      </div>

      {error && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 mb-3">
          <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
          <p className="text-sm text-red-500">{error}</p>
        </div>
      )}

      {users.length === 0 ? (
        <p className="text-sm text-[var(--text-muted)] text-center py-4">No users yet</p>
      ) : (
        <div className="divide-y divide-[var(--border-color)]">
          {users.map((user) => (
            <div key={user.user_id} className="flex items-center gap-3 py-3 first:pt-0 last:pb-0">
              <div className="w-8 h-8 rounded-full bg-primary-500/10 flex items-center justify-center shrink-0">
                <span className="text-sm font-medium text-primary-500">
                  {user.username.charAt(0).toUpperCase()}
                </span>
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-[var(--text-primary)] truncate">
                    {user.username}
                  </span>
                  {user.is_admin && (
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-accent-500/20 text-accent-500 font-medium shrink-0">
                      Admin
                    </span>
                  )}
                </div>
                <span className="text-xs text-[var(--text-muted)] truncate block">{user.email}</span>
              </div>
              <div className="flex gap-1 shrink-0">
                <button
                  onClick={() => handleToggleAdmin(user)}
                  className="p-1.5 rounded-md hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
                  title={user.is_admin ? "Remove admin" : "Make admin"}
                >
                  {user.is_admin ? (
                    <ShieldOff className="w-4 h-4 text-[var(--text-muted)]" />
                  ) : (
                    <Shield className="w-4 h-4 text-[var(--text-muted)]" />
                  )}
                </button>
                <button
                  onClick={() => handleDelete(user)}
                  className="p-1.5 rounded-md hover:bg-red-500/10 transition-colors"
                  title="Remove user"
                >
                  <Trash2 className="w-4 h-4 text-red-500" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
}

// --- Registry Section ---

function RegistrySection() {
  const [tunnelStatus, setTunnelStatus] = useState<TunnelStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [registered, setRegistered] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    CitinetAPI.getTunnelStatus()
      .then((ts) => { setTunnelStatus(ts); setLoading(false); })
      .catch(() => setLoading(false));
  }, []);

  const handleRegister = async () => {
    setWorking(true);
    setError(null);
    setSuccess(null);
    try {
      await CitinetAPI.registerHub();
      setRegistered(true);
      setSuccess("Hub listed — neighbors can now find and join your network.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setWorking(false);
    }
  };

  const handleDeregister = async () => {
    setWorking(true);
    setError(null);
    setSuccess(null);
    try {
      await CitinetAPI.deregisterHub();
      setRegistered(false);
      setSuccess("Hub removed from the public directory.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setWorking(false);
    }
  };

  if (loading) {
    return (
      <Card>
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
          <span className="text-sm text-[var(--text-secondary)]">Loading...</span>
        </div>
      </Card>
    );
  }

  const tunnelReady = tunnelStatus?.configured && tunnelStatus.running;

  return (
    <Card>
      <div className="flex items-center gap-2 mb-3">
        <BookOpen className="w-5 h-5 text-primary-500" />
        <h3 className="text-sm font-medium text-[var(--text-primary)]">Public Directory</h3>
        {registered && (
          <span className="ml-auto text-[10px] px-1.5 py-0.5 rounded bg-accent-500/20 text-accent-500 font-medium">
            Listed
          </span>
        )}
      </div>

      <p className="text-xs text-[var(--text-muted)] mb-4">
        List your hub in the citinet.cloud directory so neighbors can find and join your network.
        Your tunnel must be running first.
      </p>

      {!tunnelReady && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-surface-100 dark:bg-surface-800 border border-[var(--border-color)] mb-3">
          <AlertCircle className="w-4 h-4 text-[var(--text-muted)] mt-0.5 shrink-0" />
          <p className="text-xs text-[var(--text-muted)]">
            Start your tunnel first so the directory has a public URL to share.
          </p>
        </div>
      )}

      {error && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30 mb-3">
          <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
          <p className="text-sm text-red-500">{error}</p>
        </div>
      )}

      {success && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-accent-500/10 border border-accent-500/30 mb-3">
          <CheckCircle2 className="w-4 h-4 text-accent-500 mt-0.5 shrink-0" />
          <p className="text-sm text-accent-500">{success}</p>
        </div>
      )}

      <div className="flex gap-2">
        {!registered ? (
          <button
            onClick={handleRegister}
            disabled={working || !tunnelReady}
            className="flex-1 py-2 px-4 rounded-lg bg-primary-500 text-white text-sm font-medium hover:bg-primary-600 transition-colors disabled:opacity-50"
          >
            {working ? (
              <span className="flex items-center justify-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin" /> Listing...
              </span>
            ) : "List in Directory"}
          </button>
        ) : (
          <button
            onClick={handleDeregister}
            disabled={working}
            className="flex-1 py-2 px-4 rounded-lg border border-red-300 dark:border-red-800 text-red-500 text-sm font-medium hover:bg-red-500/10 transition-colors disabled:opacity-50"
          >
            {working ? (
              <span className="flex items-center justify-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin" /> Removing...
              </span>
            ) : "Remove from Directory"}
          </button>
        )}
      </div>
    </Card>
  );
}

// --- Main AdminPanel ---

export function AdminPanel() {
  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">Admin Panel</h2>
      <UsersSection />
      <TunnelSection />
      <RegistrySection />
    </div>
  );
}
