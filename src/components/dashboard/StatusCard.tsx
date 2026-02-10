import { useState, useEffect } from "react";
import { Card } from "../ui/Card";
import { Activity, Clock } from "lucide-react";
import { CitinetAPI, NodeStatus } from "../../api/tauri";

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function StatusCard() {
  const [status, setStatus] = useState<NodeStatus | null>(null);

  useEffect(() => {
    const fetch = () => {
      CitinetAPI.getNodeStatus().then(setStatus).catch(console.error);
    };
    fetch();
    const interval = setInterval(fetch, 5000);
    return () => clearInterval(interval);
  }, []);

  const online = status?.online ?? false;
  const nodeId = status?.node_id ?? "—";
  const truncatedId = nodeId.length > 8 ? nodeId.slice(0, 8) + "..." : nodeId;

  return (
    <Card>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-[var(--text-secondary)] mb-1">
            Node Status
          </p>
          <div className="flex items-center gap-2">
            <div className={`w-3 h-3 rounded-full ${online ? "bg-accent-500" : "bg-surface-400"}`} />
            <span className="text-lg font-bold text-[var(--text-primary)]">
              {online ? "Online" : "Offline"}
            </span>
          </div>
        </div>
        <Activity className={`w-5 h-5 ${online ? "text-accent-500" : "text-surface-400"}`} />
      </div>
      <div className="mt-3 text-xs text-[var(--text-muted)]">
        <span title={nodeId}>ID: {truncatedId}</span>
      </div>
      <div className="mt-1 flex items-center gap-1 text-xs text-[var(--text-muted)]">
        <Clock className="w-3 h-3" />
        <span>Uptime: {status ? formatUptime(status.uptime_seconds) : "—"}</span>
      </div>
    </Card>
  );
}
