import {
  LayoutDashboard,
  FolderOpen,
  Settings,
  HelpCircle,
  Shield,
  LogOut,
} from "lucide-react";
import { useAuthStore } from "../../stores/authStore";

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

interface NavItem {
  id: string;
  label: string;
  icon: React.ElementType;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "files", label: "My Files", icon: FolderOpen },
  { id: "admin", label: "Admin", icon: Shield },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "help", label: "Help", icon: HelpCircle },
];

function NavButton({ item, active, onClick }: { item: NavItem; active: boolean; onClick: () => void }) {
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
  const currentUser = useAuthStore((s) => s.currentUser);
  const logout = useAuthStore((s) => s.logout);

  return (
    <aside className="w-56 h-screen bg-[var(--bg-secondary)] border-r border-[var(--border-color)] flex flex-col transition-colors duration-200">
      {/* Logo */}
      <div className="flex items-center gap-2 px-5 py-5 border-b border-[var(--border-color)]">
        <img src="/logo.png" alt="Citinet" className="w-6 h-6" />
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

      {/* User info + logout */}
      <div className="px-4 py-4 border-t border-[var(--border-color)]">
        {currentUser ? (
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-full bg-primary-500/10 flex items-center justify-center shrink-0">
              <span className="text-sm font-medium text-primary-500">
                {currentUser.username.charAt(0).toUpperCase()}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5">
                <span className="text-sm font-medium text-[var(--text-primary)] truncate">
                  {currentUser.username}
                </span>
                {currentUser.is_admin && (
                  <span className="text-[9px] px-1 py-0.5 rounded bg-accent-500/20 text-accent-500 font-medium shrink-0">
                    Admin
                  </span>
                )}
              </div>
              <div className="flex items-center gap-1">
                <div className="w-1.5 h-1.5 rounded-full bg-accent-500" />
                <span className="text-[10px] text-[var(--text-muted)]">Online</span>
              </div>
            </div>
            <button
              onClick={logout}
              className="p-1.5 rounded-md hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
              title="Sign out"
            >
              <LogOut className="w-4 h-4 text-[var(--text-muted)]" />
            </button>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-accent-500 animate-pulse" />
            <span className="text-xs text-[var(--text-secondary)]">
              Node online
            </span>
          </div>
        )}
      </div>
    </aside>
  );
}
