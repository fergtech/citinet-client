import { useState, useEffect } from "react";
import { Card } from "../ui/Card";
import { Heart } from "lucide-react";
import { CitinetAPI, StorageStatus } from "../../api/tauri";

export function ImpactCard() {
  const [nodeCount, setNodeCount] = useState(0);
  const [storage, setStorage] = useState<StorageStatus | null>(null);

  useEffect(() => {
    const fetch = () => {
      CitinetAPI.getDiscoveredNodes()
        .then((nodes) => setNodeCount(nodes.length))
        .catch(console.error);
      CitinetAPI.getStorageStatus()
        .then(setStorage)
        .catch(console.error);
    };
    fetch();
    const interval = setInterval(fetch, 10000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Card>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-[var(--text-secondary)] mb-1">
            Your Impact
          </p>
          <span className="text-lg font-bold text-[var(--text-primary)]">
            {nodeCount} node{nodeCount !== 1 ? "s" : ""} discovered
          </span>
        </div>
        <Heart className="w-5 h-5 text-red-400" />
      </div>
      <p className="mt-2 text-xs text-[var(--text-muted)]">
        Contributing {storage ? storage.used_gb.toFixed(2) : "0"} GB to the network.
      </p>
    </Card>
  );
}
