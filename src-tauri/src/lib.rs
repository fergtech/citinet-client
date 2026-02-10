mod system_monitor;
mod hub_service;
mod storage_manager;
mod docker_manager;
mod tunnel_manager;

use std::sync::Mutex;
use std::time::Instant;
use tauri::{Manager, State};
use system_monitor::{SystemMetrics, SystemMonitor, HardwareInfo};
use hub_service::{HubService, HubNode, HubServiceInfo};
use storage_manager::{StorageManager, NodeConfig, StorageStatus, NodeStatus};
use tunnel_manager::TunnelManager;

// Application state
struct AppState {
    monitor: Mutex<SystemMonitor>,
    hub_service: Mutex<HubService>,
    storage_manager: Mutex<Option<StorageManager>>,
    tunnel_manager: Mutex<Option<TunnelManager>>,
    started_at: Instant,
}

#[tauri::command]
fn get_system_metrics(state: State<AppState>) -> Result<SystemMetrics, String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.get_metrics()
}

#[tauri::command]
fn get_hardware_info(state: State<AppState>) -> Result<HardwareInfo, String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.get_hardware_info()
}

#[tauri::command]
fn greet(name: String) -> String {
    format!("Welcome to Citinet, {}!", name)
}

#[tauri::command]
fn start_hub_broadcasting(state: State<AppState>, node_name: String, services: Vec<String>) -> Result<(), String> {
    let mut hub_service = state.hub_service.lock().map_err(|e| e.to_string())?;
    hub_service.start_broadcasting(node_name, services)
}

#[tauri::command]
fn stop_hub_broadcasting(state: State<AppState>) -> Result<(), String> {
    let mut hub_service = state.hub_service.lock().map_err(|e| e.to_string())?;
    hub_service.stop_broadcasting()
}

#[tauri::command]
fn start_node_discovery(state: State<AppState>) -> Result<(), String> {
    let mut hub_service = state.hub_service.lock().map_err(|e| e.to_string())?;
    hub_service.start_discovery()
}

#[tauri::command]
fn get_discovered_nodes(state: State<AppState>) -> Result<Vec<HubNode>, String> {
    let hub_service = state.hub_service.lock().map_err(|e| e.to_string())?;
    hub_service.get_discovered_nodes()
}

#[tauri::command]
fn get_hub_service_info(state: State<AppState>) -> Result<HubServiceInfo, String> {
    let hub_service = state.hub_service.lock().map_err(|e| e.to_string())?;
    hub_service.get_service_info()
}

// --- Storage/Node commands ---

#[tauri::command]
fn initialize_node(
    app: tauri::AppHandle,
    state: State<AppState>,
    install_path: String,
    node_type: String,
    node_name: String,
    disk_quota_gb: f64,
    bandwidth_limit_mbps: f64,
    cpu_limit_percent: f64,
    auto_start: bool,
) -> Result<NodeConfig, String> {
    let mut sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;

    let sm = StorageManager::initialize(&install_path).map_err(|e| e.to_string())?;
    let config = sm.save_node_config(
        &node_type, &node_name, disk_quota_gb, bandwidth_limit_mbps, cpu_limit_percent, auto_start,
    ).map_err(|e| e.to_string())?;

    // Save marker file so we can auto-open on next launch
    if let Ok(data_dir) = app.path().app_data_dir() {
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::write(data_dir.join("install_path.txt"), &install_path);
    }

    // Initialize TunnelManager for the new install path
    let tm = TunnelManager::new(std::path::Path::new(&install_path));
    if let Ok(mut tm_lock) = state.tunnel_manager.lock() {
        *tm_lock = Some(tm);
    }

    *sm_lock = Some(sm);
    Ok(config)
}

#[tauri::command]
fn get_node_config(state: State<AppState>) -> Result<Option<NodeConfig>, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.get_node_config().map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
fn update_resource_limits(
    state: State<AppState>,
    disk_quota_gb: f64,
    bandwidth_limit_mbps: f64,
    cpu_limit_percent: f64,
) -> Result<(), String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.update_resource_limits(disk_quota_gb, bandwidth_limit_mbps, cpu_limit_percent)
            .map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn get_storage_status(state: State<AppState>) -> Result<StorageStatus, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.get_storage_status().map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn get_node_status(state: State<AppState>) -> Result<Option<NodeStatus>, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            let config = sm.get_node_config().map_err(|e| e.to_string())?;
            match config {
                Some(cfg) => {
                    let storage = sm.get_storage_status().map_err(|e| e.to_string())?;
                    let uptime = state.started_at.elapsed().as_secs();
                    Ok(Some(NodeStatus {
                        node_id: cfg.node_id,
                        node_name: cfg.node_name,
                        node_type: cfg.node_type,
                        uptime_seconds: uptime,
                        storage,
                        online: true,
                    }))
                }
                None => Ok(None),
            }
        }
        None => Ok(None),
    }
}

