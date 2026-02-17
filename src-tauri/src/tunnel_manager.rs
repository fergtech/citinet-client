use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::fs;
use base64::Engine;
use reqwest::blocking::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflaredStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub tunnel_id: String,
    pub tunnel_name: String,
    pub hostname: String,
    pub local_port: u16,
    pub created_at: String,
    #[serde(default)]
    pub mode: String, // "quick" or "named"
    #[serde(default)]
    pub tunnel_token: String, // for API-managed tunnels
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub configured: bool,
    pub running: bool,
    pub config: Option<TunnelConfig>,
    pub error: Option<String>,
}

// --- Cloudflare API Structures ---

#[derive(Debug, Serialize, Deserialize)]
struct CfAccount {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CfAccountsResponse {
    result: Vec<CfAccount>,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CfTunnelResult {
    id: String,
    name: String,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CfTunnelResponse {
    result: CfTunnelResult,
    success: bool,
    errors: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareZonesResponse {
    result: Vec<CloudflareZone>,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareDNSRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareDNSResponse {
    success: bool,
    errors: Vec<serde_json::Value>,
}

/// Get account ID from CF API token
fn get_account_id(client: &Client, api_token: &str) -> Result<String> {
    let resp: CfAccountsResponse = client
        .get("https://api.cloudflare.com/client/v4/accounts")
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .send()?
        .json()?;

    if !resp.success || resp.result.is_empty() {
        anyhow::bail!("No Cloudflare accounts found for this API token");
    }

    Ok(resp.result[0].id.clone())
}

/// Create tunnel via CF API (returns tunnel ID + token)
fn create_tunnel_via_api(
    client: &Client,
    api_token: &str,
    account_id: &str,
    tunnel_name: &str,
) -> Result<(String, String)> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel",
        account_id
    );

    let body = serde_json::json!({
        "name": tunnel_name,
        "tunnel_secret": base64::engine::general_purpose::STANDARD.encode(uuid::Uuid::new_v4().as_bytes()),
    });

    let resp: CfTunnelResponse = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?
        .json()?;

    if !resp.success {
        anyhow::bail!("Failed to create tunnel via API: {:?}", resp.errors);
    }

    let tunnel_id = resp.result.id;
    let token = resp.result.token.unwrap_or_default();

    Ok((tunnel_id, token))
}

/// Configure tunnel ingress via CF API
fn configure_tunnel_ingress(
    client: &Client,
    api_token: &str,
    account_id: &str,
    tunnel_id: &str,
    hostname: &str,
    local_port: u16,
) -> Result<()> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}/configurations",
        account_id, tunnel_id
    );

    let body = serde_json::json!({
        "config": {
            "ingress": [
                {
                    "hostname": hostname,
                    "service": format!("http://localhost:{}", local_port),
                },
                {
                    "service": "http_status:404"
                }
            ]
        }
    });

    let resp: serde_json::Value = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?
        .json()?;

    let success = resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    if !success {
        anyhow::bail!("Failed to configure tunnel ingress: {}", resp);
    }

    Ok(())
}

