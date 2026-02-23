mod system_monitor;
mod storage_manager;
mod tunnel_manager;
mod hub_api;
mod auth;

use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_autostart::ManagerExt;
use system_monitor::{SystemMetrics, SystemMonitor, HardwareInfo, DriveSpace};
use storage_manager::{StorageManager, NodeConfig, StorageStatus, NodeStatus, File, User};
use tunnel_manager::TunnelManager;

// Application state — Arc-wrapped so it can be shared with the axum API server
struct AppState {
    monitor: Arc<Mutex<SystemMonitor>>,
    storage_manager: Arc<Mutex<Option<StorageManager>>>,
    tunnel_manager: Arc<Mutex<Option<TunnelManager>>>,
    tunnel_stopped_manually: Arc<Mutex<bool>>,
    started_at: Instant,
}

// Shared flag for close-to-tray behavior
struct BackgroundModeState(Arc<Mutex<bool>>);

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
    auth::init_jwt_secret(sm.db()).map_err(|e| e.to_string())?;
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
fn upload_file(state: State<AppState>, file_name: String, file_data: Vec<u8>, is_public: bool) -> Result<File, String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            let admin = sm.get_first_admin().map_err(|e| e.to_string())?
                .ok_or("No admin user found")?;
            sm.upload_file(&admin.user_id, &file_name, &file_data, is_public).map_err(|e| e.to_string())
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

#[tauri::command]
fn update_file_visibility(state: State<AppState>, file_name: String, is_public: bool) -> Result<(), String> {
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    match sm_lock.as_ref() {
        Some(sm) => {
            let admin = sm.get_first_admin().map_err(|e| e.to_string())?
                .ok_or("No admin user found")?;
            sm.update_file_visibility(&admin.user_id, &file_name, is_public).map_err(|e| e.to_string())
        },
        None => Err("Node not initialized".to_string()),
    }
}

#[tauri::command]
fn relocate_storage(app: tauri::AppHandle, state: State<AppState>, new_path: String) -> Result<String, String> {
    // 1. Stop tunnel if running
    {
        let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
        if let Some(tm) = tm_lock.as_mut() {
            let _ = tm.stop_tunnel();
        }
    }

    // 2. Relocate storage (copy-verify-rename)
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let backup_path = {
        let mut sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
        let sm = sm_lock.as_mut().ok_or("Node not initialized")?;
        sm.relocate(&new_path, &app_data_dir).map_err(|e| e.to_string())?
    };

    // 3. Replace TunnelManager with one pointing to new path
    {
        let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
        let sm = sm_lock.as_ref().ok_or("Node not initialized")?;
        let new_tm = TunnelManager::new(sm.install_path());
        let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
        *tm_lock = Some(new_tm);
    }

    Ok(backup_path)
}

// --- Auto-start & Background mode commands ---

