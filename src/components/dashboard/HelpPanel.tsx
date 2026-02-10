import { Card } from "../ui/Card";
import { BookOpen, MessageCircle, Github, ExternalLink } from "lucide-react";

const HELP_ITEMS = [
  {
    icon: BookOpen,
    title: "Documentation",
    description: "Learn how Citinet works and how to get the most out of it.",
  },
  {
    icon: MessageCircle,
    title: "Community Forum",
    description: "Ask questions and connect with other Citinet users.",
  },
  {
    icon: Github,
    title: "Source Code",
    description: "Citinet is open source. View the code and contribute.",
  },
];

export function HelpPanel() {
  return (
    <div className="space-y-6">
      <h2 className="text-xl font-bold text-[var(--text-primary)]">Help</h2>

      <div className="space-y-3">
        {HELP_ITEMS.map((item) => {
          const Icon = item.icon;
          return (
            <Card key={item.title} className="cursor-pointer hover:shadow-md transition-shadow duration-200">
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-primary-500/10 flex items-center justify-center shrink-0">
                  <Icon className="w-5 h-5 text-primary-500" />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-1">
                    <h3 className="text-sm font-medium text-[var(--text-primary)]">
                      {item.title}
                    </h3>
                    <ExternalLink className="w-3 h-3 text-[var(--text-muted)]" />
                  </div>
                  <p className="text-xs text-[var(--text-secondary)] mt-1">
                    {item.description}
                  </p>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      <Card>
        <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-2">
          Keyboard Shortcuts
        </h3>
        <div className="space-y-2 text-sm">
          {[
            ["Ctrl + D", "Toggle dark mode"],
            ["Ctrl + S", "Open settings"],
            ["Ctrl + U", "Upload file"],
          ].map(([key, desc]) => (
            <div key={key} className="flex justify-between">
              <span className="text-[var(--text-secondary)]">{desc}</span>
              <kbd className="px-2 py-0.5 text-xs bg-surface-100 dark:bg-surface-700 rounded border border-[var(--border-color)] font-mono text-[var(--text-primary)]">
                {key}
              </kbd>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
