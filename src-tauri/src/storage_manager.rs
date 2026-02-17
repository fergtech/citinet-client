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
pub struct FileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub created_at: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub space_id: String,
    pub user_id: String,
    pub name: String,
    pub storage_quota_gb: f64,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub file_id: String,
    pub user_id: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub is_public: bool,
    pub created_at: String,
}

pub struct StorageManager {
    db: Connection,
    install_path: PathBuf,
}

impl StorageManager {
    pub fn initialize(install_path: &str) -> Result<Self> {
        let base = Path::new(install_path);

        // Create directory structure with better error messages
        fs::create_dir_all(base.join("storage"))
            .with_context(|| format!("Failed to create storage directory at '{}'. Check permissions and ensure the path is valid.", base.join("storage").display()))?;
        fs::create_dir_all(base.join("config"))
            .with_context(|| format!("Failed to create config directory at '{}'. Check permissions.", base.join("config").display()))?;
        fs::create_dir_all(base.join("logs"))
            .with_context(|| format!("Failed to create logs directory at '{}'. Check permissions.", base.join("logs").display()))?;

        let db_path = base.join("config").join("citinet.db");
        let db = Connection::open(&db_path)
            .with_context(|| format!("Failed to open SQLite database at '{}'. Check permissions.", db_path.display()))?;

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
            );
            CREATE TABLE IF NOT EXISTS users (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                is_admin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS spaces (
                space_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                storage_quota_gb REAL NOT NULL DEFAULT 5.0,
                is_public INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_spaces_user_id ON spaces(user_id);
            CREATE TABLE IF NOT EXISTS files (
                file_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id);
            CREATE INDEX IF NOT EXISTS idx_files_is_public ON files(is_public);"
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

    pub fn upload_file(&self, user_id: &str, file_name: &str, file_data: &[u8], is_public: bool) -> Result<File> {
        validate_filename(file_name)?;
        
        // Write file to disk
        let storage_path = self.install_path.join("storage");
        fs::create_dir_all(&storage_path).context("Failed to create storage directory")?;
        let file_path = storage_path.join(file_name);
        fs::write(&file_path, file_data).context("Failed to write file")?;

        // Insert metadata to database
        let file_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let size_bytes = file_data.len() as u64;

        self.db.execute(
            "INSERT INTO files (file_id, user_id, file_name, size_bytes, is_public, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                file_id, user_id, file_name, size_bytes, is_public as i32, now
            ],
        ).context("Failed to insert file metadata")?;

        Ok(File {
            file_id,
            user_id: user_id.to_string(),
            file_name: file_name.to_string(),
            size_bytes,
            is_public,
            created_at: now,
        })
    }

    pub fn list_files(&self, requesting_user_id: Option<&str>) -> Result<Vec<File>> {
        // If no user specified, return all public files
        // If user specified, return their files + all public files
        let files = match requesting_user_id {
            Some(uid) => {
                let mut stmt = self.db.prepare(
                    "SELECT file_id, user_id, file_name, size_bytes, is_public, created_at
                     FROM files
                     WHERE user_id = ?1 OR is_public = 1
                     ORDER BY created_at DESC"
                )?;
                let rows = stmt.query_map([uid], |row| {
                    Ok(File {
                        file_id: row.get(0)?,
                        user_id: row.get(1)?,
                        file_name: row.get(2)?,
                        size_bytes: row.get(3)?,
                        is_public: row.get::<_, i32>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            }
            None => {
                let mut stmt = self.db.prepare(
                    "SELECT file_id, user_id, file_name, size_bytes, is_public, created_at
                     FROM files
                     WHERE is_public = 1
                     ORDER BY created_at DESC"
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok(File {
                        file_id: row.get(0)?,
                        user_id: row.get(1)?,
                        file_name: row.get(2)?,
                        size_bytes: row.get(3)?,
                        is_public: row.get::<_, i32>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(files)
    }

    pub fn delete_file(&self, requesting_user_id: &str, file_name: &str) -> Result<()> {
        validate_filename(file_name)?;

        // Check if file exists in database and user owns it
        let mut stmt = self.db.prepare(
            "SELECT file_id, user_id FROM files WHERE file_name = ?1"
        )?;
        let file: Option<(String, String)> = stmt.query_row([file_name], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).ok();

        let (file_id, owner_id) = file.ok_or_else(|| anyhow::anyhow!("File not found: {}", file_name))?;

        // Check ownership
        if owner_id != requesting_user_id {
            // Check if requesting user is admin
            let is_admin = self.get_user_by_id(requesting_user_id)?
                .map(|u| u.is_admin)
                .unwrap_or(false);
            
            if !is_admin {
                anyhow::bail!("Permission denied: not the file owner");
            }
        }

        // Delete from filesystem
        let file_path = self.install_path.join("storage").join(file_name);
        if file_path.exists() {
            fs::remove_file(&file_path).context("Failed to delete file")?;
        }

        // Delete from database
        self.db.execute("DELETE FROM files WHERE file_id = ?1", [file_id])?;
        
        Ok(())
    }

    pub fn read_file(&self, requesting_user_id: &str, file_name: &str) -> Result<Vec<u8>> {
        validate_filename(file_name)?;

        // Check if file exists and user has permission
        let mut stmt = self.db.prepare(
            "SELECT user_id, is_public FROM files WHERE file_name = ?1"
        )?;
        let file: Option<(String, bool)> = stmt.query_row([file_name], |row| {
            Ok((row.get(0)?, row.get::<_, i32>(1)? != 0))
        }).ok();

        let (owner_id, is_public) = file.ok_or_else(|| anyhow::anyhow!("File not found: {}", file_name))?;

        // Check permissions: owner can always read, others can only read public files
        if owner_id != requesting_user_id && !is_public {
            anyhow::bail!("Permission denied: file is private");
        }

        let file_path = self.install_path.join("storage").join(file_name);
        fs::read(&file_path).context("Failed to read file")
    }

    // --- User management methods ---

    pub fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> Result<User> {
        // If user already exists, update their password and return them
        if let Ok(Some(existing)) = self.get_user_by_username(username) {
            let now = Utc::now().to_rfc3339();
            self.db.execute(
                "UPDATE users SET password_hash = ?1, email = ?2, is_admin = ?3, updated_at = ?4 WHERE user_id = ?5",
                rusqlite::params![password_hash, email, is_admin as i32, now, existing.user_id],
            ).context("Failed to update existing user")?;
            return Ok(User {
                user_id: existing.user_id,
                username: existing.username,
                email: email.to_string(),
                is_admin,
                created_at: existing.created_at,
                updated_at: now,
            });
        }

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO users (user_id, username, email, password_hash, is_admin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                user_id, username, email, password_hash, is_admin as i32, now, now
            ],
        ).context("Failed to create user")?;

        Ok(User {
            user_id,
            username: username.to_string(),
            email: email.to_string(),
            is_admin,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let mut stmt = self.db.prepare(
            "SELECT user_id, username, email, is_admin, created_at, updated_at
             FROM users WHERE username = ?1"
        ).context("Failed to prepare query")?;

        let mut rows = stmt.query_map([username], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                is_admin: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).context("Failed to query user")?;

        match rows.next() {
            Some(Ok(user)) => Ok(Some(user)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let mut stmt = self.db.prepare(
            "SELECT user_id, username, email, is_admin, created_at, updated_at
             FROM users WHERE user_id = ?1"
        ).context("Failed to prepare query")?;

        let mut rows = stmt.query_map([user_id], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                is_admin: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).context("Failed to query user")?;

        match rows.next() {
            Some(Ok(user)) => Ok(Some(user)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_password_hash(&self, username: &str) -> Result<Option<String>> {
        let mut stmt = self.db.prepare(
            "SELECT password_hash FROM users WHERE username = ?1"
        ).context("Failed to prepare query")?;

        let mut rows = stmt.query_map([username], |row| {
            row.get::<_, String>(0)
        }).context("Failed to query password hash")?;

        match rows.next() {
            Some(Ok(hash)) => Ok(Some(hash)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.db.prepare(
            "SELECT user_id, username, email, is_admin, created_at, updated_at
             FROM users ORDER BY created_at DESC"
        ).context("Failed to prepare query")?;

        let users = stmt.query_map([], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                is_admin: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).context("Failed to query users")?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    pub fn get_first_admin(&self) -> Result<Option<User>> {
        let mut stmt = self.db.prepare(
            "SELECT user_id, username, email, is_admin, created_at, updated_at
             FROM users WHERE is_admin = 1 ORDER BY created_at ASC LIMIT 1"
        ).context("Failed to prepare query")?;

        let mut rows = stmt.query_map([], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                is_admin: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).context("Failed to query admin")?;

        match rows.next() {
            Some(Ok(user)) => Ok(Some(user)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn delete_user(&self, user_id: &str) -> Result<()> {
        // Delete user's files from disk
        let files = self.list_files(Some(user_id))?;
        for f in &files {
            let file_path = self.install_path.join("storage").join(&f.file_name);
            if file_path.exists() {
                let _ = fs::remove_file(&file_path);
            }
        }
        // Cascade deletes files and spaces via FK
        self.db.execute("DELETE FROM users WHERE user_id = ?1", [user_id])
            .context("Failed to delete user")?;
        Ok(())
    }

    pub fn update_user_role(&self, user_id: &str, is_admin: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE users SET is_admin = ?1, updated_at = ?2 WHERE user_id = ?3",
            rusqlite::params![is_admin as i32, now, user_id],
        ).context("Failed to update user role")?;
        Ok(())
    }

    pub fn list_all_files(&self) -> Result<Vec<File>> {
        // Admin-only method to list ALL files regardless of ownership
        let mut stmt = self.db.prepare(
            "SELECT file_id, user_id, file_name, size_bytes, is_public, created_at
             FROM files
             ORDER BY created_at DESC"
        )?;

        let files = stmt.query_map([], |row| {
            Ok(File {
                file_id: row.get(0)?,
                user_id: row.get(1)?,
                file_name: row.get(2)?,
                size_bytes: row.get(3)?,
                is_public: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    pub fn create_space(
        &self,
        user_id: &str,
        name: &str,
        storage_quota_gb: f64,
        is_public: bool,
    ) -> Result<Space> {
        let space_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO spaces (space_id, user_id, name, storage_quota_gb, is_public, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                space_id, user_id, name, storage_quota_gb, is_public as i32, now, now
            ],
        ).context("Failed to create space")?;

        Ok(Space {
            space_id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            storage_quota_gb,
            is_public,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_user_spaces(&self, user_id: &str) -> Result<Vec<Space>> {
        let mut stmt = self.db.prepare(
            "SELECT space_id, user_id, name, storage_quota_gb, is_public, created_at, updated_at
             FROM spaces WHERE user_id = ?1 ORDER BY created_at DESC"
        ).context("Failed to prepare query")?;

        let spaces = stmt.query_map([user_id], |row| {
            Ok(Space {
                space_id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                storage_quota_gb: row.get(3)?,
                is_public: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).context("Failed to query spaces")?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(spaces)
    }
}

fn validate_filename(name: &str) -> Result<()> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        anyhow::bail!("Invalid filename: {}", name);
    }
    Ok(())
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