/// Create DNS record via Cloudflare API
fn create_dns_record(
    api_token: &str,
    zone_name: &str,
    hostname: &str,
    tunnel_id: &str,
) -> Result<()> {
    let client = Client::new();

    log::info!("Fetching zone ID for {}", zone_name);
    let zones_url = format!("https://api.cloudflare.com/client/v4/zones?name={}", zone_name);
    let zones_resp: CloudflareZonesResponse = client
        .get(&zones_url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .send()?
        .json()?;

    if !zones_resp.success || zones_resp.result.is_empty() {
        anyhow::bail!("Zone {} not found or access denied", zone_name);
    }

    let zone_id = &zones_resp.result[0].id;
    log::info!("Found zone ID: {}", zone_id);

    let dns_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
    let tunnel_cname = format!("{}.cfargotunnel.com", tunnel_id);

    let dns_record = CloudflareDNSRecord {
        record_type: "CNAME".to_string(),
        name: hostname.to_string(),
        content: tunnel_cname,
        ttl: 1,
        proxied: true,
    };

    log::info!("Creating DNS record: {} -> {}", hostname, dns_record.content);
    let dns_resp: CloudflareDNSResponse = client
        .post(&dns_url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&dns_record)
        .send()?
        .json()?;

    if !dns_resp.success {
        anyhow::bail!("Failed to create DNS record: {:?}", dns_resp.errors);
    }

    log::info!("DNS record created successfully");
    Ok(())
}

/// Download and install cloudflared for Windows
pub fn install_cloudflared(install_dir: &Path) -> Result<PathBuf> {
    log::info!("Downloading cloudflared...");

    let cloudflared_url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe";
    let bin_dir = install_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let cloudflared_path = bin_dir.join("cloudflared.exe");

    if cloudflared_path.exists() {
        log::info!("cloudflared already exists at {:?}", cloudflared_path);
        return Ok(cloudflared_path);
    }

    let client = Client::new();
    let response = client
        .get(cloudflared_url)
        .send()
        .context("Failed to download cloudflared")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download cloudflared: HTTP {}", response.status());
    }

    let bytes = response.bytes()?;
    fs::write(&cloudflared_path, &bytes)
        .context("Failed to write cloudflared executable")?;

    log::info!("cloudflared installed to {:?}", cloudflared_path);
    Ok(cloudflared_path)
}

pub fn check_cloudflared(install_path: Option<&Path>) -> CloudflaredStatus {
    // First check local bin directory (where we install it)
    if let Some(path) = install_path {
        let local_bin = path.join("bin").join("cloudflared.exe");
        if local_bin.exists() {
            let output = Command::new(&local_bin)
                .args(["version"])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    return CloudflaredStatus {
                        installed: true,
                        version: Some(version),
                        error: None,
                    };
                }
            }
            return CloudflaredStatus {
                installed: true,
                version: None,
                error: None,
            };
        }
    }

    // Fallback: check system PATH
    let output = Command::new("cloudflared")
        .args(["version"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                CloudflaredStatus {
                    installed: true,
                    version: Some(version),
                    error: None,
                }
            } else {
                CloudflaredStatus {
                    installed: true,
                    version: None,
                    error: Some("cloudflared returned an error".to_string()),
                }
            }
        }
        Err(_) => CloudflaredStatus {
            installed: false,
            version: None,
            error: Some("cloudflared not found".to_string()),
        },
    }
}

pub struct TunnelManager {
    config_dir: PathBuf,
    bin_dir: PathBuf,
    child: Option<Child>,
    config: Option<TunnelConfig>,
    db_path: PathBuf,
}

impl TunnelManager {
    pub fn new(install_path: &Path) -> Self {
        let config_dir = install_path.join("config");
        let bin_dir = install_path.join("bin");
        let db_path = config_dir.join("citinet.db");

        let _ = fs::create_dir_all(&config_dir);
        let _ = fs::create_dir_all(&bin_dir);

        let config = Self::load_config_from_db(&db_path);

        Self {
            config_dir,
            bin_dir,
            child: None,
            config,
            db_path,
        }
    }

    fn cloudflared_path(&self) -> PathBuf {
        self.bin_dir.join("cloudflared.exe")
    }

    fn get_cloudflared_command(&self) -> String {
        let local_path = self.cloudflared_path();
        if local_path.exists() {
            local_path.to_string_lossy().to_string()
        } else {
            "cloudflared".to_string()
        }
    }

