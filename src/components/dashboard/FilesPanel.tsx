import { useState, useEffect, useRef } from "react";
import { Card } from "../ui/Card";
import {
  FileText,
  Upload,
  FolderOpen,
  Trash2,
  Download,
  AlertCircle,
  Lock,
  Globe,
  HardDrive,
  Users,
} from "lucide-react";
import { Button } from "../ui/Button";
import { CitinetAPI, FileInfo } from "../../api/tauri";

type DriveTab = "personal" | "shared";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

function formatDate(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = (now.getTime() - date.getTime()) / 1000;

  if (diff < 60) return "Just now";
  if (diff < 3600) return Math.floor(diff / 60) + " min ago";
  if (diff < 86400) return Math.floor(diff / 3600) + " hours ago";
  if (diff < 604800) return Math.floor(diff / 86400) + " days ago";

  return date.toLocaleDateString();
}

function TabButton({
  active,
  onClick,
  icon: Icon,
  label,
  count,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ElementType;
  label: string;
  count: number;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ${
        active
          ? "bg-primary-500/10 text-primary-500"
          : "text-[var(--text-secondary)] hover:bg-surface-100 dark:hover:bg-surface-800"
      }`}
    >
      <Icon className="w-4 h-4" />
      {label}
      <span
        className={`text-xs px-1.5 py-0.5 rounded-full ${
          active
            ? "bg-primary-500/20 text-primary-500"
            : "bg-surface-200 dark:bg-surface-700 text-[var(--text-muted)]"
        }`}
      >
        {count}
      </span>
    </button>
  );
}

export function FilesPanel() {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [activeTab, setActiveTab] = useState<DriveTab>("personal");
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const personalFiles = files.filter((f) => !f.is_public);
  const sharedFiles = files.filter((f) => f.is_public);
  const visibleFiles = activeTab === "personal" ? personalFiles : sharedFiles;

  useEffect(() => {
    loadFiles();
  }, []);

  const loadFiles = async () => {
    try {
      const fileList = await CitinetAPI.listFiles();
      setFiles(fileList);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load files");
      console.error("Failed to load files:", err);
    }
  };

  const handleUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setUploading(true);
    setError(null);

    try {
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      const isPublic = activeTab === "shared";
      await CitinetAPI.uploadFile(file.name, uint8Array, isPublic);
      await loadFiles();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
      console.error("Upload failed:", err);
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const handleDelete = async (fileName: string) => {
    if (!confirm(`Delete ${fileName}?`)) return;

    try {
      await CitinetAPI.deleteFile(fileName);
      await loadFiles();
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Delete failed");
      console.error("Delete failed:", err);
    }
  };

  const handleToggleVisibility = async (
    fileName: string,
    currentlyPublic: boolean
  ) => {
    const action = currentlyPublic
      ? "Move to My Drive (private)?"
      : "Move to Shared Drive (public)?";
    if (!confirm(`${fileName}\n${action}`)) return;

    try {
      await CitinetAPI.updateFileVisibility(fileName, !currentlyPublic);
      await loadFiles();
      setError(null);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to update visibility"
      );
    }
  };

  const handleDownload = async (fileName: string) => {
    try {
      const data = await CitinetAPI.readFile(fileName);
      const blob = new Blob([data as unknown as ArrayBuffer]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Download failed");
      console.error("Download failed:", err);
    }
  };

  const emptyMessage =
    activeTab === "personal"
      ? {
          title: "No personal files",
          subtitle: "Upload files here to keep them private",
        }
      : {
          title: "No shared files",
          subtitle: "Upload files here for the community to access",
        };

  return (
    <div className="space-y-5">
      {/* Header with tabs and upload */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1 p-1 rounded-xl bg-surface-50 dark:bg-surface-900 border border-[var(--border-color)]">
          <TabButton
            active={activeTab === "personal"}
            onClick={() => setActiveTab("personal")}
            icon={HardDrive}
            label="My Drive"
            count={personalFiles.length}
          />
          <TabButton
            active={activeTab === "shared"}
            onClick={() => setActiveTab("shared")}
            icon={Users}
            label="Shared"
            count={sharedFiles.length}
          />
        </div>
        <div>
          <input
            ref={fileInputRef}
            type="file"
            onChange={handleUpload}
            className="hidden"
            aria-label="Choose file to upload"
          />
          <Button
            size="sm"
            onClick={() => fileInputRef.current?.click()}
            disabled={uploading}
          >
            <Upload className="w-4 h-4 mr-2" />
            {uploading ? "Uploading..." : "Upload"}
          </Button>
        </div>
      </div>

      {/* Context hint */}
      <p className="text-xs text-[var(--text-muted)] flex items-center gap-1.5">
        {activeTab === "personal" ? (
          <>
            <Lock className="w-3 h-3" />
            Files uploaded here are only visible to you
          </>
        ) : (
          <>
            <Globe className="w-3 h-3" />
            Files uploaded here are visible to all community members
          </>
        )}
      </p>

      {error && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30">
          <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
          <p className="text-sm text-red-500">{error}</p>
        </div>
      )}

      <Card>
        {visibleFiles.length === 0 ? (
          <div className="text-center py-8">
            <FolderOpen className="w-12 h-12 text-[var(--text-muted)] mx-auto mb-3" />
            <p className="text-[var(--text-secondary)]">{emptyMessage.title}</p>
            <p className="text-sm text-[var(--text-muted)]">
              {emptyMessage.subtitle}
            </p>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border-color)]">
            {visibleFiles.map((file) => (
              <div
                key={file.file_id}
                className="flex items-center gap-3 py-3 first:pt-0 last:pb-0"
              >
                <FileText className="w-5 h-5 text-primary-400 shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-[var(--text-primary)] truncate">
                    {file.file_name}
                  </p>
                  <p className="text-xs text-[var(--text-muted)]">
                    {formatBytes(file.size_bytes)}
                  </p>
                </div>
                <span className="text-xs text-[var(--text-muted)] hidden sm:block">
                  {formatDate(file.created_at)}
                </span>
                <div className="flex gap-1">
                  <button
                    onClick={() =>
                      handleToggleVisibility(file.file_name, file.is_public)
                    }
                    className="p-1.5 hover:bg-[var(--surface-hover)] rounded transition-colors"
                    title={
                      file.is_public
                        ? "Move to My Drive"
                        : "Move to Shared Drive"
                    }
                  >
                    {file.is_public ? (
                      <Lock className="w-3.5 h-3.5 text-[var(--text-muted)]" />
                    ) : (
                      <Globe className="w-3.5 h-3.5 text-[var(--text-muted)]" />
                    )}
                  </button>
                  <button
                    onClick={() => handleDownload(file.file_name)}
                    className="p-1.5 hover:bg-[var(--surface-hover)] rounded transition-colors"
                    title="Download"
                  >
                    <Download className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                  </button>
                  <button
                    onClick={() => handleDelete(file.file_name)}
                    className="p-1.5 hover:bg-red-500/10 rounded transition-colors"
                    title="Delete"
                  >
                    <Trash2 className="w-3.5 h-3.5 text-red-500" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
