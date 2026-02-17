mod system_monitor;
mod storage_manager;
mod tunnel_manager;
mod hub_api;
mod auth;

use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};
use system_monitor::{SystemMetrics, SystemMonitor, HardwareInfo, DriveSpace};
use storage_manager::{StorageManager, NodeConfig, StorageStatus, NodeStatus, File, User};
use tunnel_manager::TunnelManager;

// Application state — Arc-wrapped so it can be shared with the axum API server
struct AppState {
    monitor: Arc<Mutex<SystemMonitor>>,
    storage_manager: Arc<Mutex<Option<StorageManager>>>,
    tunnel_manager: Arc<Mutex<Option<TunnelManager>>>,
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
fn get_recommended_install_path(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn validate_install_path(path: String) -> Result<bool, String> {
    use std::path::Path;
    use std::fs;
    
    let path_obj = Path::new(&path);
    
    // Try to create the directory to test write permissions
    match fs::create_dir_all(&path_obj) {
        Ok(_) => Ok(true),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                Err(format!("Permission denied: Cannot write to '{}'. Please choose a different location or run as administrator.", path))
            } else {
                Err(format!("Cannot create directory: {}", e))
            }
        }
    }
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

#[tauri::command]
fn get_install_drive_space(state: State<AppState>) -> Result<DriveSpace, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => system_monitor::get_drive_space_for_path(sm.install_path()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn create_admin_user(
    state: State<AppState>,
    username: String,
    email: String,
    password: String,
) -> Result<User, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            // Hash password
            let password_hash = auth::hash_password(&password).map_err(|e| e.to_string())?;
            // Create admin user (is_admin=true)
            sm.create_user(&username, &email, &password_hash, true)
                .map_err(|e| e.to_string())
        }
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn login_user(state: State<AppState>, username: String, password: String) -> Result<User, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized".to_string())?;

    let user = sm.get_user_by_username(&username)
        .map_err(|e| e.to_string())?
        .ok_or("Invalid credentials".to_string())?;

    let hash = sm.get_password_hash(&username)
        .map_err(|e| e.to_string())?
        .ok_or("Invalid credentials".to_string())?;

    let valid = auth::verify_password(&password, &hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("Invalid credentials".to_string());
    }

    Ok(user)
}

#[tauri::command]
fn list_users(state: State<AppState>) -> Result<Vec<User>, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.list_users().map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn delete_user(state: State<AppState>, user_id: String) -> Result<(), String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.delete_user(&user_id).map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn update_user_role(state: State<AppState>, user_id: String, is_admin: bool) -> Result<(), String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => sm.update_user_role(&user_id, is_admin).map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

// --- File commands ---

#[tauri::command]
fn upload_file(state: State<AppState>, file_name: String, file_data: Vec<u8>) -> Result<File, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            // Get admin user for desktop operations
            let admin = sm.get_first_admin().map_err(|e| e.to_string())?
                .ok_or("No admin user found")?;
            // Desktop uploads default to public
            sm.upload_file(&admin.user_id, &file_name, &file_data, true).map_err(|e| e.to_string())
        },
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn list_files(state: State<AppState>) -> Result<Vec<File>, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        // Desktop admin can see all files
        Some(sm) => sm.list_all_files().map_err(|e| e.to_string()),
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn delete_file(state: State<AppState>, file_name: String) -> Result<(), String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            // Get admin user for desktop operations
            let admin = sm.get_first_admin().map_err(|e| e.to_string())?
                .ok_or("No admin user found")?;
            sm.delete_file(&admin.user_id, &file_name).map_err(|e| e.to_string())
        },
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn read_file(state: State<AppState>, file_name: String) -> Result<Vec<u8>, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            // Get admin user for desktop operations
            let admin = sm.get_first_admin().map_err(|e| e.to_string())?
                .ok_or("No admin user found")?;
            sm.read_file(&admin.user_id, &file_name).map_err(|e| e.to_string())
        },
        None => Err("Node not initialized".to_string()),
    }
}

// --- Tunnel commands ---

#[tauri::command]
fn start_quick_tunnel(state: State<AppState>, local_port: u16) -> Result<String, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized")?;

    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    let tm = tm_lock.get_or_insert_with(|| TunnelManager::new(sm.install_path()));

    tm.start_quick_tunnel(local_port).map_err(|e| e.to_string())
}

#[tauri::command]
fn install_cloudflared(state: State<AppState>) -> Result<String, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized")?;

    let install_path = sm.install_path();
    tunnel_manager::install_cloudflared(install_path)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_cloudflared(state: State<AppState>) -> tunnel_manager::CloudflaredStatus {
    let install_path = state.storage_manager.lock().ok()
        .and_then(|sm| sm.as_ref().map(|s| s.install_path().to_path_buf()));
    tunnel_manager::check_cloudflared(install_path.as_deref())
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
    let started_at = Instant::now();
    let storage_manager = Arc::new(Mutex::new(None::<StorageManager>));
    let tunnel_manager = Arc::new(Mutex::new(None::<TunnelManager>));
    let monitor = Arc::new(Mutex::new(SystemMonitor::new()));

    // Clone Arcs for the API server
    let api_sm = storage_manager.clone();
    let api_tm = tunnel_manager.clone();

    tauri::Builder::default()
        .manage(AppState {
            monitor,
            storage_manager,
            tunnel_manager,
            started_at,
        })
        .invoke_handler(tauri::generate_handler![
            get_system_metrics,
            get_hardware_info,
            get_install_drive_space,
            greet,
            get_recommended_install_path,
            validate_install_path,
            initialize_node,
            get_node_config,
            update_resource_limits,
            get_storage_status,
            get_node_status,
            create_admin_user,
            login_user,
            list_users,
            delete_user,
            update_user_role,
            upload_file,
            list_files,
            delete_file,
            read_file,
            start_quick_tunnel,
            install_cloudflared,
            check_cloudflared,
            setup_tunnel,
            start_tunnel,
            stop_tunnel,
            get_tunnel_status
        ])
        .setup(move |app| {
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
                        
                        // Skip auto-loading if path is in Program Files (requires admin)
                        let is_program_files = install_path.to_lowercase().contains("program files");
                        if is_program_files {
                            log::warn!("Skipping auto-load from restricted path: {}", install_path);
                            log::info!("Please complete the installation wizard to set a new path");
                        } else {
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
            }

            // Spawn the Hub HTTP API server on port 9090
            let api_state = hub_api::ApiState {
                storage_manager: api_sm,
                tunnel_manager: api_tm,
                started_at,
            };

            tauri::async_runtime::spawn(async move {
                if let Err(e) = hub_api::start_hub_api(api_state, 9090).await {
                    log::error!("Hub API server failed: {}", e);
                }
            });

            log::info!("Hub API server starting on port 9090");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
