use serde::{Deserialize, Serialize};
use sysinfo::{
    System, Disks, Networks, CpuRefreshKind, MemoryRefreshKind, RefreshKind,
};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub network_up_mbps: f64,
    pub network_down_mbps: f64,
    pub uptime_seconds: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub cpu_count: usize,
    pub total_memory_gb: f64,
    pub total_disk_gb: f64,
    pub is_raspberry_pi: bool,
}

pub struct SystemMonitor {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    networks: Arc<Mutex<Networks>>,
    last_network_update: Arc<Mutex<Instant>>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        
        Self {
            sys: Arc::new(Mutex::new(sys)),
            disks: Arc::new(Mutex::new(Disks::new_with_refreshed_list())),
            networks: Arc::new(Mutex::new(Networks::new_with_refreshed_list())),
            last_network_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn get_metrics(&self) -> Result<SystemMetrics, String> {
        let mut sys = self.sys.lock().map_err(|e| e.to_string())?;
        let mut disks = self.disks.lock().map_err(|e| e.to_string())?;
        let mut networks = self.networks.lock().map_err(|e| e.to_string())?;
        
        // Refresh system info
        sys.refresh_cpu_all();
        sys.refresh_memory();
        disks.refresh_list();
        
        // Calculate CPU usage (average across all cores)
        let cpu_usage = sys.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / sys.cpus().len() as f32;

        // Calculate memory usage
        let memory_used_gb = sys.used_memory() as f64 / 1_073_741_824.0; // bytes to GB
        let memory_total_gb = sys.total_memory() as f64 / 1_073_741_824.0;

        // Calculate disk usage
        let (disk_used, disk_total) = disks.iter().fold((0u64, 0u64), |(used, total), disk| {
            (used + (disk.total_space() - disk.available_space()), total + disk.total_space())
        });
        let disk_used_gb = disk_used as f64 / 1_073_741_824.0;
        let disk_total_gb = disk_total as f64 / 1_073_741_824.0;

        // Calculate network throughput
        let now = Instant::now();
        let mut last_update = self.last_network_update.lock().map_err(|e| e.to_string())?;
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        
        networks.refresh();
        let (tx_bytes, rx_bytes) = networks.iter().fold((0u64, 0u64), |(tx, rx), (_, net)| {
            (tx + net.transmitted(), rx + net.received())
        });
        
        // Convert to Mbps (accounting for time elapsed)
        let network_up_mbps = if elapsed > 0.0 {
            (tx_bytes as f64 * 8.0) / (elapsed * 1_000_000.0)
        } else {
            0.0
        };
        let network_down_mbps = if elapsed > 0.0 {
            (rx_bytes as f64 * 8.0) / (elapsed * 1_000_000.0)
        } else {
            0.0
        };
        
        *last_update = now;

        // Get uptime
        let uptime_seconds = System::uptime();

        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(SystemMetrics {
            cpu_usage: cpu_usage as f64,
            memory_used_gb,
            memory_total_gb,
            disk_used_gb,
            disk_total_gb,
            network_up_mbps,
            network_down_mbps,
            uptime_seconds,
            timestamp,
        })
    }

    pub fn get_hardware_info(&self) -> Result<HardwareInfo, String> {
        let sys = self.sys.lock().map_err(|e| e.to_string())?;
        let disks = self.disks.lock().map_err(|e| e.to_string())?;
        
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        let cpu_count = sys.cpus().len();
        let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;
        
        let disk_total = disks.iter().map(|d| d.total_space()).sum::<u64>();
        let total_disk_gb = disk_total as f64 / 1_073_741_824.0;
        
        // Detect Raspberry Pi by checking for ARM architecture and specific CPU info
        let is_raspberry_pi = cfg!(target_arch = "aarch64") || cfg!(target_arch = "arm") ||
            sys.cpus().first()
                .and_then(|cpu| Some(cpu.brand().to_lowercase().contains("arm")))
                .unwrap_or(false);

        Ok(HardwareInfo {
            hostname,
            os_name,
            os_version,
            cpu_count,
            total_memory_gb,
            total_disk_gb,
            is_raspberry_pi,
        })
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSpace {
    pub total_gb: f64,
    pub available_gb: f64,
}

/// Get disk space for the drive containing the given path.
/// On Windows, matches by drive letter (e.g., C:\). On Unix, finds longest mount prefix.
pub fn get_drive_space_for_path(path: &std::path::Path) -> Result<DriveSpace, String> {
    let disks = Disks::new_with_refreshed_list();
    let path_str = path.to_string_lossy().to_lowercase();

    let mut best_match: Option<&sysinfo::Disk> = None;
    let mut best_len = 0;

    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().to_lowercase();
        if path_str.starts_with(&mount) && mount.len() > best_len {
            best_len = mount.len();
            best_match = Some(disk);
        }
    }

    match best_match {
        Some(disk) => Ok(DriveSpace {
            total_gb: disk.total_space() as f64 / 1_073_741_824.0,
            available_gb: disk.available_space() as f64 / 1_073_741_824.0,
        }),
        None => Err("Could not find drive for path".to_string()),
    }
}
