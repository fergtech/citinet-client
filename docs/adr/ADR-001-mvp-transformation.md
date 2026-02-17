# ADR-001: MVP Transformation - Mock to Production

**Status**: In Progress  
**Date**: 2026-02-10  
**Deciders**: Development Team  

---

## Context

Transform Citinet client from a working prototype with some real features to a production-ready Windows MVP. Current state has real file operations and system monitoring, but lacks proper storage paths, encryption, role selection, and several core features.

---

## Current State Analysis

### ✅ Already Implemented (Real, Not Mock)
- **File Operations**: Upload/download/delete with real FS operations via Rust
  - Location: `src-tauri/src/storage_manager.rs` (lines 218-280)
  - API: `list_files`, `upload_file`, `delete_file`, `read_file`
  - UI: `src/components/dashboard/FilesPanel.tsx` (fully functional)
  
- **System Monitoring**: Live CPU/memory/disk/network metrics
  - Location: `src-tauri/src/system_monitor.rs`
  - Uses: `sysinfo` crate for real Windows system calls
  - Updates: Every 2 seconds from frontend
  
- **Resource Limits**: Storage/bandwidth/CPU configuration
  - Location: `src-tauri/src/storage_manager.rs` (`update_resource_limits`)
  - Persisted: Plain SQLite database
  - UI: Settings panel with sliders and live updates
  
- **Node Configuration**: Basic SQLite storage
  - Location: `src-tauri/src/storage_manager.rs` (`save_node_config`, `get_node_config`)
  - Schema: `node_config` table with node_id, type, name, quotas
  
- **Cloudflare Tunnel UI**: Setup panel with token input
  - Location: `src/components/dashboard/AdminPanel.tsx` (lines 185-424)
  - Backend: `src-tauri/src/tunnel_manager.rs` (ready but needs work)
  
- **Docker Detection**: Container management UI
  - Location: `src/components/dashboard/AdminPanel.tsx` (lines 13-167)
  - Backend: `src-tauri/src/docker_manager.rs`

### ❌ Critical Issues

#### 1. **Storage Paths (BLOCKER)**
**Current**: User selects install path via wizard (default: `C:\Program Files\Citinet`)
- **Problem**: Admin-only path causes permission errors
- **Files**:
  - `src/stores/wizardStore.ts` (line 27): `installPath: "C:\\Program Files\\Citinet"`
  - `src-tauri/src/storage_manager.rs` (line 50-55): Uses install_path blindly
  
**Required**:
```
%APPDATA%\Citinet\          → C:\Users\<user>\AppData\Roaming\Citinet\
  ├── config/               → citinet.db (encrypted SQLite)
  ├── logs/                 → app.log, errors.log
  
%LOCALAPPDATA%\Citinet\     → C:\Users\<user>\AppData\Local\Citinet\
  ├── storage/              → user files (UUID-named)
  ├── cache/                → temp files, thumbnails
  ├── db/                   → local content cache
  
C:\ProgramData\Citinet\     → cloudflared\ (service binaries)
```

#### 2. **No Role Selection (BLOCKER)**
**Current**: Wizard has no Hub/Client/Personal choice
- **Files**:
  - `src/components/wizard/ServiceStep.tsx` - EXISTS but needs work
  - `src/stores/wizardStore.ts` (line 31): `nodeType: 'client'` (hardcoded default)
  - No capability detection
  - No Docker requirement check for Hub

**Required**:
- Capability scan: CPU cores, RAM, disk, Docker presence, AC power
- Present 3 cards: Hub / Client / Personal with requirements
- Show warnings if insufficient (e.g., "Hub needs Docker Desktop")
- Persist choice to `node_config` table
- Display in Settings → Diagnostics

#### 3. **No Encryption (SECURITY)**
**Current**: Plain SQLite at `{install_path}/config/citinet.db`
- **File**: `src-tauri/src/storage_manager.rs` (line 62): `Connection::open(&db_path)`
- No encryption wrapper
- No DPAPI for secrets (tunnel tokens stored in plaintext)

**Required**:
- SQLCipher or `rusqlite` + encryption layer
- DPAPI via `windows` crate for:
  - Cloudflare tunnel tokens
  - Auth refresh tokens (when implemented)
  - Node private keys (when implemented)

#### 4. **Mock/Missing Features**
- **Discussions**: No implementation
- **Auth/Pairing**: No implementation
- **Reverse Proxy**: Not started
- **Background Sync**: Not started
- **Installer**: No MSI, just bare exe

---

## Decision: Transformation Plan

