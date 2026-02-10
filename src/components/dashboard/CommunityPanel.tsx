import { Card } from "../ui/Card";
import { Users, Globe, TrendingUp } from "lucide-react";

const MOCK_PEERS = [
  { name: "Node-Alpha", location: "Germany", uptime: "99.2%" },
  { name: "CloudRunner", location: "Japan", uptime: "98.7%" },
  { name: "DataKeeper", location: "Brazil", uptime: "97.5%" },
  { name: "NetSharer", location: "Canada", uptime: "99.8%" },
];

export function CommunityPanel() {
  return (
    <div className="space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">
        Community
      </h2>

      <div className="grid grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-2 mb-2">
            <Users className="w-4 h-4 text-primary-500" />
            <span className="text-xs text-[var(--text-secondary)]">
              Active Nodes
            </span>
          </div>
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            12,437
          </span>
        </Card>
        <Card>
          <div className="flex items-center gap-2 mb-2">
            <Globe className="w-4 h-4 text-accent-500" />
            <span className="text-xs text-[var(--text-secondary)]">
              Countries
            </span>
          </div>
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            42
          </span>
        </Card>
        <Card>
          <div className="flex items-center gap-2 mb-2">
            <TrendingUp className="w-4 h-4 text-primary-400" />
            <span className="text-xs text-[var(--text-secondary)]">
              Network Growth
            </span>
          </div>
          <span className="text-2xl font-bold text-[var(--text-primary)]">
            +8.2%
          </span>
        </Card>
      </div>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-4">
          Nearby Peers
        </h3>
        <div className="divide-y divide-[var(--border-color)]">
          {MOCK_PEERS.map((peer) => (
            <div
              key={peer.name}
              className="flex items-center gap-3 py-3 first:pt-0 last:pb-0"
            >
              <div className="w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-800 flex items-center justify-center">
                <span className="text-xs font-bold text-primary-500">
                  {peer.name.charAt(0)}
                </span>
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium text-[var(--text-primary)]">
                  {peer.name}
                </p>
                <p className="text-xs text-[var(--text-muted)]">
                  {peer.location}
                </p>
              </div>
              <span className="text-xs text-accent-500 font-medium">
                {peer.uptime}
              </span>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
