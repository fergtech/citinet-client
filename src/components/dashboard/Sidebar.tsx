import {
  LayoutDashboard,
  FolderOpen,
  Users,
  Settings,
  HelpCircle,
  Cloud,
  Shield,
  Share2,
  Compass,
} from "lucide-react";
import { useFeature, type FeatureFlag } from "../../lib/features";

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

interface NavItem {
  id: string;
  label: string;
  icon: React.ElementType;
  flag?: FeatureFlag;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "files", label: "My Files", icon: FolderOpen },
  { id: "community", label: "Community", icon: Users },
  { id: "discover", label: "Discover", icon: Compass, flag: "discover_tab" },
  { id: "admin", label: "Admin Panel", icon: Shield, flag: "admin_panel" },
  { id: "contribution", label: "Contribution", icon: Share2, flag: "contribution" },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "help", label: "Help", icon: HelpCircle },
];

function NavButton({ item, active, onClick }: { item: NavItem; active: boolean; onClick: () => void }) {
  const enabled = item.flag ? useFeature(item.flag) : true;
  if (!enabled) return null;

  const Icon = item.icon;
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 mb-1 ${
        active
          ? "bg-primary-500/10 text-primary-500"
          : "text-[var(--text-secondary)] hover:bg-surface-100 dark:hover:bg-surface-800"
      }`}
    >
      <Icon className="w-4.5 h-4.5" />
      {item.label}
    </button>
  );
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <aside className="w-56 h-screen bg-[var(--bg-secondary)] border-r border-[var(--border-color)] flex flex-col transition-colors duration-200">
      {/* Logo */}
      <div className="flex items-center gap-2 px-5 py-5 border-b border-[var(--border-color)]">
        <Cloud className="w-6 h-6 text-primary-500" />
        <span className="text-lg font-bold text-[var(--text-primary)]">
          Citinet
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-4 px-3">
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            active={activeTab === item.id}
            onClick={() => onTabChange(item.id)}
          />
        ))}
      </nav>

      {/* Status indicator */}
      <div className="px-5 py-4 border-t border-[var(--border-color)]">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-accent-500 animate-pulse" />
          <span className="text-xs text-[var(--text-secondary)]">
            Node online
          </span>
        </div>
      </div>
    </aside>
  );
}