// --- Docker commands ---

#[tauri::command]
fn check_docker() -> docker_manager::DockerStatus {
    docker_manager::check_docker()
}

#[tauri::command]
fn list_docker_containers() -> Result<Vec<docker_manager::DockerContainer>, String> {
    docker_manager::list_containers()
}

#[tauri::command]
fn start_docker_container(id: String) -> Result<(), String> {
    docker_manager::start_container(&id)
}

#[tauri::command]
fn stop_docker_container(id: String) -> Result<(), String> {
    docker_manager::stop_container(&id)
}

#[tauri::command]
fn restart_docker_container(id: String) -> Result<(), String> {
    docker_manager::restart_container(&id)
}

// --- Tunnel commands ---

#[tauri::command]
fn check_cloudflared() -> tunnel_manager::CloudflaredStatus {
    tunnel_manager::check_cloudflared()
}

#[tauri::command]
fn setup_tunnel(
    state: State<AppState>,
    api_token: String,
    tunnel_name: String,
    hostname: String,
    local_port: u16,
) -> Result<tunnel_manager::TunnelConfig, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized")?;

    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    let tm = tm_lock.get_or_insert_with(|| TunnelManager::new(sm.install_path()));

    tm.setup_tunnel(&api_token, &tunnel_name, &hostname, local_port)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_tunnel(state: State<AppState>) -> Result<(), String> {
    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    match tm_lock.as_mut() {
        Some(tm) => tm.start_tunnel().map_err(|e| e.to_string()),
        None => Err("Tunnel not configured".to_string()),
    }
}

#[tauri::command]
fn stop_tunnel(state: State<AppState>) -> Result<(), String> {
    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    match tm_lock.as_mut() {
        Some(tm) => tm.stop_tunnel().map_err(|e| e.to_string()),
        None => Err("Tunnel not configured".to_string()),
    }
}

#[tauri::command]
fn get_tunnel_status(state: State<AppState>) -> Result<tunnel_manager::TunnelStatus, String> {
    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    match tm_lock.as_mut() {
        Some(tm) => Ok(tm.get_status()),
        None => Ok(tunnel_manager::TunnelStatus {
            configured: false,
            running: false,
            config: None,
            error: None,
        }),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            monitor: Mutex::new(SystemMonitor::new()),
            hub_service: Mutex::new(HubService::new().expect("Failed to create HubService")),
            storage_manager: Mutex::new(None),
            tunnel_manager: Mutex::new(None),
            started_at: Instant::now(),
        })
        .invoke_handler(tauri::generate_handler![
            get_system_metrics,
            get_hardware_info,
            start_hub_broadcasting,
            stop_hub_broadcasting,
            start_node_discovery,
            get_discovered_nodes,
            get_hub_service_info,
            greet,
            initialize_node,
            get_node_config,
            update_resource_limits,
            get_storage_status,
            get_node_status,
            check_docker,
            list_docker_containers,
            start_docker_container,
            stop_docker_container,
            restart_docker_container,
            check_cloudflared,
            setup_tunnel,
            start_tunnel,
            stop_tunnel,
            get_tunnel_status
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Try to auto-open StorageManager if node was previously configured
            let app_data_dir = app.path().app_data_dir().ok();
            if let Some(data_dir) = app_data_dir {
                let marker_path = data_dir.join("install_path.txt");
                if marker_path.exists() {
                    if let Ok(install_path) = std::fs::read_to_string(&marker_path) {
                        let install_path = install_path.trim().to_string();
                        let db_path = std::path::Path::new(&install_path)
                            .join("config")
                            .join("citinet.db");
                        if db_path.exists() {
                            if let Ok(sm) = StorageManager::open(&install_path) {
                                let install = std::path::PathBuf::from(&install_path);
                                let tm = TunnelManager::new(&install);
                                {
                                    let app_state = app.state::<AppState>();
                                    let mut sm_lock = app_state.storage_manager.lock().unwrap();
                                    *sm_lock = Some(sm);
                                }
                                {
                                    let app_state = app.state::<AppState>();
                                    let mut tm_lock = app_state.tunnel_manager.lock().unwrap();
                                    *tm_lock = Some(tm);
                                }
                                log::info!("Auto-opened StorageManager at {}", install_path);
                            }
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
