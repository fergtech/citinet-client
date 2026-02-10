use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub configured: bool,
    pub running: bool,
    pub config: Option<TunnelConfig>,
    pub error: Option<String>,
}

pub fn check_cloudflared() -> CloudflaredStatus {
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
            error: Some("cloudflared not found in PATH".to_string()),
        },
    }
}

pub struct TunnelManager {
    config_dir: PathBuf,
    child: Option<Child>,
    config: Option<TunnelConfig>,
    db_path: PathBuf,
}

impl TunnelManager {
    pub fn new(install_path: &Path) -> Self {
        let config_dir = install_path.join("config");
        let db_path = config_dir.join("citinet.db");

        // Try to load existing tunnel config from DB
        let config = Self::load_config_from_db(&db_path);

        Self {
            config_dir,
            child: None,
            config,
            db_path,
        }
    }

    fn load_config_from_db(db_path: &Path) -> Option<TunnelConfig> {
        let db = rusqlite::Connection::open(db_path).ok()?;
        let mut stmt = db.prepare(
            "SELECT tunnel_id, tunnel_name, hostname, local_port, created_at
             FROM tunnel_config LIMIT 1"
        ).ok()?;

        stmt.query_row([], |row| {
            Ok(TunnelConfig {
                tunnel_id: row.get(0)?,
                tunnel_name: row.get(1)?,
                hostname: row.get(2)?,
                local_port: row.get::<_, u32>(3)? as u16,
                created_at: row.get(4)?,
            })
        }).ok()
    }

    pub fn setup_tunnel(
        &mut self,
        api_token: &str,
        tunnel_name: &str,
        hostname: &str,
        local_port: u16,
    ) -> Result<TunnelConfig> {
        // Write credentials / token file
        let creds_path = self.config_dir.join("cf-token.txt");
        std::fs::write(&creds_path, api_token)
            .context("Failed to write credentials file")?;

        // Create tunnel
        let create_output = Command::new("cloudflared")
            .args(["tunnel", "create", tunnel_name])
            .env("TUNNEL_ORIGIN_CERT", creds_path.to_string_lossy().to_string())
            .output()
            .context("Failed to run cloudflared tunnel create")?;

        let tunnel_id = if create_output.status.success() {
            // Parse tunnel ID from output
            let stdout = String::from_utf8_lossy(&create_output.stdout);
            // Output typically contains "Created tunnel <name> with id <uuid>"
            stdout.split_whitespace()
                .last()
                .unwrap_or(tunnel_name)
                .to_string()
        } else {
            let stderr = String::from_utf8_lossy(&create_output.stderr);
            // If tunnel already exists, try to continue
            if stderr.contains("already exists") {
                tunnel_name.to_string()
            } else {
                anyhow::bail!("Failed to create tunnel: {}", stderr.trim());
            }
        };

        // Route DNS
        let _ = Command::new("cloudflared")
            .args(["tunnel", "route", "dns", tunnel_name, hostname])
            .output();

        // Generate tunnel config YAML
        let tunnel_yml = format!(
            "tunnel: {tunnel_name}\n\
             credentials-file: {creds}\n\
             ingress:\n  \
               - hostname: {hostname}\n    \
                 service: http://localhost:{local_port}\n  \
               - service: http_status:404\n",
            tunnel_name = tunnel_name,
            creds = creds_path.to_string_lossy(),
            hostname = hostname,
            local_port = local_port,
        );

        let config_path = self.config_dir.join("tunnel.yml");
        std::fs::write(&config_path, &tunnel_yml)
            .context("Failed to write tunnel config")?;

        let now = Utc::now().to_rfc3339();
        let config = TunnelConfig {
            tunnel_id: tunnel_id.clone(),
            tunnel_name: tunnel_name.to_string(),
            hostname: hostname.to_string(),
            local_port,
            created_at: now.clone(),
        };

        // Persist to SQLite
        let db = rusqlite::Connection::open(&self.db_path)
            .context("Failed to open database")?;
        db.execute(
            "INSERT OR REPLACE INTO tunnel_config
                (tunnel_id, tunnel_name, hostname, local_port, api_token, credentials_path, config_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                tunnel_id,
                tunnel_name,
                hostname,
                local_port as u32,
                api_token,
                creds_path.to_string_lossy().to_string(),
                config_path.to_string_lossy().to_string(),
                now,
                now,
            ],
        ).context("Failed to save tunnel config")?;

        self.config = Some(config.clone());
        Ok(config)
    }

    pub fn start_tunnel(&mut self) -> Result<()> {
        if self.child.is_some() {
            anyhow::bail!("Tunnel is already running");
        }

        let config_path = self.config_dir.join("tunnel.yml");
        if !config_path.exists() {
            anyhow::bail!("Tunnel config not found. Run setup first.");
        }

        let child = Command::new("cloudflared")
            .args(["tunnel", "--config", &config_path.to_string_lossy(), "run"])
            .spawn()
            .context("Failed to spawn cloudflared")?;

        self.child = Some(child);
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
        // Check if child process is still alive
        let running = if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited
                    self.child = None;
                    false
                }
                Ok(None) => true, // Still running
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
