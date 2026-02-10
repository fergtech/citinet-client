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

export interface HubNode {
  id: string;
  name: string;
  addresses: string[];
  port: number;
  node_type: string;
  services: string[];
  last_seen: number;
}

export interface HubServiceInfo {
  service_name: string;
  port: number;
  services: string[];
  is_running: boolean;
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

// --- Docker types ---

export interface DockerStatus {
  installed: boolean;
  running: boolean;
  version: string | null;
  error: string | null;
}

export interface DockerContainer {
  id: string;
  names: string;
  image: string;
  status: string;
  state: string;
  ports: string;
  created: string;
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

  static async startHubBroadcasting(
    nodeName: string,
    services: string[]
  ): Promise<void> {
    return await invoke("start_hub_broadcasting", {
      nodeName,
      services,
    });
  }

  static async stopHubBroadcasting(): Promise<void> {
    return await invoke("stop_hub_broadcasting");
  }

  static async startNodeDiscovery(): Promise<void> {
    return await invoke("start_node_discovery");
  }

  static async getDiscoveredNodes(): Promise<HubNode[]> {
    return await invoke<HubNode[]>("get_discovered_nodes");
  }

  static async getHubServiceInfo(): Promise<HubServiceInfo> {
    return await invoke<HubServiceInfo>("get_hub_service_info");
  }

  static async greet(name: string): Promise<string> {
    return await invoke<string>("greet", { name });
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

  // --- Docker commands ---

  static async checkDocker(): Promise<DockerStatus> {
    return await invoke<DockerStatus>("check_docker");
  }

  static async listDockerContainers(): Promise<DockerContainer[]> {
    return await invoke<DockerContainer[]>("list_docker_containers");
  }

  static async startDockerContainer(id: string): Promise<void> {
    return await invoke("start_docker_container", { id });
  }

  static async stopDockerContainer(id: string): Promise<void> {
    return await invoke("stop_docker_container", { id });
  }

  static async restartDockerContainer(id: string): Promise<void> {
    return await invoke("restart_docker_container", { id });
  }

  // --- Tunnel commands ---

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
