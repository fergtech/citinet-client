import { useEffect, useState } from "react";
import { useOnboardingStore } from "../../stores/onboardingStore";
import { Button } from "../ui/Button";
import { CheckCircle2, Loader2, Wifi, Shield, Server, XCircle } from "lucide-react";
import { CitinetAPI } from "../../api/tauri";

interface CheckItem {
  label: string;
  icon: React.ElementType;
  status: "pending" | "checking" | "passed" | "failed";
  error?: string;
}

export function NetworkCheck() {
  const { setNetworkCheckPassed, nextStep, prevStep } = useOnboardingStore();
  const [checks, setChecks] = useState<CheckItem[]>([
    { label: "System hardware detection", icon: Server, status: "pending" },
    { label: "Node discovery service", icon: Wifi, status: "pending" },
    { label: "Network readiness", icon: Shield, status: "pending" },
  ]);
  const [allPassed, setAllPassed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    
    const runChecks = async () => {
      // Check 1: Hardware detection
      setChecks((prev) =>
        prev.map((c, idx) => idx === 0 ? { ...c, status: "checking" } : c)
      );
      
      try {
        const hwInfo = await CitinetAPI.getHardwareInfo();
        if (!cancelled) {
          console.log("Hardware info:", hwInfo);
          setChecks((prev) =>
            prev.map((c, idx) => idx === 0 ? { ...c, status: "passed" } : c)
          );
        }
      } catch (err) {
        if (!cancelled) {
          setChecks((prev) =>
            prev.map((c, idx) => 
              idx === 0 
                ? { ...c, status: "failed", error: err instanceof Error ? err.message : "Failed" } 
                : c
            )
          );
          return;
        }
      }

      await new Promise((r) => setTimeout(r, 500));
      if (cancelled) return;

      // Check 2: Start discovery service
      setChecks((prev) =>
        prev.map((c, idx) => idx === 1 ? { ...c, status: "checking" } : c)
      );
      
      try {
        await CitinetAPI.startNodeDiscovery();
        if (!cancelled) {
          setChecks((prev) =>
            prev.map((c, idx) => idx === 1 ? { ...c, status: "passed" } : c)
          );
        }
      } catch (err) {
        if (!cancelled) {
          setChecks((prev) =>
            prev.map((c, idx) => 
              idx === 1 
                ? { ...c, status: "failed", error: err instanceof Error ? err.message : "Failed" } 
                : c
            )
          );
          return;
        }
      }

      await new Promise((r) => setTimeout(r, 500));
      if (cancelled) return;

      // Check 3: Verify metrics are accessible (network ready)
      setChecks((prev) =>
        prev.map((c, idx) => idx === 2 ? { ...c, status: "checking" } : c)
      );
      
      try {
        const metrics = await CitinetAPI.getSystemMetrics();
        if (!cancelled) {
          console.log("System metrics:", metrics);
          setChecks((prev) =>
            prev.map((c, idx) => idx === 2 ? { ...c, status: "passed" } : c)
          );
        }
      } catch (err) {
        if (!cancelled) {
          setChecks((prev) =>
            prev.map((c, idx) => 
              idx === 2 
                ? { ...c, status: "failed", error: err instanceof Error ? err.message : "Failed" } 
                : c
            )
          );
          return;
        }
      }

      if (!cancelled) {
        setAllPassed(true);
        setNetworkCheckPassed(true);
      }
    };

    runChecks();
    
    return () => {
      cancelled = true;
    };
  }, [setNetworkCheckPassed]);

  const hasFailures = checks.some((c) => c.status === "failed");

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Network Health Check
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Verifying your system is ready to connect...
      </p>

      <div className="space-y-4 mb-8">
        {checks.map((check) => {
          const Icon = check.icon;
          return (
            <div
              key={check.label}
              className="flex items-center gap-3 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]"
            >
              <Icon className="w-5 h-5 text-primary-500" />
              <div className="flex-1">
                <span className="text-sm text-[var(--text-primary)] block">
                  {check.label}
                </span>
                {check.status === "failed" && check.error && (
                  <span className="text-xs text-red-500 block mt-1">
                    {check.error}
                  </span>
                )}
              </div>
              {check.status === "pending" && (
                <div className="w-5 h-5 rounded-full border-2 border-surface-300 dark:border-surface-600" />
              )}
              {check.status === "checking" && (
                <Loader2 className="w-5 h-5 text-primary-500 animate-spin" />
              )}
              {check.status === "passed" && (
                <CheckCircle2 className="w-5 h-5 text-accent-500" />
              )}
              {check.status === "failed" && (
                <XCircle className="w-5 h-5 text-red-500" />
              )}
            </div>
          );
        })}
      </div>

      {allPassed && (
        <p className="text-sm text-accent-500 text-center mb-4 font-medium">
          All checks passed! Your node is ready.
        </p>
      )}

      {hasFailures && (
        <p className="text-sm text-red-500 text-center mb-4 font-medium">
          Some checks failed. Please check your system configuration.
        </p>
      )}

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} disabled={!allPassed} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
