import { useState, useEffect } from "react";
import { Card } from "../ui/Card";
import { ProgressBar } from "../ui/ProgressBar";
import { HardDrive } from "lucide-react";
import { CitinetAPI, StorageStatus } from "../../api/tauri";

export function ContributionCard() {
  const [storage, setStorage] = useState<StorageStatus | null>(null);

  useEffect(() => {
    const fetch = () => {
      CitinetAPI.getStorageStatus().then(setStorage).catch(console.error);
    };
    fetch();
    const interval = setInterval(fetch, 10000);
    return () => clearInterval(interval);
  }, []);

  const used = storage?.used_gb ?? 0;
  const quota = storage?.quota_gb ?? 0;

  return (
    <Card>
      <div className="flex items-start justify-between mb-4">
        <div>
          <p className="text-sm text-[var(--text-secondary)] mb-1">
            Storage Contribution
          </p>
          <span className="text-lg font-bold text-[var(--text-primary)]">
            {used.toFixed(2)} GB / {quota.toFixed(0)} GB
          </span>
        </div>
        <HardDrive className="w-5 h-5 text-primary-500" />
      </div>

      <ProgressBar
        value={used}
        max={quota || 1}
        label="Used"
        color="accent"
      />

      {storage && (
        <div className="mt-3 text-xs text-[var(--text-muted)]">
          {storage.file_count} file{storage.file_count !== 1 ? "s" : ""} stored
        </div>
      )}
    </Card>
  );
}
