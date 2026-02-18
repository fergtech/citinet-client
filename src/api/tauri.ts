import { invoke } from "@tauri-apps/api/core";

export interface SystemMetrics {
  cpu_usage: number;
  memory_used_gb: number;
  memory_total_gb: number;
  disk_used_gb: number;
  disk_total_gb: number;
  network_up_mbps: number;
  network_down_mbps: number;
  uptime_seconds: number;
  timestamp: number;
}

export interface HardwareInfo {
  hostname: string;
  os_name: string;
  os_version: string;
  cpu_count: number;
  total_memory_gb: number;
  total_disk_gb: number;
  is_raspberry_pi: boolean;
}

export interface DriveSpace {
  total_gb: number;
  available_gb: number;
}

// --- Node/Storage types ---

export interface NodeConfig {
  node_id: string;
  node_type: string;
  node_name: string;
  install_path: string;
  disk_quota_gb: number;
  bandwidth_limit_mbps: number;
  cpu_limit_percent: number;
  auto_start: boolean;
  background_mode: boolean;
  created_at: string;
  updated_at: string;
}

export interface StorageStatus {
  used_gb: number;
  quota_gb: number;
  file_count: number;
  data_path: string;
}

export interface NodeStatus {
  node_id: string;
  node_name: string;
  node_type: string;
  uptime_seconds: number;
  storage: StorageStatus;
  online: boolean;
}

export interface FileInfo {
  file_id: string;
  user_id: string;
  file_name: string;
  size_bytes: number;
  is_public: boolean;
  created_at: string;
}

export interface User {
  user_id: string;
  username: string;
  email: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
}

// --- Tunnel types ---

export interface CloudflaredStatus {
  installed: boolean;
  version: string | null;
  error: string | null;
}

export interface TunnelConfig {
  tunnel_id: string;
  tunnel_name: string;
  hostname: string;
  local_port: number;
  created_at: string;
  mode: string;
  tunnel_token: string;
}

export interface TunnelStatus {
  configured: boolean;
  running: boolean;
  config: TunnelConfig | null;
  error: string | null;
}

export class CitinetAPI {
  static async getSystemMetrics(): Promise<SystemMetrics> {
    return await invoke<SystemMetrics>("get_system_metrics");
  }

  static async getHardwareInfo(): Promise<HardwareInfo> {
    return await invoke<HardwareInfo>("get_hardware_info");
  }

  static async getInstallDriveSpace(): Promise<DriveSpace> {
    return await invoke<DriveSpace>("get_install_drive_space");
  }

  static async greet(name: string): Promise<string> {
    return await invoke<string>("greet", { name });
  }

  static async getRecommendedInstallPath(): Promise<string> {
    return await invoke<string>("get_recommended_install_path");
  }

  static async validateInstallPath(path: string): Promise<boolean> {
    return await invoke<boolean>("validate_install_path", { path });
  }

  // --- Node/Storage commands ---

  static async initializeNode(
    installPath: string,
    nodeType: string,
    nodeName: string,
    diskQuotaGb: number,
    bandwidthLimitMbps: number,
    cpuLimitPercent: number,
    autoStart: boolean,
  ): Promise<NodeConfig> {
    return await invoke<NodeConfig>("initialize_node", {
      installPath, nodeType, nodeName, diskQuotaGb,
      bandwidthLimitMbps, cpuLimitPercent, autoStart,
    });
  }

  static async getNodeConfig(): Promise<NodeConfig | null> {
    return await invoke<NodeConfig | null>("get_node_config");
  }

  static async updateResourceLimits(
    diskQuotaGb: number,
    bandwidthLimitMbps: number,
    cpuLimitPercent: number,
  ): Promise<void> {
    return await invoke("update_resource_limits", {
      diskQuotaGb, bandwidthLimitMbps, cpuLimitPercent,
    });
  }

  static async getStorageStatus(): Promise<StorageStatus> {
    return await invoke<StorageStatus>("get_storage_status");
  }

  static async getNodeStatus(): Promise<NodeStatus | null> {
    return await invoke<NodeStatus | null>("get_node_status");
  }

  static async createAdminUser(
    username: string,
    email: string,
    password: string,
  ): Promise<User> {
    return await invoke<User>("create_admin_user", {
      username, email, password,
    });
  }

  static async loginUser(username: string, password: string): Promise<User> {
    return await invoke<User>("login_user", { username, password });
  }

  static async listUsers(): Promise<User[]> {
    return await invoke<User[]>("list_users");
  }

  static async deleteUser(userId: string): Promise<void> {
    return await invoke("delete_user", { userId });
  }

  static async updateUserRole(userId: string, isAdmin: boolean): Promise<void> {
    return await invoke("update_user_role", { userId, isAdmin });
  }

  // --- File operations ---

  static async listFiles(): Promise<FileInfo[]> {
    return await invoke<FileInfo[]>("list_files");
  }

  static async uploadFile(fileName: string, fileData: Uint8Array, isPublic: boolean = true): Promise<void> {
    return await invoke("upload_file", {
      fileName,
      fileData: Array.from(fileData),
      isPublic,
    });
  }

  static async deleteFile(fileName: string): Promise<void> {
    return await invoke("delete_file", { fileName });
  }

  static async readFile(fileName: string): Promise<Uint8Array> {
    const data = await invoke<number[]>("read_file", { fileName });
    return new Uint8Array(data);
  }

  static async updateFileVisibility(fileName: string, isPublic: boolean): Promise<void> {
    return await invoke("update_file_visibility", { fileName, isPublic });
  }

  static async setAutoStart(enabled: boolean): Promise<void> {
    return await invoke("set_auto_start", { enabled });
  }

  static async setBackgroundMode(enabled: boolean): Promise<void> {
    return await invoke("set_background_mode", { enabled });
  }

  static async relocateStorage(newPath: string): Promise<string> {
    return await invoke<string>("relocate_storage", { newPath });
  }

  // --- Tunnel commands ---

  static async startQuickTunnel(localPort: number): Promise<string> {
    return await invoke<string>("start_quick_tunnel", { localPort });
  }

  static async installCloudflared(): Promise<string> {
    return await invoke<string>("install_cloudflared");
  }

  static async checkCloudflared(): Promise<CloudflaredStatus> {
    return await invoke<CloudflaredStatus>("check_cloudflared");
  }

  static async setupTunnel(
    apiToken: string,
    tunnelName: string,
    hostname: string,
    localPort: number,
  ): Promise<TunnelConfig> {
    return await invoke<TunnelConfig>("setup_tunnel", {
      apiToken, tunnelName, hostname, localPort,
    });
  }

  static async startTunnel(): Promise<void> {
    return await invoke("start_tunnel");
  }

  static async stopTunnel(): Promise<void> {
    return await invoke("stop_tunnel");
  }

  static async getTunnelStatus(): Promise<TunnelStatus> {
    return await invoke<TunnelStatus>("get_tunnel_status");
  }
}
