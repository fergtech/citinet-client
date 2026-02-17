import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { StatusCard } from "./StatusCard";
import { MetricsPanel } from "./MetricsPanel";
import { SettingsPanel } from "./SettingsPanel";
import { FilesPanel } from "./FilesPanel";
import { HelpPanel } from "./HelpPanel";
import { AdminPanel } from "./AdminPanel";

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
            <div className="mb-4">
              <StatusCard />
            </div>
            <MetricsPanel />
          </div>
        )}
        {activeTab === "files" && (
          <div className="max-w-4xl">
            <FilesPanel />
          </div>
        )}
        {activeTab === "admin" && (
          <div className="max-w-4xl">
            <AdminPanel />
          </div>
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
