use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub node_type: String,
    pub node_name: String,
    pub install_path: String,
    pub disk_quota_gb: f64,
    pub bandwidth_limit_mbps: f64,
    pub cpu_limit_percent: f64,
    pub auto_start: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatus {
    pub used_gb: f64,
    pub quota_gb: f64,
    pub file_count: u64,
    pub data_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub uptime_seconds: u64,
    pub storage: StorageStatus,
    pub online: bool,
}

pub struct StorageManager {
    db: Connection,
    install_path: PathBuf,
}

impl StorageManager {
    pub fn initialize(install_path: &str) -> Result<Self> {
        let base = Path::new(install_path);

        // Create directory structure
        fs::create_dir_all(base.join("storage"))
            .context("Failed to create storage directory")?;
        fs::create_dir_all(base.join("config"))
            .context("Failed to create config directory")?;
        fs::create_dir_all(base.join("logs"))
            .context("Failed to create logs directory")?;

        let db_path = base.join("config").join("citinet.db");
        let db = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;

        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS node_config (
                node_id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                node_name TEXT NOT NULL,
                install_path TEXT NOT NULL,
                disk_quota_gb REAL NOT NULL,
                bandwidth_limit_mbps REAL NOT NULL,
                cpu_limit_percent REAL NOT NULL,
                auto_start INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tunnel_config (
                tunnel_id TEXT PRIMARY KEY,
                tunnel_name TEXT NOT NULL,
                hostname TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                api_token TEXT NOT NULL,
                credentials_path TEXT,
                config_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );"
        ).context("Failed to run schema migrations")?;

        Ok(Self {
            db,
            install_path: base.to_path_buf(),
        })
    }

    pub fn open(install_path: &str) -> Result<Self> {
        let base = Path::new(install_path);
        let db_path = base.join("config").join("citinet.db");
        if !db_path.exists() {
            anyhow::bail!("Database not found at {:?}", db_path);
        }
        let db = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;
        Ok(Self {
            db,
            install_path: base.to_path_buf(),
        })
    }

    pub fn save_node_config(
        &self,
        node_type: &str,
        node_name: &str,
        disk_quota_gb: f64,
        bandwidth_limit_mbps: f64,
        cpu_limit_percent: f64,
        auto_start: bool,
    ) -> Result<NodeConfig> {
        let node_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let install_path = self.install_path.to_string_lossy().to_string();

        self.db.execute(
            "INSERT OR REPLACE INTO node_config
                (node_id, node_type, node_name, install_path, disk_quota_gb,
                 bandwidth_limit_mbps, cpu_limit_percent, auto_start, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                node_id, node_type, node_name, install_path,
                disk_quota_gb, bandwidth_limit_mbps, cpu_limit_percent,
                auto_start as i32, now, now
            ],
        ).context("Failed to save node config")?;

        Ok(NodeConfig {
            node_id,
            node_type: node_type.to_string(),
            node_name: node_name.to_string(),
            install_path,
            disk_quota_gb,
            bandwidth_limit_mbps,
            cpu_limit_percent,
            auto_start,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_node_config(&self) -> Result<Option<NodeConfig>> {
        let mut stmt = self.db.prepare(
            "SELECT node_id, node_type, node_name, install_path, disk_quota_gb,
                    bandwidth_limit_mbps, cpu_limit_percent, auto_start, created_at, updated_at
             FROM node_config LIMIT 1"
        ).context("Failed to prepare query")?;

        let mut rows = stmt.query_map([], |row| {
            Ok(NodeConfig {
                node_id: row.get(0)?,
                node_type: row.get(1)?,
                node_name: row.get(2)?,
                install_path: row.get(3)?,
                disk_quota_gb: row.get(4)?,
                bandwidth_limit_mbps: row.get(5)?,
                cpu_limit_percent: row.get(6)?,
                auto_start: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }).context("Failed to query node config")?;

        match rows.next() {
            Some(Ok(config)) => Ok(Some(config)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn update_resource_limits(
        &self,
        disk_quota_gb: f64,
        bandwidth_limit_mbps: f64,
        cpu_limit_percent: f64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE node_config SET disk_quota_gb = ?1, bandwidth_limit_mbps = ?2,
             cpu_limit_percent = ?3, updated_at = ?4",
            rusqlite::params![disk_quota_gb, bandwidth_limit_mbps, cpu_limit_percent, now],
        ).context("Failed to update resource limits")?;
        Ok(())
    }

    pub fn get_storage_status(&self) -> Result<StorageStatus> {
        let storage_path = self.install_path.join("storage");
        let (total_size, file_count) = walk_dir_size(&storage_path)?;

        let quota_gb = match self.get_node_config()? {
            Some(config) => config.disk_quota_gb,
            None => 0.0,
        };

        Ok(StorageStatus {
            used_gb: total_size as f64 / 1_073_741_824.0,
            quota_gb,
            file_count,
            data_path: storage_path.to_string_lossy().to_string(),
        })
    }

    pub fn install_path(&self) -> &Path {
        &self.install_path
    }
}

fn walk_dir_size(path: &Path) -> Result<(u64, u64)> {
    let mut total_size = 0u64;
    let mut file_count = 0u64;

    if !path.exists() {
        return Ok((0, 0));
    }

    for entry in fs::read_dir(path).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let metadata = entry.metadata().context("Failed to read metadata")?;
        if metadata.is_dir() {
            let (size, count) = walk_dir_size(&entry.path())?;
            total_size += size;
            file_count += count;
        } else {
            total_size += metadata.len();
            file_count += 1;
        }
    }

    Ok((total_size, file_count))
}
