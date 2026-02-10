interface ProgressBarProps {
  value: number;
  max?: number;
  label?: string;
  showPercent?: boolean;
  color?: "primary" | "accent";
}

export function ProgressBar({
  value,
  max = 100,
  label,
  showPercent = true,
  color = "primary",
}: ProgressBarProps) {
  const pct = Math.min(100, Math.round((value / max) * 100));
  const barColor =
    color === "accent" ? "bg-accent-500" : "bg-primary-500";

  return (
    <div className="w-full">
      {(label || showPercent) && (
        <div className="flex justify-between mb-1">
          {label && (
            <span className="text-sm text-[var(--text-secondary)]">{label}</span>
          )}
          {showPercent && (
            <span className="text-sm font-medium text-[var(--text-primary)]">
              {pct}%
            </span>
          )}
        </div>
      )}
      <div className="w-full h-2.5 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
        <div
          className={`h-full ${barColor} rounded-full transition-all duration-500 ease-out`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
