import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { StatusCard } from "./StatusCard";
import { ContributionCard } from "./ContributionCard";
import { ImpactCard } from "./ImpactCard";
import { MetricsPanel } from "./MetricsPanel";
import { SettingsPanel } from "./SettingsPanel";
import { FilesPanel } from "./FilesPanel";
import { CommunityPanel } from "./CommunityPanel";
import { HelpPanel } from "./HelpPanel";
import { AdminPanel } from "./AdminPanel";
import { FeatureGate } from "../../lib/features";
import { Card } from "../ui/Card";

function PlaceholderPanel({ title, description }: { title: string; description: string }) {
  return (
    <div className="max-w-2xl">
      <h2 className="text-xl font-bold text-[var(--text-primary)] mb-4">{title}</h2>
      <Card>
        <p className="text-sm text-[var(--text-secondary)]">{description}</p>
      </Card>
    </div>
  );
}

export function Dashboard() {
  const [activeTab, setActiveTab] = useState("dashboard");

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />

      <main className="flex-1 overflow-y-auto p-8">
        {activeTab === "dashboard" && (
          <div className="max-w-4xl">
            <h1 className="text-2xl font-bold text-[var(--text-primary)] mb-6">
              Dashboard
            </h1>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
              <StatusCard />
              <ContributionCard />
              <ImpactCard />
            </div>
            <MetricsPanel />
          </div>
        )}
        {activeTab === "files" && (
          <div className="max-w-4xl">
            <FilesPanel />
          </div>
        )}
        {activeTab === "community" && (
          <div className="max-w-4xl">
            <CommunityPanel />
          </div>
        )}
        {activeTab === "discover" && (
          <FeatureGate flag="discover_tab">
            <PlaceholderPanel
              title="Discover"
              description="Curated links and resources for the Citinet community. Coming soon."
            />
          </FeatureGate>
        )}
        {activeTab === "admin" && (
          <FeatureGate flag="admin_panel">
            <AdminPanel />
          </FeatureGate>
        )}
        {activeTab === "contribution" && (
          <FeatureGate flag="contribution">
            <PlaceholderPanel
              title="Contribution"
              description="Opt-in resource sharing settings and contribution metrics. Coming soon."
            />
          </FeatureGate>
        )}
        {activeTab === "settings" && (
          <div className="max-w-2xl">
            <SettingsPanel />
          </div>
        )}
        {activeTab === "help" && (
          <div className="max-w-2xl">
            <HelpPanel />
          </div>
        )}
      </main>
    </div>
  );
}
