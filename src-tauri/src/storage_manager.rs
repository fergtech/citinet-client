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
    pub background_mode: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub conversation_id: String,
    pub kind: String,
    pub name: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMember {
    pub user_id: String,
    pub username: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationWithMembers {
    pub conversation: Conversation,
    pub members: Vec<ConversationMember>,
    pub last_message: Option<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub file_id: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub body: String,
    pub attachments: Vec<MessageAttachment>,
    pub created_at: String,
}

pub struct StorageManager {
    db: Connection,
    install_path: PathBuf,
}

fn run_migrations(db: &Connection) -> Result<()> {
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
        CREATE INDEX IF NOT EXISTS idx_files_is_public ON files(is_public);
        CREATE TABLE IF NOT EXISTS conversations (
            conversation_id TEXT PRIMARY KEY,
            kind TEXT NOT NULL DEFAULT 'dm',
            name TEXT,
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (created_by) REFERENCES users(user_id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS conversation_members (
            conversation_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            joined_at TEXT NOT NULL,
            PRIMARY KEY (conversation_id, user_id),
            FOREIGN KEY (conversation_id) REFERENCES conversations(conversation_id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_conv_members_user ON conversation_members(user_id);
        CREATE TABLE IF NOT EXISTS messages (
            message_id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(conversation_id) ON DELETE CASCADE,
            FOREIGN KEY (sender_id) REFERENCES users(user_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, created_at);
        CREATE TABLE IF NOT EXISTS message_attachments (
            message_id TEXT NOT NULL,
            file_id TEXT NOT NULL,
            PRIMARY KEY (message_id, file_id),
            FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE,
            FOREIGN KEY (file_id) REFERENCES files(file_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_msg_attach_file ON message_attachments(file_id);"
    ).context("Failed to run schema migrations")?;

    // Safe column addition — only runs if column doesn't exist yet
    let has_bg_mode: bool = db.prepare("PRAGMA table_info(node_config)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|col| col.as_deref() == Ok("background_mode"));
    if !has_bg_mode {
        db.execute_batch(
            "ALTER TABLE node_config ADD COLUMN background_mode INTEGER NOT NULL DEFAULT 1;"
        ).context("Failed to add background_mode column")?;
    }

    Ok(())
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

        run_migrations(&db)?;

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
        run_migrations(&db)?;
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
                 bandwidth_limit_mbps, cpu_limit_percent, auto_start, background_mode, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                node_id, node_type, node_name, install_path,
                disk_quota_gb, bandwidth_limit_mbps, cpu_limit_percent,
                auto_start as i32, 1i32, now, now
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
            background_mode: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_node_config(&self) -> Result<Option<NodeConfig>> {
        let mut stmt = self.db.prepare(
            "SELECT node_id, node_type, node_name, install_path, disk_quota_gb,
                    bandwidth_limit_mbps, cpu_limit_percent, auto_start, background_mode,
                    created_at, updated_at
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
                background_mode: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
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

    pub fn update_auto_start(&self, enabled: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE node_config SET auto_start = ?1, updated_at = ?2",
            rusqlite::params![enabled as i32, now],
        ).context("Failed to update auto_start")?;
        Ok(())
    }

    pub fn update_background_mode(&self, enabled: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE node_config SET background_mode = ?1, updated_at = ?2",
            rusqlite::params![enabled as i32, now],
        ).context("Failed to update background_mode")?;
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

    pub fn db(&self) -> &Connection {
        &self.db
    }

    /// Safely relocate all Citinet data to a new path.
    /// Uses copy-verify-rename strategy: old data is never deleted, only renamed as backup.
    pub fn relocate(&mut self, new_path: &str, app_data_dir: &Path) -> Result<String> {
        let new_base = Path::new(new_path);
        let old_base = self.install_path.clone();

        // 1. Validate
        if new_path.is_empty() {
            anyhow::bail!("New path cannot be empty");
        }
        if new_base == old_base {
            anyhow::bail!("New path is the same as current path");
        }
        if new_path.to_lowercase().contains("program files") {
            anyhow::bail!("Cannot relocate to Program Files directory");
        }

        // 2. Check new path is writable
        fs::create_dir_all(new_base).context("Cannot create target directory")?;
        let probe = new_base.join(".citinet_write_test");
        fs::write(&probe, b"test").context("Target directory is not writable")?;
        fs::remove_file(&probe).ok();

        // 3. Check free space
        let (current_size, _) = walk_dir_size(&old_base)?;
        let required_bytes = current_size + (100 * 1024 * 1024); // current + 100MB buffer
        if let Ok(space) = crate::system_monitor::get_drive_space_for_path(new_base) {
            let free_bytes = (space.available_gb * 1_073_741_824.0) as u64;
            if free_bytes < required_bytes {
                anyhow::bail!(
                    "Not enough space on target drive. Need {:.1} GB, have {:.1} GB free",
                    required_bytes as f64 / 1_073_741_824.0,
                    space.available_gb
                );
            }
        }

        // 4. Create directory structure
        for dir in &["storage", "config", "logs", "bin"] {
            fs::create_dir_all(new_base.join(dir))
                .with_context(|| format!("Failed to create {}", dir))?;
        }

        // 5. Close current DB by replacing with a dummy in-memory connection
        //    (we'll reopen the real one after copy)
        let dummy_db = rusqlite::Connection::open_in_memory()?;
        let _old_db = std::mem::replace(&mut self.db, dummy_db);
        drop(_old_db);

        // 6. Copy all files recursively
        let copy_result = copy_dir_recursive(&old_base, new_base);
        if let Err(e) = copy_result {
            // Rollback: reopen old DB
            let db_path = old_base.join("config").join("citinet.db");
            self.db = rusqlite::Connection::open(&db_path)?;
            // Clean up partial copy
            let _ = fs::remove_dir_all(new_base);
            anyhow::bail!("Copy failed, no changes made: {}", e);
        }

        // 7. Verify copy integrity
        let (old_size, old_count) = walk_dir_size(&old_base)?;
        let (new_size, new_count) = walk_dir_size(new_base)?;
        if new_count < old_count || new_size < old_size {
            // Rollback
            let db_path = old_base.join("config").join("citinet.db");
            self.db = rusqlite::Connection::open(&db_path)?;
            let _ = fs::remove_dir_all(new_base);
            anyhow::bail!(
                "Verification failed: expected {} files ({} bytes), got {} files ({} bytes)",
                old_count, old_size, new_count, new_size
            );
        }

        // 8. Open new DB
        let new_db_path = new_base.join("config").join("citinet.db");
        self.db = rusqlite::Connection::open(&new_db_path)
            .context("Failed to open database at new location")?;

        // 9. Update install_path in the DB
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE node_config SET install_path = ?1, updated_at = ?2",
            rusqlite::params![new_path, now],
        ).context("Failed to update install_path in database")?;

        // 10. Update marker file
        let marker_path = app_data_dir.join("install_path.txt");
        fs::write(&marker_path, new_path)
            .context("Failed to update install_path.txt marker file")?;

        // 11. Update in-memory path
        self.install_path = new_base.to_path_buf();

        // 12. Rename old directory as backup (don't delete)
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}_migrated_{}",
            old_base.file_name().unwrap_or_default().to_string_lossy(),
            timestamp
        );
        let backup_path = old_base.parent().unwrap_or(&old_base).join(&backup_name);
        let backup_display = if fs::rename(&old_base, &backup_path).is_ok() {
            backup_path.to_string_lossy().to_string()
        } else {
            // Rename failed (cross-drive) — leave old dir in place
            format!("{} (could not rename, please delete manually)", old_base.display())
        };

        Ok(backup_display)
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

        // Check permissions: owner can always read, others can read public files or files attached to their conversations
        if owner_id != requesting_user_id && !is_public {
            if !self.can_access_attached_file(requesting_user_id, file_name)? {
                anyhow::bail!("Permission denied: file is private");
            }
        }

        let file_path = self.install_path.join("storage").join(file_name);
        fs::read(&file_path).context("Failed to read file")
    }

    pub fn update_file_visibility(&self, requesting_user_id: &str, file_name: &str, is_public: bool) -> Result<()> {
        validate_filename(file_name)?;

        let mut stmt = self.db.prepare(
            "SELECT file_id, user_id FROM files WHERE file_name = ?1"
        )?;
        let file: Option<(String, String)> = stmt.query_row([file_name], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).ok();

        let (file_id, owner_id) = file.ok_or_else(|| anyhow::anyhow!("File not found: {}", file_name))?;

        if owner_id != requesting_user_id {
            let is_admin = self.get_user_by_id(requesting_user_id)?
                .map(|u| u.is_admin)
                .unwrap_or(false);
            if !is_admin {
                anyhow::bail!("Permission denied: not the file owner");
            }
        }

        self.db.execute(
            "UPDATE files SET is_public = ?1 WHERE file_id = ?2",
            rusqlite::params![is_public as i32, file_id],
        )?;

        Ok(())
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

    // --- Messaging methods ---

    pub fn create_dm_conversation(&self, user_a_id: &str, user_b_id: &str) -> Result<Conversation> {
        // Check if a DM already exists between these two users
        let existing = self.db.prepare(
            "SELECT c.conversation_id, c.kind, c.name, c.created_by, c.created_at, c.updated_at
             FROM conversations c
             JOIN conversation_members cm1 ON c.conversation_id = cm1.conversation_id
             JOIN conversation_members cm2 ON c.conversation_id = cm2.conversation_id
             WHERE c.kind = 'dm' AND cm1.user_id = ?1 AND cm2.user_id = ?2"
        )?.query_row(rusqlite::params![user_a_id, user_b_id], |row| {
            Ok(Conversation {
                conversation_id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                created_by: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).ok();

        if let Some(conv) = existing {
            return Ok(conv);
        }

        let conversation_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO conversations (conversation_id, kind, name, created_by, created_at, updated_at)
             VALUES (?1, 'dm', NULL, ?2, ?3, ?4)",
            rusqlite::params![conversation_id, user_a_id, now, now],
        ).context("Failed to create conversation")?;

        for uid in &[user_a_id, user_b_id] {
            self.db.execute(
                "INSERT INTO conversation_members (conversation_id, user_id, joined_at)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![conversation_id, uid, now],
            ).context("Failed to add conversation member")?;
        }

        Ok(Conversation {
            conversation_id,
            kind: "dm".to_string(),
            name: None,
            created_by: user_a_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn create_group_conversation(
        &self,
        creator_id: &str,
        name: &str,
        member_ids: &[String],
    ) -> Result<Conversation> {
        let conversation_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO conversations (conversation_id, kind, name, created_by, created_at, updated_at)
             VALUES (?1, 'group', ?2, ?3, ?4, ?5)",
            rusqlite::params![conversation_id, name, creator_id, now, now],
        ).context("Failed to create group conversation")?;

        // Add creator as member
        self.db.execute(
            "INSERT INTO conversation_members (conversation_id, user_id, joined_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![conversation_id, creator_id, now],
        )?;

        // Add other members
        for uid in member_ids {
            if uid != creator_id {
                self.db.execute(
                    "INSERT OR IGNORE INTO conversation_members (conversation_id, user_id, joined_at)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![conversation_id, uid, now],
                )?;
            }
        }

        Ok(Conversation {
            conversation_id,
            kind: "group".to_string(),
            name: Some(name.to_string()),
            created_by: creator_id.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn add_group_member(&self, conversation_id: &str, user_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "INSERT OR IGNORE INTO conversation_members (conversation_id, user_id, joined_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![conversation_id, user_id, now],
        ).context("Failed to add group member")?;
        Ok(())
    }

    pub fn remove_group_member(&self, conversation_id: &str, user_id: &str) -> Result<()> {
        self.db.execute(
            "DELETE FROM conversation_members WHERE conversation_id = ?1 AND user_id = ?2",
            rusqlite::params![conversation_id, user_id],
        ).context("Failed to remove group member")?;
        Ok(())
    }

    pub fn rename_conversation(&self, conversation_id: &str, name: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "UPDATE conversations SET name = ?1, updated_at = ?2 WHERE conversation_id = ?3",
            rusqlite::params![name, now, conversation_id],
        ).context("Failed to rename conversation")?;
        Ok(())
    }

    pub fn is_conversation_member(&self, conversation_id: &str, user_id: &str) -> Result<bool> {
        self.db.prepare(
            "SELECT 1 FROM conversation_members WHERE conversation_id = ?1 AND user_id = ?2"
        )?.exists(rusqlite::params![conversation_id, user_id])
            .context("Failed to check membership")
    }

    pub fn get_conversation_members(&self, conversation_id: &str) -> Result<Vec<ConversationMember>> {
        let mut stmt = self.db.prepare(
            "SELECT cm.user_id, u.username, cm.joined_at
             FROM conversation_members cm
             JOIN users u ON cm.user_id = u.user_id
             WHERE cm.conversation_id = ?1"
        )?;

        let members = stmt.query_map([conversation_id], |row| {
            Ok(ConversationMember {
                user_id: row.get(0)?,
                username: row.get(1)?,
                joined_at: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(members)
    }

    fn get_last_message(&self, conversation_id: &str) -> Result<Option<Message>> {
        let mut stmt = self.db.prepare(
            "SELECT m.message_id, m.conversation_id, m.sender_id, u.username, m.body, m.created_at
             FROM messages m
             JOIN users u ON m.sender_id = u.user_id
             WHERE m.conversation_id = ?1
             ORDER BY m.created_at DESC
             LIMIT 1"
        )?;

        let mut rows = stmt.query_map([conversation_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        match rows.next() {
            Some(Ok((message_id, conv_id, sender_id, username, body, created_at))) => {
                let attachments = self.get_message_attachments(&message_id)?;
                Ok(Some(Message {
                    message_id,
                    conversation_id: conv_id,
                    sender_id,
                    sender_username: username,
                    body,
                    attachments,
                    created_at,
                }))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_conversations(&self, user_id: &str) -> Result<Vec<ConversationWithMembers>> {
        let mut stmt = self.db.prepare(
            "SELECT c.conversation_id, c.kind, c.name, c.created_by, c.created_at, c.updated_at
             FROM conversations c
             JOIN conversation_members cm ON c.conversation_id = cm.conversation_id
             WHERE cm.user_id = ?1
             ORDER BY c.updated_at DESC"
        )?;

        let convs: Vec<Conversation> = stmt.query_map([user_id], |row| {
            Ok(Conversation {
                conversation_id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                created_by: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for conv in convs {
            let members = self.get_conversation_members(&conv.conversation_id)?;
            let last_message = self.get_last_message(&conv.conversation_id)?;
            result.push(ConversationWithMembers {
                conversation: conv,
                members,
                last_message,
            });
        }

        Ok(result)
    }

    pub fn create_message(
        &self,
        conversation_id: &str,
        sender_id: &str,
        body: &str,
        attachment_ids: &[String],
    ) -> Result<Message> {
        let is_member = self.is_conversation_member(conversation_id, sender_id)?;
        if !is_member {
            anyhow::bail!("User is not a member of this conversation");
        }

        let message_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO messages (message_id, conversation_id, sender_id, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![message_id, conversation_id, sender_id, body, now],
        ).context("Failed to create message")?;

        // Link attachments
        let mut attachments = Vec::new();
        for file_id in attachment_ids {
            // Verify file exists and sender owns it
            let file: Option<(String, u64)> = self.db.prepare(
                "SELECT file_name, size_bytes FROM files WHERE file_id = ?1 AND user_id = ?2"
            )?.query_row(rusqlite::params![file_id, sender_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            }).ok();

            if let Some((file_name, size_bytes)) = file {
                self.db.execute(
                    "INSERT OR IGNORE INTO message_attachments (message_id, file_id) VALUES (?1, ?2)",
                    rusqlite::params![message_id, file_id],
                )?;
                attachments.push(MessageAttachment {
                    file_id: file_id.clone(),
                    mime_type: mime_from_ext(&file_name),
                    file_name,
                    size_bytes,
                });
            }
        }

        self.db.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE conversation_id = ?2",
            rusqlite::params![now, conversation_id],
        )?;

        let username = self.get_user_by_id(sender_id)?
            .map(|u| u.username)
            .unwrap_or_default();

        Ok(Message {
            message_id,
            conversation_id: conversation_id.to_string(),
            sender_id: sender_id.to_string(),
            sender_username: username,
            body: body.to_string(),
            attachments,
            created_at: now,
        })
    }

    fn get_message_attachments(&self, message_id: &str) -> Result<Vec<MessageAttachment>> {
        let mut stmt = self.db.prepare(
            "SELECT f.file_id, f.file_name, f.size_bytes
             FROM message_attachments ma
             JOIN files f ON ma.file_id = f.file_id
             WHERE ma.message_id = ?1"
        )?;
        let rows = stmt.query_map([message_id], |row| {
            let file_name: String = row.get(1)?;
            Ok(MessageAttachment {
                file_id: row.get(0)?,
                mime_type: mime_from_ext(&file_name),
                file_name,
                size_bytes: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Check if a file is attached to any conversation the user is a member of
    pub fn can_access_attached_file(&self, user_id: &str, file_name: &str) -> Result<bool> {
        let exists = self.db.prepare(
            "SELECT 1 FROM files f
             JOIN message_attachments ma ON f.file_id = ma.file_id
             JOIN messages m ON ma.message_id = m.message_id
             JOIN conversation_members cm ON m.conversation_id = cm.conversation_id
             WHERE f.file_name = ?1 AND cm.user_id = ?2
             LIMIT 1"
        )?.exists(rusqlite::params![file_name, user_id])?;
        Ok(exists)
    }

    pub fn list_messages(
        &self,
        conversation_id: &str,
        limit: u32,
        before: Option<&str>,
    ) -> Result<Vec<Message>> {
        let raw_rows: Vec<(String, String, String, String, String, String)> = if let Some(cursor) = before {
            let mut stmt = self.db.prepare(
                "SELECT m.message_id, m.conversation_id, m.sender_id, u.username, m.body, m.created_at
                 FROM messages m
                 JOIN users u ON m.sender_id = u.user_id
                 WHERE m.conversation_id = ?1 AND m.created_at < ?2
                 ORDER BY m.created_at DESC
                 LIMIT ?3"
            )?;
            let result = stmt.query_map(rusqlite::params![conversation_id, cursor, limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
            })?.collect::<Result<Vec<_>, _>>()?;
            result
        } else {
            let mut stmt = self.db.prepare(
                "SELECT m.message_id, m.conversation_id, m.sender_id, u.username, m.body, m.created_at
                 FROM messages m
                 JOIN users u ON m.sender_id = u.user_id
                 WHERE m.conversation_id = ?1
                 ORDER BY m.created_at DESC
                 LIMIT ?2"
            )?;
            let result = stmt.query_map(rusqlite::params![conversation_id, limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
            })?.collect::<Result<Vec<_>, _>>()?;
            result
        };

        let mut messages = Vec::with_capacity(raw_rows.len());
        for (message_id, conv_id, sender_id, username, body, created_at) in raw_rows {
            let attachments = self.get_message_attachments(&message_id)?;
            messages.push(Message {
                message_id,
                conversation_id: conv_id,
                sender_id,
                sender_username: username,
                body,
                attachments,
                created_at,
            });
        }
        Ok(messages)
    }
}

fn mime_from_ext(name: &str) -> String {
    match name.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("bmp") => "image/bmp".to_string(),
        Some("ico") => "image/x-icon".to_string(),
        Some("mp4") | Some("m4v") => "video/mp4".to_string(),
        Some("webm") => "video/webm".to_string(),
        Some("mov") => "video/quicktime".to_string(),
        Some("avi") => "video/x-msvideo".to_string(),
        Some("mkv") => "video/x-matroska".to_string(),
        Some("ogv") => "video/ogg".to_string(),
        Some("3gp") => "video/3gpp".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("txt") => "text/plain".to_string(),
        Some("json") => "application/json".to_string(),
        Some("zip") => "application/zip".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn validate_filename(name: &str) -> Result<()> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        anyhow::bail!("Invalid filename: {}", name);
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).context("Failed to read source directory")? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.metadata()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {:?}", src_path))?;
        }
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
