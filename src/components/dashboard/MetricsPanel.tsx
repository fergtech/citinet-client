import { useEffect, useState } from "react";
import { Card } from "../ui/Card";
import { ProgressBar } from "../ui/ProgressBar";
import { Cpu, HardDrive, Wifi, Clock, TrendingUp } from "lucide-react";
import { CitinetAPI, SystemMetrics } from "../../api/tauri";

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

function formatSpeed(mbps: number): string {
  if (mbps < 0.1) return `${(mbps * 1000).toFixed(0)} Kbps`;
  if (mbps < 1) return `${mbps.toFixed(2)} Mbps`;
  return `${mbps.toFixed(1)} Mbps`;
}

export function MetricsPanel() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Fetch metrics immediately
    fetchMetrics();

    // Update metrics every 2 seconds
    const interval = setInterval(fetchMetrics, 2000);
    return () => clearInterval(interval);
  }, []);

  const fetchMetrics = async () => {
    try {
      const data = await CitinetAPI.getSystemMetrics();
      setMetrics(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch metrics");
      console.error("Failed to fetch system metrics:", err);
    }
  };

  if (error) {
    return (
      <Card className="col-span-full">
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          System Metrics
        </h3>
        <p className="text-sm text-red-500">Error: {error}</p>
      </Card>
    );
  }

  if (!metrics) {
    return (
      <Card className="col-span-full">
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          System Metrics
        </h3>
        <p className="text-sm text-[var(--text-muted)]">Loading metrics...</p>
      </Card>
    );
  }

  const diskPercent = (metrics.disk_used_gb / metrics.disk_total_gb) * 100;
  const memoryPercent = (metrics.memory_used_gb / metrics.memory_total_gb) * 100;

  return (
    <Card className="col-span-full">
      <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
        System Metrics
      </h3>

      <div className="grid grid-cols-2 md:grid-cols-5 gap-6">
        {/* CPU */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Cpu className="w-4 h-4 text-primary-500" />
            <span className="text-xs text-[var(--text-secondary)]">CPU</span>
          </div>
          <ProgressBar
            value={Math.round(metrics.cpu_usage)}
            showPercent
            color="primary"
          />
        </div>

        {/* Memory */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <TrendingUp className="w-4 h-4 text-purple-500" />
            <span className="text-xs text-[var(--text-secondary)]">Memory</span>
          </div>
          <ProgressBar
            value={Math.round(memoryPercent)}
            showPercent
            color="accent"
          />
          <div className="text-xs text-[var(--text-muted)] mt-1">
            {metrics.memory_used_gb.toFixed(1)} / {metrics.memory_total_gb.toFixed(1)} GB
          </div>
        </div>

        {/* Disk */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <HardDrive className="w-4 h-4 text-accent-500" />
            <span className="text-xs text-[var(--text-secondary)]">Disk</span>
          </div>
          <ProgressBar
            value={Math.round(diskPercent)}
            showPercent
            color="accent"
          />
          <div className="text-xs text-[var(--text-muted)] mt-1">
            {metrics.disk_used_gb.toFixed(1)} / {metrics.disk_total_gb.toFixed(1)} GB
          </div>
        </div>

        {/* Network */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Wifi className="w-4 h-4 text-primary-400" />
            <span className="text-xs text-[var(--text-secondary)]">
              Network
            </span>
          </div>
          <div className="text-sm">
            <div className="flex justify-between">
              <span className="text-[var(--text-muted)]">&uarr;</span>
              <span className="text-[var(--text-primary)] font-medium">
                {formatSpeed(metrics.network_up_mbps)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[var(--text-muted)]">&darr;</span>
              <span className="text-[var(--text-primary)] font-medium">
                {formatSpeed(metrics.network_down_mbps)}
              </span>
            </div>
          </div>
        </div>

        {/* Uptime */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-primary-500" />
            <span className="text-xs text-[var(--text-secondary)]">
              Uptime
            </span>
          </div>
          <span className="text-lg font-bold text-[var(--text-primary)]">
            {formatUptime(metrics.uptime_seconds)}
          </span>
        </div>
      </div>
    </Card>
  );
}
