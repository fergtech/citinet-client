import { useState } from "react";
import { useWizardStore } from "../../stores/wizardStore";
import { Button } from "../ui/Button";
import { Server, Globe } from "lucide-react";

function generateSlug(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 63);
}

export function NodeIdentityStep() {
  const { nodeName, nodeSlug, setNodeName, setNodeSlug, nextStep, prevStep } =
    useWizardStore();
  const [slugManuallyEdited, setSlugManuallyEdited] = useState(false);

  const handleNameChange = (name: string) => {
    setNodeName(name);
    if (!slugManuallyEdited) {
      setNodeSlug(generateSlug(name));
    }
  };

  const handleSlugChange = (slug: string) => {
    const cleaned = slug
      .toLowerCase()
      .replace(/[^a-z0-9-]/g, "")
      .slice(0, 63);
    setNodeSlug(cleaned);
    setSlugManuallyEdited(true);
  };

  const isValid = nodeName.trim().length >= 2 && nodeSlug.length >= 3;

  return (
    <div>
      <h2 className="text-xl font-bold mb-2 text-[var(--text-primary)]">
        Name Your Hub
      </h2>
      <p className="text-sm text-[var(--text-secondary)] mb-6">
        Give your hub node a name and a unique slug for its public address.
      </p>

      <div className="flex items-center gap-2 mb-6 p-2 rounded-lg bg-primary-500/10 border border-primary-500/20">
        <Server className="w-4 h-4 text-primary-500 shrink-0" />
        <span className="text-xs font-medium text-primary-500">Hub Node</span>
      </div>

      <div className="space-y-4 mb-6">
        <div>
          <label
            htmlFor="node-name"
            className="text-xs text-[var(--text-secondary)] block mb-1"
          >
            Node Name
          </label>
          <input
            id="node-name"
            type="text"
            value={nodeName}
            onChange={(e) => handleNameChange(e.target.value)}
            placeholder="e.g. Merryweather Commons"
            maxLength={60}
            className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)] focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label
            htmlFor="node-slug"
            className="text-xs text-[var(--text-secondary)] block mb-1"
          >
            Node Slug
          </label>
          <input
            id="node-slug"
            type="text"
            value={nodeSlug}
            onChange={(e) => handleSlugChange(e.target.value)}
            placeholder="merryweather-commons"
            className="w-full px-3 py-2 text-sm rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] text-[var(--text-primary)] focus:ring-2 focus:ring-primary-500 focus:border-primary-500 font-mono"
          />
          <p className="text-xs text-[var(--text-muted)] mt-1">
            Lowercase letters, numbers, and hyphens only (3-63 chars)
          </p>
        </div>

        {nodeSlug.length >= 3 && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
            <Globe className="w-4 h-4 text-primary-500 shrink-0" />
            <span className="text-sm text-[var(--text-primary)]">
              <span className="font-medium text-primary-500">
                {nodeSlug}.citinet.cloud
              </span>
            </span>
          </div>
        )}
      </div>

      <div className="flex gap-3">
        <Button variant="secondary" onClick={prevStep} className="flex-1">
          Back
        </Button>
        <Button onClick={nextStep} disabled={!isValid} className="flex-1">
          Continue
        </Button>
      </div>
    </div>
  );
}