### Phase 1: Foundation (PR1-PR2) - **THIS WEEK**
**PR1: Storage Path Manager + Encryption**
- Create `StoragePathManager` Rust module
- Use `windows::Win32::UI::Shell::SHGetKnownFolderPath` for proper paths
- Wrap SQLite with encryption (SQLCipher or custom)
- Add DPAPI wrapper for secrets
- Migrate wizard to remove install path selection
- **Files Changed**: 12 files
  - NEW: `src-tauri/src/storage_paths.rs`
  - NEW: `src-tauri/src/crypto.rs`
  - MODIFY: `src-tauri/src/storage_manager.rs`
  - MODIFY: `src-tauri/src/tunnel_manager.rs`
  - MODIFY: `src/components/wizard/LocationStep.tsx` (remove or simplify)
  - MODIFY: `src/stores/wizardStore.ts`
  - MODIFY: `src-tauri/Cargo.toml` (add deps)

**PR2: Role Selection Wizard**
- Add capability detection module
- Redesign ServiceStep as role selection
- Docker Desktop detection + install consent flow
- Persist role to database
- Update Settings panel Diagnostics section
- **Files Changed**: 8 files
  - NEW: `src-tauri/src/capabilities.rs`
  - NEW: `src-tauri/src/docker_installer.rs`
  - MODIFY: `src/components/wizard/ServiceStep.tsx`
  - MODIFY: `src/stores/wizardStore.ts`
  - MODIFY: `src/components/dashboard/SettingsPanel.tsx`

### Phase 2: Real Features (PR3-PR4) - **NEXT WEEK**
**PR3: Discussions (Real)**
- SQLite schema: `posts` table
- Rust commands: `create_post`, `list_posts`, `delete_post`
- UI panel (replace CommunityPanel placeholder)
- **Files Changed**: 6 files
  - NEW: `src-tauri/src/discussions.rs`
  - NEW: `src/components/dashboard/DiscussionsPanel.tsx`
  - MODIFY: `src-tauri/src/storage_manager.rs` (add posts table to schema)
  - MODIFY: `src/components/dashboard/Dashboard.tsx`
  - MODIFY: `src/api/tauri.ts`

**PR4: Auth Scaffolding + Local-Only Mode**
- Pairing UI (enter node URL)
- Local-Only identity for testing
- DPAPI storage for tokens
- **Files Changed**: 7 files
  - NEW: `src-tauri/src/auth.rs`
  - NEW: `src/components/onboarding/PairNode.tsx`
  - MODIFY: `src/components/onboarding/Onboarding.tsx`

### Phase 3: Hub Features (PR5-PR6) - **WEEK 3**
**PR5: Reverse Proxy (Caddy)**
- Download/install Caddy to ProgramData
- Start localhost:8080 proxy
- Health check UI
- **Files Changed**: 5 files
  - NEW: `src-tauri/src/reverse_proxy.rs`
  - MODIFY: `src/components/dashboard/AdminPanel.tsx`

**PR6: Cloudflared Service Manager**
- Download/install cloudflared with consent
- Windows service registration
- Config generation
- Status UI improvements
- **Files Changed**: 3 files
  - MODIFY: `src-tauri/src/tunnel_manager.rs` (complete implementation)
  - MODIFY: `src/components/dashboard/AdminPanel.tsx`

### Phase 4: Polish (PR7-PR9) - **WEEK 4**
**PR7: Background Sync Worker**
- Sync queue table
- Worker thread with task processing
- UI status indicators

**PR8: Contribution Worker**
- Safe task execution (thumbnails, etc.)
- Resource limit enforcement
- Opt-in/out UI

**PR9: Installer + Auto-Update**
- WiX MSI project
- Code signing setup
- Auto-update via Tauri updater
- Clean uninstall

---

## File Structure Changes

### New Files to Create (18)
```
src-tauri/src/
  ├── storage_paths.rs          # Windows known folders API
  ├── crypto.rs                 # DPAPI + SQLite encryption
  ├── capabilities.rs           # System capability detection
  ├── docker_installer.rs       # Docker Desktop download/install
  ├── discussions.rs            # Posts CRUD operations
  ├── auth.rs                   # Pairing & auth flow
  ├── reverse_proxy.rs          # Caddy management
  ├── sync_worker.rs            # Background sync tasks
  ├── contribution_worker.rs    # Safe task execution
  
src/components/
  ├── dashboard/
  │   ├── DiscussionsPanel.tsx  # Real posts UI
  │   └── CapabilityCheck.tsx   # System requirements display
  ├── onboarding/
  │   └── PairNode.tsx          # Node pairing UI
  
installer/
  ├── wix/                      # MSI installer project
  │   ├── main.wxs
  │   └── bundle.wxs
  └── scripts/
      └── sign.ps1              # Code signing script
```