    fn load_config_from_db(db_path: &Path) -> Option<TunnelConfig> {
        let db = rusqlite::Connection::open(db_path).ok()?;
        let mut stmt = db.prepare(
            "SELECT tunnel_id, tunnel_name, hostname, local_port, created_at,
                    COALESCE(mode, 'named') as mode, COALESCE(tunnel_token, '') as tunnel_token
             FROM tunnel_config LIMIT 1"
        ).ok()?;

        stmt.query_row([], |row| {
            Ok(TunnelConfig {
                tunnel_id: row.get(0)?,
                tunnel_name: row.get(1)?,
                hostname: row.get(2)?,
                local_port: row.get::<_, u32>(3)? as u16,
                created_at: row.get(4)?,
                mode: row.get(5)?,
                tunnel_token: row.get(6)?,
            })
        }).ok()
    }

    /// Start a quick tunnel (random trycloudflare.com URL, no auth needed)
    pub fn start_quick_tunnel(&mut self, local_port: u16) -> Result<String> {
        if self.child.is_some() {
            anyhow::bail!("Tunnel is already running");
        }

        let cloudflared_cmd = self.get_cloudflared_command();
        let url = format!("http://localhost:{}", local_port);
        log::info!("Starting quick tunnel for {}", url);

        let mut child = Command::new(&cloudflared_cmd)
            .args(["tunnel", "--url", &url])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .context("Failed to spawn cloudflared quick tunnel")?;

        // Parse the public URL from stderr output
        // cloudflared prints something like: "your url is: https://xxx-yyy-zzz.trycloudflare.com"
        let stderr = child.stderr.take().context("No stderr from cloudflared")?;
        let mut reader = BufReader::new(stderr);

        let mut public_url = String::new();
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);

        let mut line_buf = String::new();
        loop {
            if start.elapsed() > timeout {
                let _ = child.kill();
                anyhow::bail!("Timed out waiting for quick tunnel URL");
            }

            line_buf.clear();
            match reader.read_line(&mut line_buf) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line_buf.trim();
                    log::info!("cloudflared: {}", line);

                    if let Some(url_start) = line.find("https://") {
                        let url_part = &line[url_start..];
                        if url_part.contains("trycloudflare.com") {
                            public_url = url_part
                                .split_whitespace()
                                .next()
                                .unwrap_or(url_part)
                                .trim_end_matches('|')
                                .to_string();
                            break;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Error reading cloudflared stderr: {}", e);
                    break;
                }
            }
        }

        if public_url.is_empty() {
            let _ = child.kill();
            anyhow::bail!("Could not determine quick tunnel URL");
        }

        log::info!("Quick tunnel URL: {}", public_url);

