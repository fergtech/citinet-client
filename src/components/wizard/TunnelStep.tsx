import { useState } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { CitinetAPI } from "../../api/tauri";
import {
  Globe,
  CheckCircle2,
  Loader2,
  XCircle,
  SkipForward,
  Copy,
  Check,
} from "lucide-react";

type SubStepStatus = "pending" | "running" | "done" | "failed";
type TunnelMode = "choose" | "quick" | "custom" | "tailscale";

interface SubStep {
  label: string;
  status: SubStepStatus;
  error?: string;
}

export function TunnelStep() {
  const {
    nodeSlug,
    cfApiToken,
    setCfApiToken,
    setTunnelSkipped,
    nextStep,
    prevStep,
  } = useWizardStore();

  const [mode, setMode] = useState<TunnelMode>("choose");
  const [setting, setSetting] = useState(false);
  const [done, setDone] = useState(false);
  const [publicUrl, setPublicUrl] = useState("");
  const [subSteps, setSubSteps] = useState<SubStep[]>([]);
  const [tsLoginUrl, setTsLoginUrl] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const updateSubStep = (
    index: number,
    update: Partial<SubStep>,
    prev: SubStep[]
  ): SubStep[] => {
    const next = [...prev];
    next[index] = { ...next[index], ...update };
    return next;
  };

  // Tailscale Funnel: check install, login if needed, enable funnel
  const handleTailscaleConnect = async () => {
    setMode("tailscale");
    setSetting(true);

    let steps: SubStep[] = [
      { label: "Checking Tailscale installation", status: "pending" },
      { label: "Sign in to Tailscale", status: "pending" },
      { label: "Enabling Tailscale Funnel", status: "pending" },
    ];
    setSubSteps(steps);

    // Step 1: Check / install
    steps = updateSubStep(0, { status: "running" }, steps);
    setSubSteps(steps);
    let ts = await CitinetAPI.checkTailscale().catch(() => null);
    if (!ts) {
      steps = updateSubStep(0, { status: "failed", error: "Could not reach Tailscale" }, steps);
      setSubSteps(steps);
      setSetting(false);
      return;
    }

    if (!ts.installed) {
      steps = updateSubStep(0, { label: "Installing Tailscale...", status: "running" }, steps);
      setSubSteps(steps);
      try {
        await CitinetAPI.installTailscale();
        ts = await CitinetAPI.checkTailscale();
      } catch (e) {
        steps = updateSubStep(0, {
          status: "failed",
          error: e instanceof Error ? e.message : String(e),
        }, steps);
        setSubSteps(steps);
        setSetting(false);
        return;
      }
    }
    steps = updateSubStep(0, { label: "Tailscale installed", status: "done" }, steps);
    setSubSteps(steps);

    // Step 2: Login if needed
    if (!ts.logged_in) {
      // Show "check browser" immediately — do NOT await startTailscaleLogin.
      // The backend fires tailscale up/login and returns a URL if the CLI prints one,
      // but the tray app often opens the browser directly without any CLI output.
      // We poll BackendState independently so login detection is never blocked.
      steps = updateSubStep(1, { label: "Check your browser or Tailscale taskbar icon", status: "running" }, steps);
      setSubSteps(steps);

      // Fire-and-forget: kicks off tailscale up + login, captures URL if CLI prints it
      CitinetAPI.startTailscaleLogin()
        .then((url) => { if (url) setTsLoginUrl(url); })
        .catch(() => {});

      let loggedIn = false;
      for (let i = 0; i < 90; i++) {
        await new Promise<void>((resolve) => setTimeout(resolve, 2000));
        loggedIn = await CitinetAPI.pollTailscaleLogin();
        if (loggedIn) break;
      }

      if (!loggedIn) {
        steps = updateSubStep(1, {
          status: "failed",
          error: "Sign-in timed out. Please try again.",
        }, steps);
        setSubSteps(steps);
        setSetting(false);
        return;
      }

      setTsLoginUrl(null);
      ts = await CitinetAPI.checkTailscale();
    }
    steps = updateSubStep(1, { label: "Signed in to Tailscale", status: "done" }, steps);
    setSubSteps(steps);

    // Step 3: Enable funnel
    steps = updateSubStep(2, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      const url = await CitinetAPI.startTailscaleFunnel(9090);
      steps = updateSubStep(2, { status: "done" }, steps);
      setSubSteps(steps);
      setPublicUrl(url.replace("https://", ""));
      setDone(true);
    } catch (e) {
      steps = updateSubStep(2, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
    }
    setSetting(false);
  };

  // Quick Connect: install + quick tunnel
  const handleQuickConnect = async () => {
    setMode("quick");
    setSetting(true);

    let steps: SubStep[] = [
      { label: "Installing cloudflared", status: "pending" },
      { label: "Starting tunnel", status: "pending" },
    ];
    setSubSteps(steps);

    // Install
    steps = updateSubStep(0, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      await CitinetAPI.installCloudflared();
      steps = updateSubStep(0, { status: "done" }, steps);
      setSubSteps(steps);
    } catch (e) {
      steps = updateSubStep(0, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
      setSetting(false);
      return;
    }

    // Quick tunnel
    steps = updateSubStep(1, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      const url = await CitinetAPI.startQuickTunnel(9090);
      steps = updateSubStep(1, { status: "done" }, steps);
      setSubSteps(steps);
      setPublicUrl(url.replace("https://", ""));
      setDone(true);
    } catch (e) {
      steps = updateSubStep(1, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
    }
    setSetting(false);
  };

  // Custom Domain: install + API tunnel + start
  const handleCustomConnect = async () => {
    if (!cfApiToken || !nodeSlug) return;
    setSetting(true);

    const hostname = `${nodeSlug}.citinet.cloud`; // will be user-configurable later

    let steps: SubStep[] = [
      { label: "Installing cloudflared", status: "pending" },
      { label: "Creating tunnel", status: "pending" },
      { label: "Starting tunnel", status: "pending" },
    ];
    setSubSteps(steps);

    // Install
    steps = updateSubStep(0, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      await CitinetAPI.installCloudflared();
      steps = updateSubStep(0, { status: "done" }, steps);
      setSubSteps(steps);
    } catch (e) {
      steps = updateSubStep(0, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
      setSetting(false);
      return;
    }

    // Setup tunnel via API
    steps = updateSubStep(1, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      await CitinetAPI.setupTunnel(cfApiToken, nodeSlug, hostname, 9090);
      steps = updateSubStep(1, { status: "done" }, steps);
      setSubSteps(steps);
    } catch (e) {
      steps = updateSubStep(1, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
      setSetting(false);
      return;
    }

    // Start tunnel
    steps = updateSubStep(2, { status: "running" }, steps);
    setSubSteps(steps);
    try {
      await CitinetAPI.startTunnel();
      steps = updateSubStep(2, { status: "done" }, steps);
      setSubSteps(steps);
      setPublicUrl(hostname);
      setDone(true);
    } catch (e) {
      steps = updateSubStep(2, {
        status: "failed",
        error: e instanceof Error ? e.message : String(e),
      }, steps);
      setSubSteps(steps);
    }
    setSetting(false);
  };

  const handleSkip = () => {
    setTunnelSkipped(true);
    nextStep();
  };

  const hasFailed = subSteps.some((s) => s.status === "failed");

  // Success screen
  if (done) {
    return (
      <div>
        <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
          Tunnel Ready
        </h2>
        <div className="flex flex-col items-center gap-3 my-8">
          <CheckCircle2 className="w-12 h-12 text-accent-500" />
          <p className="text-sm text-[var(--text-secondary)]">
            Your hub is accessible at:
          </p>
          <span className="text-primary-500 font-medium">{publicUrl}</span>
        </div>
        <Button onClick={nextStep} className="w-full">
          Continue
        </Button>
      </div>
    );
  }

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Public Access
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Make your hub discoverable so others can find and join your network.
      </p>

      {/* Mode chooser */}
      {mode === "choose" && !setting && !hasFailed && (
        <>
          <div className="space-y-3 mb-4">
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
                Stable URL that never changes. Works on IPv4 and IPv6. Free Tailscale account required.
              </p>
            </button>

            <button
              onClick={handleQuickConnect}
              className="w-full p-4 rounded-lg border border-[var(--border-color)] hover:border-primary-500 transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-[var(--text-primary)]">Quick Connect</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)]">
                  cloudflared
                </span>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                Instant public URL — no account needed. URL is temporary.
              </p>
            </button>

            <button
              onClick={() => setMode("custom")}
              className="w-full p-4 rounded-lg border border-[var(--border-color)] hover:border-primary-500 transition-colors text-left"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-[var(--text-primary)]">Custom Domain</span>
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)]">
                  cloudflared
                </span>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                Permanent URL with your own domain. Requires Cloudflare account.
              </p>
            </button>
          </div>

          <div className="flex gap-3">
            <Button variant="secondary" onClick={prevStep} className="flex-1">
              Back
            </Button>
            <button
              onClick={handleSkip}
              className="flex-1 flex items-center justify-center gap-2 py-2 text-sm text-[var(--text-muted)] hover:text-[var(--text-secondary)] transition-colors"
            >
              <SkipForward className="w-4 h-4" />
              Skip for now
            </button>
          </div>
        </>
      )}

      {/* Custom domain form */}
      {mode === "custom" && !setting && !hasFailed && (
        <>
          <div className="mb-4">
            <label
              htmlFor="cf-token"
              className="text-xs text-[var(--text-secondary)] block mb-1"
            >
              Cloudflare API Token
            </label>
            <input
              id="cf-token"
              type="password"
              value={cfApiToken}
              onChange={(e) => setCfApiToken(e.target.value)}
              placeholder="Enter your Cloudflare API token"
              className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)]"
            />
            <p className="text-xs text-[var(--text-muted)] mt-1">
              Needs Account, Tunnel, and DNS permissions
            </p>
          </div>

          <div className="flex gap-3">
            <Button variant="secondary" onClick={() => setMode("choose")} className="flex-1">
              Back
            </Button>
            <Button
              onClick={handleCustomConnect}
              disabled={!cfApiToken}
              className="flex-1"
            >
              <Globe className="w-4 h-4 mr-2" />
              Connect
            </Button>
          </div>
        </>
      )}

      {/* Progress steps */}
      {(setting || hasFailed) && (
        <div className="space-y-3 mb-6">
          {subSteps.map((step) => (
            <div
              key={step.label}
              className="flex items-center gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]"
            >
              <div className="flex-1">
                <span className="text-sm text-[var(--text-primary)] block">
                  {step.label}
                </span>
                {step.status === "failed" && step.error && (
                  <span className="text-xs text-red-500 block mt-1">
                    {step.error}
                  </span>
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
                <XCircle className="w-5 h-5 text-red-500" />
              )}
            </div>
          ))}

          {/* Fallback login URL — shown when browser didn't open automatically */}
          {tsLoginUrl && setting && (
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

          {hasFailed && (
            <div className="flex gap-3 mt-4">
              <Button
                variant="secondary"
                onClick={() => { setMode("choose"); setSubSteps([]); setTsLoginUrl(null); }}
                className="flex-1"
              >
                Try Again
              </Button>
              <Button onClick={nextStep} className="flex-1">
                Continue Anyway
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