### Files to Modify Significantly (15)
```
src-tauri/src/
  ├── lib.rs                    # Add new command handlers
  ├── storage_manager.rs        # Use new paths, encryption
  ├── tunnel_manager.rs         # Complete implementation
  ├── docker_manager.rs         # Add install capability
  
src-tauri/
  ├── Cargo.toml                # Add: sqlcipher, windows, etc.
  ├── tauri.conf.json           # Update bundle, updater config
  
src/components/
  ├── wizard/
  │   ├── ServiceStep.tsx       # Complete redesign for role selection
  │   └── LocationStep.tsx      # Remove or make read-only
  ├── dashboard/
  │   ├── AdminPanel.tsx        # Enhance connectivity section
  │   ├── SettingsPanel.tsx     # Add diagnostics info
  │   └── Sidebar.tsx           # Add discussions tab
  
src/stores/
  ├── wizardStore.ts            # Remove installPath, add roleSelection
  ├── configStore.ts            # Add paired node URL
  
src/api/
  ├── tauri.ts                  # Add all new commands
```

### Files to Remove (2)
```
src/components/
  └── dashboard/
      └── CommunityPanel.tsx    # Replaced by DiscussionsPanel
```

---

## Dependencies to Add

### Rust (Cargo.toml)
```toml
[dependencies]
# Encryption
sqlcipher = "0.35"  # Or rusqlite with crypto
ring = "0.17"       # Encryption primitives

# Windows APIs
windows = { version = "0.58", features = [
    "Win32_Security_Credentials",
    "Win32_UI_Shell",
    "Win32_System_Services"
]}

# HTTP for downloads
reqwest = { version = "0.12", features = ["blocking", "stream"] }
```

### TypeScript (package.json)
```json
{
  "dependencies": {
    "@tanstack/react-query": "^5.17.0",
    "react-markdown": "^9.0.1",
    "date-fns": "^3.0.0"
  }
}
```

---

## Testing Plan

### Manual Test Checklist (Clean Windows 10 21H2 VM)
- [ ] Fresh install opens wizard
- [ ] Capability detection shows accurate info
- [ ] Hub selection requires Docker (or offers to install)
- [ ] Storage paths are in AppData (verify with Process Monitor)
- [ ] Upload file → persist → restart app → file still there
- [ ] Database file exists and is encrypted (cannot open with sqlite3)
- [ ] Create discussion post → restart → post still there
- [ ] Settings shows correct role and diagnostics
- [ ] Hub: Cloudflared setup flow works with dummy token
- [ ] Hub: Local-Only mode clearly indicated
- [ ] Uninstall removes services but prompts for user data

### Automated Tests (Add to vitest.config.ts)
- Storage path resolution (mock Windows APIs)
- SQLite encryption wrapper
- DPAPI encrypt/decrypt
- Capability detection logic
- File upload quota enforcement
- Post creation validation

---

## Success Metrics

**Week 1 (Foundation)**
- ✅ No "permission denied" errors on storage operations
- ✅ SQLite database encrypted (verified with hex editor)
- ✅ Role wizard functional with Docker detection

**Week 2 (Real Features)**
- ✅ Upload 5 files → restart → all 5 still listed
- ✅ Create 10 posts → restart → all 10 persisted
- ✅ Auth tokens stored in DPAPI (not plaintext)

**Week 3 (Hub Features)**
- ✅ Hub node starts reverse proxy on localhost:8080
- ✅ Cloudflared service registered (visible in services.msc)
- ✅ Health panel shows all component statuses

**Week 4 (Polish)**
- ✅ MSI installs to Program Files, data to AppData
- ✅ Auto-update notifies on new version
- ✅ Export diagnostics creates valid ZIP

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| SQLite encryption breaks migrations | High | Test migrations on encrypted DB; have rollback |
| DPAPI fails on non-domain PCs | Medium | Fallback to best-effort local encryption with warning |
| Docker install requires admin | High | Clear UI: "Hub needs Docker. Click to download installer" |
| Cloudflared download blocked by AV | Medium | Sign binaries; provide manual download link |
| MSI code signing expensive | Low | Self-sign for dev; plan budget for production cert |

---

## Next Actions

1. **Immediate** (Today):
   - [x] Create this ADR
   - [ ] Start PR1: StoragePathManager branch
   - [ ] Implement `storage_paths.rs` with Windows Known Folders
   - [ ] Add DPAPI wrapper in `crypto.rs`

2. **This Week**:
   - [ ] Complete PR1 and merge
   - [ ] Start PR2: Role wizard redesign
   - [ ] Update ServiceStep with capability UI

3. **Document**:
   - [ ] Update README with new install paths
   - [ ] Add ARCHITECTURE.md updates for new modules
   - [ ] Create user guide for role selection

---

## References

- Tauri Security Best Practices: https://tauri.app/v1/guides/security/
- Windows Known Folders: https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
- DPAPI: https://learn.microsoft.com/en-us/windows/win32/api/dpapi/
- WiX Toolset: https://wixtoolset.org/docs/