#[tauri::command]
fn set_auto_start(
    app: tauri::AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    // Persist to DB
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized")?;
    sm.update_auto_start(enabled).map_err(|e| e.to_string())?;

    // Sync with OS autostart via plugin
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(|e| e.to_string())?;
    } else {
        autolaunch.disable().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn set_background_mode(
    app: tauri::AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    // Persist to DB
    let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
    let sm = sm_lock.as_ref().ok_or("Node not initialized")?;
    sm.update_background_mode(enabled).map_err(|e| e.to_string())?;

    // Update in-memory flag
    let bg_state = app.state::<BackgroundModeState>();
    let mut bg = bg_state.0.lock().map_err(|e| e.to_string())?;
    *bg = enabled;

    Ok(())
}

// --- Tunnel commands ---

#[tauri::command]
fn start_quick_tunnel(state: State<AppState>, local_port: u16) -> Result<String, String> {
    // Clear manual-stop flag so watchdog can monitor this tunnel
    if let Ok(mut flag) = state.tunnel_stopped_manually.lock() {
        *flag = false;
    }
    // Warn if port doesn't match the API server
    if local_port != hub_api::HUB_API_PORT {
        log::warn!(
            "Quick tunnel port {} differs from Hub API port {}. Tunnel may not work correctly.",
            local_port, hub_api::HUB_API_PORT
        );
    }

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
    // Clear manual-stop flag so watchdog can monitor this tunnel
    if let Ok(mut flag) = state.tunnel_stopped_manually.lock() {
        *flag = false;
    }
    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    match tm_lock.as_mut() {
        Some(tm) => {
            // Validate configured port matches API port
            if let Some(cfg) = tm.get_config() {
                if cfg.local_port != hub_api::HUB_API_PORT {
                    return Err(format!(
                        "Tunnel is configured for port {} but the Hub API runs on port {}. \
                         Please reconfigure the tunnel with the correct port.",
                        cfg.local_port, hub_api::HUB_API_PORT
                    ));
                }
            }
            tm.start_tunnel().map_err(|e| e.to_string())
        }
        None => Err("Tunnel not configured".to_string()),
    }
}

#[tauri::command]
fn stop_tunnel(state: State<AppState>) -> Result<(), String> {
    // Signal watchdog: this is intentional, don't auto-restart
    if let Ok(mut flag) = state.tunnel_stopped_manually.lock() {
        *flag = true;
    }
    let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
    match tm_lock.as_mut() {
        Some(tm) => tm.stop_tunnel().map_err(|e| e.to_string()),
        None => Err("Tunnel not configured".to_string()),
    }
}

// --- Registry commands ---
// REGISTRY_SECRET is embedded at compile time via: REGISTRY_SECRET=xxx cargo tauri build
// During development without the env var, registry commands return a descriptive error.

fn make_slug(name: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = false;
    for c in name.to_lowercase().chars() {
        if c.is_alphanumeric() {
            result.push(c);
            last_was_dash = false;
        } else if !last_was_dash && !result.is_empty() {
            result.push('-');
            last_was_dash = true;
        }
    }
    let result = result.trim_end_matches('-').to_string();
    result.chars().take(63).collect()
}

#[tauri::command]
fn register_hub(state: State<AppState>) -> Result<(), String> {
    let secret = option_env!("REGISTRY_SECRET")
        .ok_or("Registry not configured in this build (REGISTRY_SECRET not set)")?;

    let (node_id, node_name) = {
        let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
        let sm = sm_lock.as_ref().ok_or("Node not initialized")?;
        let config = sm.get_node_config().map_err(|e| e.to_string())?
            .ok_or("Node not configured")?;
        (config.node_id, config.node_name)
    };

    let tunnel_url = {
        let mut tm_lock = state.tunnel_manager.lock().map_err(|e| e.to_string())?;
        let status = tm_lock.as_mut()
            .map(|tm| tm.get_status())
            .ok_or("Tunnel not initialized")?;
        if !status.configured {
            return Err("Tunnel not configured — set up public access first".to_string());
        }
        let config = status.config.ok_or("Tunnel config missing")?;
        if config.hostname.starts_with("http") {
            config.hostname
        } else {
            format!("https://{}", config.hostname)
        }
    };

    let slug = make_slug(&node_name);
    let payload = serde_json::json!({
        "id": node_id,
        "name": node_name,
        "slug": slug,
        "location": "",
        "tunnel_url": tunnel_url,
        "online": true,
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://registry.citinet.cloud/hubs")
        .header("Authorization", format!("Bearer {}", secret))
        .json(&payload)
        .send()
        .map_err(|e| format!("Registry request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        return Err(format!("Registry error {}: {}", status, body));
    }

    Ok(())
}

#[tauri::command]
fn deregister_hub(state: State<AppState>) -> Result<(), String> {
    let secret = option_env!("REGISTRY_SECRET")
        .ok_or("Registry not configured in this build (REGISTRY_SECRET not set)")?;

    let node_id = {
        let sm_lock = state.storage_manager.lock().map_err(|e| e.to_string())?;
        let sm = sm_lock.as_ref().ok_or("Node not initialized")?;
        let config = sm.get_node_config().map_err(|e| e.to_string())?
            .ok_or("Node not configured")?;
        config.node_id
    };

    let url = format!("https://registry.citinet.cloud/hubs/{}", node_id);
    let client = reqwest::blocking::Client::new();
    let res = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", secret))
        .send()
        .map_err(|e| format!("Registry request failed: {}", e))?;

    if res.status().as_u16() != 204 && !res.status().is_success() {
        let status = res.status();
        let body = res.text().unwrap_or_default();
        return Err(format!("Registry error {}: {}", status, body));
    }

    Ok(())
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
    let tunnel_stopped_manually = Arc::new(Mutex::new(false));
    let monitor = Arc::new(Mutex::new(SystemMonitor::new()));
    let background_mode = Arc::new(Mutex::new(true)); // default: minimize to tray

    // Clone Arcs for the API server and tunnel watchdog
    let api_sm = storage_manager.clone();
    let api_tm = tunnel_manager.clone();
    let watchdog_stopped_flag = tunnel_stopped_manually.clone();
    let watchdog_tm = tunnel_manager.clone();
    let autostart_tm = tunnel_manager.clone();
    let autostart_stopped_flag = tunnel_stopped_manually.clone();
    let bg_mode_for_close = background_mode.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            monitor,
            storage_manager,
            tunnel_manager,
            tunnel_stopped_manually,
            started_at,
        })
        .manage(BackgroundModeState(background_mode))
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
            update_file_visibility,
            relocate_storage,
            set_auto_start,
            set_background_mode,
            start_quick_tunnel,
            install_cloudflared,
            check_cloudflared,
            setup_tunnel,
            start_tunnel,
            stop_tunnel,
            get_tunnel_status,
            register_hub,
            deregister_hub
        ])
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let bg = bg_mode_for_close.lock().unwrap_or_else(|e| e.into_inner());
                if *bg {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
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
                                    if let Err(e) = auth::init_jwt_secret(sm.db()) {
                                        log::error!("Failed to initialize JWT secret: {}", e);
                                    }
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

            // --- Tray icon ---
            let show_i = MenuItem::with_id(app, "show", "Show Citinet", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // --- Sync autostart + background_mode from DB ---
            {
                let app_state = app.state::<AppState>();
                let sm_lock = app_state.storage_manager.lock().unwrap();
                if let Some(sm) = sm_lock.as_ref() {
                    if let Ok(Some(config)) = sm.get_node_config() {
                        // Sync OS autostart with saved preference
                        let autolaunch = app.autolaunch();
                        if config.auto_start {
                            let _ = autolaunch.enable();
                        } else {
                            let _ = autolaunch.disable();
                        }

                        // Sync background mode flag
                        let bg_state = app.state::<BackgroundModeState>();
                        let mut bg = bg_state.0.lock().unwrap();
                        *bg = config.background_mode;
                    }
                }
            }

            // Spawn the Hub HTTP API server on port 9090
            let (msg_tx, _) = tokio::sync::broadcast::channel::<hub_api::BroadcastMessage>(256);
            let api_state = hub_api::ApiState {
                storage_manager: api_sm,
                tunnel_manager: api_tm,
                started_at,
                msg_tx,
                auth_limiter: hub_api::RateLimiter::new(10, 1.0),
            };

            tauri::async_runtime::spawn(async move {
                if let Err(e) = hub_api::start_hub_api(api_state, hub_api::HUB_API_PORT).await {
                    log::error!("Hub API server failed: {}", e);
                }
            });

            log::info!("Hub API server starting on port {}", hub_api::HUB_API_PORT);

            // Auto-start tunnel if previously configured
            // Runs in a background thread since quick tunnel startup blocks on URL parsing
            std::thread::spawn(move || {
                // Wait for API server to be ready by probing it
                let api_url = format!("http://127.0.0.1:{}/api/health", hub_api::HUB_API_PORT);
                let mut ready = false;
                for attempt in 1..=20 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    match reqwest::blocking::Client::new()
                        .get(&api_url)
                        .timeout(std::time::Duration::from_secs(2))
                        .send()
                    {
                        Ok(resp) if resp.status().is_success() => {
                            log::info!("Hub API ready after {} attempts", attempt);
                            ready = true;
                            break;
                        }
                        Ok(resp) => {
                            log::debug!("API probe attempt {}: status {}", attempt, resp.status());
                        }
                        Err(e) => {
                            log::debug!("API probe attempt {}: {}", attempt, e);
                        }
                    }
                }
                if !ready {
                    log::error!("Hub API not ready after 10s — skipping tunnel auto-start");
                    return;
                }

                let has_config = {
                    let tm_lock = autostart_tm.lock().ok();
                    tm_lock.as_ref()
                        .and_then(|l| l.as_ref())
                        .and_then(|tm| tm.get_config().cloned())
                };

                if let Some(config) = has_config {
                    log::info!("Auto-starting tunnel (mode: {}, port: {})", config.mode, config.local_port);
                    let mut tm_lock = match autostart_tm.lock() {
                        Ok(l) => l,
                        Err(_) => return,
                    };
                    if let Some(tm) = tm_lock.as_mut() {
                        match tm.start_tunnel() {
                            Ok(_) => {
                                // Clear manual-stop flag so watchdog can monitor
                                if let Ok(mut flag) = autostart_stopped_flag.lock() {
                                    *flag = false;
                                }
                                log::info!("Tunnel auto-started successfully");
                            }
                            Err(e) => log::error!("Tunnel auto-start failed: {}", e),
                        }
                    }
                }
            });

            // Tunnel watchdog — checks every 30s, auto-restarts if process crashes
            // Only activates after seeing the tunnel running at least once.
            // Respects manual stop — won't restart if user intentionally stopped it.
            tauri::async_runtime::spawn(async move {
                let mut was_running = false;
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                    // Skip if user manually stopped the tunnel
                    let manually_stopped = watchdog_stopped_flag.lock()
                        .map(|f| *f).unwrap_or(false);
                    if manually_stopped {
                        was_running = false;
                        continue;
                    }

                    let (is_running, is_configured) = {
                        let mut tm_lock = match watchdog_tm.lock() {
                            Ok(l) => l,
                            Err(_) => continue,
                        };
                        if let Some(tm) = tm_lock.as_mut() {
                            let status = tm.get_status();
                            (status.running, status.configured)
                        } else {
                            (false, false)
                        }
                    };

                    if is_running {
                        was_running = true;
                    } else if was_running && is_configured {
                        log::warn!("Tunnel process crashed — attempting auto-restart...");
                        was_running = false;
                        let mut tm_lock = match watchdog_tm.lock() {
                            Ok(l) => l,
                            Err(_) => continue,
                        };
                        if let Some(tm) = tm_lock.as_mut() {
                            match tm.start_tunnel() {
                                Ok(_) => {
                                    log::info!("Tunnel auto-restarted successfully");
                                    was_running = true;
                                }
                                Err(e) => log::error!("Tunnel auto-restart failed: {}", e),
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
