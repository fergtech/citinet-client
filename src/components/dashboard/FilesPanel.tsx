import { useState, useEffect, useRef } from "react";
import { Card } from "../ui/Card";
import { FileText, Upload, FolderOpen, Trash2, Download, AlertCircle, Lock, Globe } from "lucide-react";
import { Button } from "../ui/Button";
import { CitinetAPI, FileInfo } from "../../api/tauri";

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

export function FilesPanel() {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

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
      
      await CitinetAPI.uploadFile(file.name, uint8Array);
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

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          My Files
        </h2>
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

      {error && (
        <div className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30">
          <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
          <p className="text-sm text-red-500">{error}</p>
        </div>
      )}

      <Card>
        {files.length === 0 ? (
          <div className="text-center py-8">
            <FolderOpen className="w-12 h-12 text-[var(--text-muted)] mx-auto mb-3" />
            <p className="text-[var(--text-secondary)]">No files yet</p>
            <p className="text-sm text-[var(--text-muted)]">
              Upload files to store them on your hub
            </p>
          </div>
        ) : (
          <div className="divide-y divide-[var(--border-color)]">
            {files.map((file) => (
              <div
                key={file.file_id}
                className="flex items-center gap-3 py-3 first:pt-0 last:pb-0"
              >
                <FileText className="w-5 h-5 text-primary-400 shrink-0" />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="text-sm font-medium text-[var(--text-primary)] truncate">
                      {file.file_name}
                    </p>
                    {file.is_public ? (
                      <Globe className="w-3 h-3 text-green-500" />
                    ) : (
                      <Lock className="w-3 h-3 text-[var(--text-muted)]" />
                    )}
                  </div>
                  <p className="text-xs text-[var(--text-muted)]">
                    {formatBytes(file.size_bytes)}
                    {file.is_public && " • Public"}
                  </p>
                </div>
                <span className="text-xs text-[var(--text-muted)] hidden sm:block">
                  {formatDate(file.created_at)}
                </span>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleDownload(file.file_name)}
                    className="p-1 hover:bg-[var(--surface-hover)] rounded"
                    title="Download"
                  >
                    <Download className="w-4 h-4 text-[var(--text-secondary)]" />
                  </button>
                  <button
                    onClick={() => handleDelete(file.file_name)}
                    className="p-1 hover:bg-red-500/10 rounded"
                    title="Delete"
                  >
                    <Trash2 className="w-4 h-4 text-red-500" />
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