        // Keep draining stderr in a background thread so cloudflared
        // doesn't die from a broken pipe on Windows
        std::thread::spawn(move || {
            let mut buf = String::new();
            loop {
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => log::debug!("cloudflared: {}", buf.trim()),
                }
            }
        });

        // Extract hostname from URL
        let hostname = public_url
            .trim_start_matches("https://")
            .trim_end_matches('/')
            .to_string();

        let now = Utc::now().to_rfc3339();
        let config = TunnelConfig {
            tunnel_id: "quick".to_string(),
            tunnel_name: "quick-tunnel".to_string(),
            hostname: hostname.clone(),
            local_port,
            created_at: now.clone(),
            mode: "quick".to_string(),
            tunnel_token: String::new(),
        };

        // Persist to SQLite
        let db = rusqlite::Connection::open(&self.db_path)
            .context("Failed to open database")?;

        // Ensure columns exist (migration for older DBs)
        let _ = db.execute("ALTER TABLE tunnel_config ADD COLUMN mode TEXT DEFAULT 'named'", []);
        let _ = db.execute("ALTER TABLE tunnel_config ADD COLUMN tunnel_token TEXT DEFAULT ''", []);

        // Try to clear old config, but don't fail if it's locked or permission denied
        match db.execute("DELETE FROM tunnel_config", []) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not clear old tunnel config (may be locked): {}", e);
                // Try INSERT OR REPLACE instead
            }
        }
        
        db.execute(
            "INSERT OR REPLACE INTO tunnel_config
                (tunnel_id, tunnel_name, hostname, local_port, api_token, credentials_path, config_path, created_at, updated_at, mode, tunnel_token)
             VALUES (?1, ?2, ?3, ?4, '', '', '', ?5, ?5, 'quick', '')",
            rusqlite::params![
                "quick",
                "quick-tunnel",
                hostname,
                local_port as u32,
                now,
            ],
        ).context("Failed to save quick tunnel config")?;

        self.config = Some(config);
        self.child = Some(child);

        Ok(public_url)
    }

    /// Setup a named tunnel via CF API (no cert.pem needed)
    pub fn setup_tunnel(
        &mut self,
        api_token: &str,
        tunnel_name: &str,
        hostname: &str,
        local_port: u16,
    ) -> Result<TunnelConfig> {
        log::info!("Setting up API-managed tunnel: {} -> {}", tunnel_name, hostname);

        let client = Client::new();

        // 1. Get account ID
        log::info!("Fetching Cloudflare account ID...");
        let account_id = get_account_id(&client, api_token)
            .context("Failed to get Cloudflare account ID")?;
        log::info!("Account ID: {}", account_id);

        // 2. Create tunnel via API
        log::info!("Creating tunnel '{}' via API...", tunnel_name);
        let (tunnel_id, tunnel_token) = create_tunnel_via_api(&client, api_token, &account_id, tunnel_name)
            .context("Failed to create tunnel")?;
        log::info!("Tunnel created: {} (token: {}...)", tunnel_id, &tunnel_token[..tunnel_token.len().min(8)]);

        // 3. Configure ingress
        log::info!("Configuring tunnel ingress...");
        configure_tunnel_ingress(&client, api_token, &account_id, &tunnel_id, hostname, local_port)
            .context("Failed to configure tunnel ingress")?;

        // 4. Create DNS CNAME record
        // Extract zone name from hostname (e.g., "hub.citinet.io" -> "citinet.io")
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 2 {
            let zone_name = format!("{}.{}", parts[1], parts[0]);
            log::info!("Creating DNS record for zone {}", zone_name);
            if let Err(e) = create_dns_record(api_token, &zone_name, hostname, &tunnel_id) {
                log::warn!("DNS record creation failed (may already exist): {}", e);
            }
        }

        let now = Utc::now().to_rfc3339();
        let config = TunnelConfig {
            tunnel_id: tunnel_id.clone(),
            tunnel_name: tunnel_name.to_string(),
            hostname: hostname.to_string(),
            local_port,
            created_at: now.clone(),
            mode: "named".to_string(),
            tunnel_token: tunnel_token.clone(),
        };

        // Persist to SQLite
        let db = rusqlite::Connection::open(&self.db_path)
            .context("Failed to open database")?;

        let _ = db.execute("ALTER TABLE tunnel_config ADD COLUMN mode TEXT DEFAULT 'named'", []);
        let _ = db.execute("ALTER TABLE tunnel_config ADD COLUMN tunnel_token TEXT DEFAULT ''", []);

        // Try to clear old config, but don't fail if it's locked or permission denied
        match db.execute("DELETE FROM tunnel_config", []) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not clear old tunnel config (may be locked): {}", e);
                // Try INSERT OR REPLACE instead
            }
        }
        
        db.execute(
            "INSERT OR REPLACE INTO tunnel_config
                (tunnel_id, tunnel_name, hostname, local_port, api_token, credentials_path, config_path, created_at, updated_at, mode, tunnel_token)
             VALUES (?1, ?2, ?3, ?4, ?5, '', '', ?6, ?6, 'named', ?7)",
            rusqlite::params![
                tunnel_id,
                tunnel_name,
                hostname,
                local_port as u32,
                api_token,
                now,
                tunnel_token,
            ],
        ).context("Failed to save tunnel config")?;

        log::info!("Named tunnel setup complete!");
        self.config = Some(config.clone());
        Ok(config)
    }

    pub fn start_tunnel(&mut self) -> Result<()> {
        if self.child.is_some() {
            anyhow::bail!("Tunnel is already running");
        }

        let config = self.config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Tunnel not configured. Run setup first."))?;

        let cloudflared_cmd = self.get_cloudflared_command();

        if config.mode == "quick" {
            // Restart quick tunnel — need to capture the new URL
            let url = format!("http://localhost:{}", config.local_port);
            log::info!("Restarting quick tunnel for {}", url);

            let mut child = Command::new(&cloudflared_cmd)
                .args(["tunnel", "--url", &url])
                .stderr(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .context("Failed to spawn cloudflared quick tunnel")?;

            let stderr = child.stderr.take().context("No stderr from cloudflared")?;
            let mut reader = BufReader::new(stderr);

            let mut new_hostname = String::new();
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(30);

            let mut line_buf = String::new();
            loop {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    anyhow::bail!("Timed out waiting for quick tunnel URL");
                }

                line_buf.clear();
                match reader.read_line(&mut line_buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = line_buf.trim();
                        log::info!("cloudflared: {}", line);

                        if let Some(url_start) = line.find("https://") {
                            let url_part = &line[url_start..];
                            if url_part.contains("trycloudflare.com") {
                                let public_url = url_part
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or(url_part)
                                    .trim_end_matches('|')
                                    .to_string();
                                new_hostname = public_url
                                    .trim_start_matches("https://")
                                    .trim_end_matches('/')
                                    .to_string();
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // Drain stderr in background to keep cloudflared alive
            std::thread::spawn(move || {
                let mut buf = String::new();
                loop {
                    buf.clear();
                    match reader.read_line(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => log::debug!("cloudflared: {}", buf.trim()),
                    }
                }
            });

            if !new_hostname.is_empty() {
                log::info!("Quick tunnel restarted with new URL: {}", new_hostname);
                // Update DB with new hostname
                let now = chrono::Utc::now().to_rfc3339();
                if let Ok(db) = rusqlite::Connection::open(&self.db_path) {
                    let _ = db.execute(
                        "UPDATE tunnel_config SET hostname = ?1, updated_at = ?2",
                        rusqlite::params![new_hostname, now],
                    );
                }
                // Update in-memory config
                if let Some(ref mut cfg) = self.config {
                    cfg.hostname = new_hostname;
                }
            }

            self.child = Some(child);
        } else if !config.tunnel_token.is_empty() {
            // API-managed tunnel: run with token
            log::info!("Starting tunnel with token for {}", config.tunnel_name);

            let child = Command::new(&cloudflared_cmd)
                .args(["tunnel", "run", "--token", &config.tunnel_token])
                .spawn()
                .context("Failed to spawn cloudflared")?;

            log::info!("Tunnel started with PID: {:?}", child.id());
            self.child = Some(child);
        } else {
            // Legacy: config file based
            let config_path = self.config_dir.join("tunnel.yml");
            if !config_path.exists() {
                anyhow::bail!("Tunnel config not found. Run setup first.");
            }

            log::info!("Starting tunnel with config: {:?}", config_path);
            let child = Command::new(&cloudflared_cmd)
                .args(["tunnel", "--config", &config_path.to_string_lossy(), "run"])
                .spawn()
                .context("Failed to spawn cloudflared")?;

            log::info!("Tunnel started with PID: {:?}", child.id());
            self.child = Some(child);
        }

        Ok(())
    }

    pub fn stop_tunnel(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill().context("Failed to kill cloudflared process")?;
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn get_status(&mut self) -> TunnelStatus {
        let running = if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    self.child = None;
                    false
                }
            }
        } else {
            false
        };

        TunnelStatus {
            configured: self.config.is_some(),
            running,
            config: self.config.clone(),
            error: None,
        }
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        let _ = self.stop_tunnel();
    }
}
