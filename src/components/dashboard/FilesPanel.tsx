import { Card } from "../ui/Card";
import { FileText, Upload, FolderOpen } from "lucide-react";
import { Button } from "../ui/Button";

const MOCK_FILES = [
  { name: "project-backup.zip", size: "24.5 MB", date: "2 hours ago" },
  { name: "photos-2024.tar.gz", size: "156 MB", date: "1 day ago" },
  { name: "documents.enc", size: "8.2 MB", date: "3 days ago" },
];

export function FilesPanel() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          My Files
        </h2>
        <Button size="sm">
          <Upload className="w-4 h-4 mr-2" />
          Upload
        </Button>
      </div>

      <Card>
        {MOCK_FILES.length === 0 ? (
          <div className="text-center py-8">
            <FolderOpen className="w-12 h-12 text-[var(--text-muted)] mx-auto mb-3" />
            <p className="text-[var(--text-secondary)]">No files yet</p>
            <p className="text-sm text-[var(--text-muted)]">
              Upload files to store them on the network
            </p>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border-color)]">
            {MOCK_FILES.map((file) => (
              <div
                key={file.name}
                className="flex items-center gap-3 py-3 first:pt-0 last:pb-0"
              >
                <FileText className="w-5 h-5 text-primary-400" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-[var(--text-primary)] truncate">
                    {file.name}
                  </p>
                  <p className="text-xs text-[var(--text-muted)]">
                    {file.size}
                  </p>
                </div>
                <span className="text-xs text-[var(--text-muted)]">
                  {file.date}
                </span>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
