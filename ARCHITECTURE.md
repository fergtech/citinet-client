# Citinet Architecture

## Overview

Citinet is a web-first community network platform. A **hub** is a Docker Compose stack that any machine (PC, Raspberry Pi, VPS, or NAS) can run. The hub exposes a local HTTP API on port 9090, and a public access tunnel (Tailscale Funnel, Cloudflare, or a reverse proxy) makes it reachable on the internet. Community members connect through a browser at `{slug}.citinet.cloud` — no installation required.

This **citinet-client** desktop app is the hub operator's management interface: a Tauri 2 + React Windows app that handles Docker lifecycle, tunnel setup, user management, and hub configuration. It is an admin/operator tool, not a client app for community members.

---

## Node Types

### 1. Hub Node (Community Micro-Data Center)

**Purpose**: Provides services to the local community

**Hardware**: Typically Raspberry Pi + workstation + network switch

**Capabilities**:
- Broadcasts availability via mDNS
- Hosts community services (identity, storage, messaging, social)
- Manages resource allocation
- Serves as local network gateway

**Deployment**: Home, library, community center, school

**Software Mode**: Runs with `node_type: 'hub'`

### 2. Web Client (Browser-Based Participant)

**Purpose**: Connects to Hub Nodes to use services

**Hardware**: Any device with a browser

**Capabilities**:
- Discovers hubs via the citinet.cloud registry
- Accesses hub services (files, messaging, discussions) via web portal
- No installation required — opens `start.citinet.cloud` and joins
- Bookmarks hub at `{slug}.citinet.cloud`

**Deployment**: `start.citinet.cloud` / `{slug}.citinet.cloud` (hosted on Vercel + CF Workers)

### 3. Personal Node (Optional Sovereign Device)

**Purpose**: User's primary data store that syncs to hubs

**Hardware**: Desktop, laptop, or dedicated device

**Capabilities**:
- Local-first data storage
- Syncs with Hub Nodes when online
- Works offline by default
- Full data sovereignty

**Deployment**: User's personal device

**Software Mode**: Runs with `node_type: 'personal'`

---

## Technical Stack

### Frontend (React/TypeScript)
- **Framework**: React 19
- **Language**: TypeScript
- **Styling**: Tailwind CSS 4
- **State Management**: Zustand with localStorage persistence
- **Icons**: Lucide React
- **Build Tool**: Vite 7

### Backend (Rust)
- **Framework**: Tauri 2
- **System Monitoring**: `sysinfo` crate
- **Network Discovery**: `mdns-sd` (mDNS/DNS-SD)
- **Database**: `rusqlite` (SQLite)
- **Async Runtime**: `tokio`
- **Error Handling**: `anyhow`

### Communication Layer
- **Frontend ↔ Backend**: Tauri IPC (invoke commands)
- **Node Discovery**: mDNS broadcasting and discovery
- **Future**: WebRTC for peer-to-peer, ActivityPub for federation

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                        │
│  ┌──────────────┬──────────────┬──────────────────────┐    │
│  │   Dashboard  │  Onboarding  │  Wizard (Setup)       │    │
│  └──────────────┴──────────────┴──────────────────────┘    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          State Management (Zustand)                   │   │
│  │   • appStore    • configStore   • onboardingStore    │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↕ Tauri IPC
┌─────────────────────────────────────────────────────────────┐
│                     Backend (Rust)                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              System Monitor                           │   │
│  │  • CPU/Memory/Disk tracking                          │   │
│  │  • Network bandwidth monitoring                      │   │
│  │  • Hardware detection (Pi vs PC)                     │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Hub Service                              │   │
│  │  • mDNS broadcasting (Hub Nodes)                     │   │
│  │  • mDNS discovery (Client Nodes)                     │   │
│  │  • Service registry                                  │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │        Resource Management (Future)                   │   │
│  │  • Enforce contribution limits                       │   │
│  │  • Track actual usage vs configured                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                   Network Layer                             │
│  • mDNS: _citinet._tcp.local.                              │
│  • Local network discovery                                 │
│  • Future: WireGuard tunnels, Yggdrasil overlay            │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Resource Monitoring

```
1. Timer triggers every 2 seconds (frontend)
2. Frontend → CitinetAPI.getSystemMetrics()
3. Tauri IPC → get_system_metrics command
4. Backend queries SystemMonitor
5. SystemMonitor uses sysinfo crate to read OS metrics
6. Returns: CPU %, memory, disk, network, uptime
7. Frontend displays in MetricsPanel
```

### Hub Discovery (Client Node)

```
1. User completes onboarding
2. Frontend → CitinetAPI.startNodeDiscovery()
3. Backend initializes mDNS service daemon
4. Daemon listens for _citinet._tcp.local. broadcasts
5. Background thread receives ServiceResolved events
6. Updates discovered_nodes HashMap
7. Frontend polls CitinetAPI.getDiscoveredNodes()
8. Displays available hubs in CommunityPanel
```

### Hub Broadcasting (Hub Node)

```
1. User completes wizard, selects Hub mode
2. Frontend → CitinetAPI.startHubBroadcasting(name, services)
3. Backend creates ServiceInfo with node metadata
4. Registers service with mDNS daemon
5. Broadcasts on local network continuously
6. Client nodes discover via mDNS
```

---

## State Management

### Frontend Stores (Zustand)

**appStore.ts**
- Current UI phase: wizard | onboarding | dashboard
- Theme: light | dark | system
- Persisted in localStorage

**configStore.ts** (New)
- Node type: hub | client | personal
- Node name and install path
- Resource contribution limits:
  - Disk space (GB)
  - Bandwidth (Mbps)
  - CPU (%)
- **Persisted in localStorage** via Zustand middleware

**wizardStore.ts**
- Installation wizard state
- License acceptance, install path
- Node type selection (added recently)

**onboardingStore.ts**
- User profile information
- Onboarding progress

### Backend State (Rust)

**AppState**
- `SystemMonitor`: Tracks system metrics
- `HubService`: Manages mDNS and discovered nodes

---

## Implementation Status

### ✅ Completed

- [x] Hub setup wizard (node naming, storage allocation, admin account)
- [x] Docker Compose lifecycle management (start, stop, status)
- [x] Embedded axum HTTP API server on port 9090
- [x] User authentication (bcrypt + JWT)
- [x] File storage (upload/download/delete/visibility toggle)
- [x] Real-time messaging (WebSocket, DMs + group conversations)
- [x] Cloudflare Quick Tunnel + Custom Domain tunnel modes
- [x] Tailscale Funnel (stable IPv4+IPv6 URL, recommended gateway)
- [x] Tunnel auto-start + watchdog (restarts crashed tunnels every 30s)
- [x] Hub registry auto-registration on every tunnel start
- [x] System monitoring (CPU, memory, disk, network metrics)
- [x] Auto-start on boot, background/tray mode
- [x] Feature flag system (profile-based per node type)
- [x] Factory reset, auto-updater

### 🚧 In Progress

- [ ] Web-based admin panel (manage hub config, users, tunnels from the browser)
- [ ] Hub-served web app (Docker stack serves web portal for offline/local access)

### 📋 Planned

- [ ] Simplified one-click launcher (Python/Tkinter) for non-technical hub operators
- [ ] Personal Node sync engine (local-first data storage)
- [ ] ActivityPub federation for social features
- [ ] Matrix server integration for messaging
- [ ] Encrypted peer-to-peer connections (WireGuard)
- [ ] Mobile-first improvements to web portal
- [ ] Linux / Raspberry Pi hub management app

---

## Platform Support

### Desktop (Current)
- **Windows**: Full support
- **macOS**: Full support (Tauri 2)
- **Linux**: Full support (Tauri 2)

### Raspberry Pi
- **OS**: Raspberry Pi OS (Debian-based)
- **Architecture**: ARM64/ARMv7
- **Mode**: Hub Node (lightweight services)
- **Deployment**: Cross-compile Rust for ARM, systemd service

### Mobile (Future)
- **Strategy**: Separate React Native app
- **Mode**: Client Node only (no hosting)
- **Platforms**: iOS, Android
- **Shared**: Design language, API contracts

---

## Network Discovery Protocol

Citinet uses **mDNS (Multicast DNS)** and **DNS-SD (DNS Service Discovery)** for zero-configuration networking on local networks.

### Service Type
```
_citinet._tcp.local.
```

### Broadcast Metadata (Hub Nodes)
```json
{
  "version": "0.1.0",
  "node_type": "hub",
  "services": "identity,storage,social,messaging"
}
```

### Discovery Process
1. Hub broadcasts via mDNS every few seconds
2. Clients listen for `_citinet._tcp.local.` services
3. When discovered, clients receive:
   - IP addresses
   - Port number
   - Service metadata
   - Hostname
4. Client stores in `discovered_nodes` map
5. UI displays available hubs

---

## Security Considerations

### Current (Phase 1)
- Local network only (mDNS doesn't traverse routers)
- No authentication yet
- No encryption yet
- Relies on trusted local network

### Future (Phase 2+)
- Mutual TLS for node communication
- Ed25519 key pairs for node identity
- WireGuard tunnels for internet federation
- OIDC for user authentication
- Capability-based access control
- Encrypted storage at rest

---

## Configuration

### Node Configuration (Frontend - Zustand)
```typescript
{
  nodeType: 'hub' | 'client' | 'personal',
  nodeName: string,
  installPath: string,
  contribution: {
    diskSpaceGB: number,    // 5-500 GB
    bandwidthMbps: number,  // 1-100 Mbps
    cpuPercent: number      // 5-50 %
  }
}
```

### Backend Configuration (Future)
```toml
[node]
type = "hub"
name = "Downtown Library Hub"
port = 9090

[resources]
max_disk_gb = 100
max_bandwidth_mbps = 20
max_cpu_percent = 30

[services]
enabled = ["identity", "storage", "social"]
```

---

## API Reference

### Frontend → Backend Commands

**System Monitoring**
```typescript
CitinetAPI.getSystemMetrics(): Promise<SystemMetrics>
CitinetAPI.getHardwareInfo(): Promise<HardwareInfo>
```

**Hub Services**
```typescript
CitinetAPI.startHubBroadcasting(name: string, services: string[]): Promise<void>
CitinetAPI.stopHubBroadcasting(): Promise<void>
CitinetAPI.getHubServiceInfo(): Promise<HubServiceInfo>
```

**Node Discovery**
```typescript
CitinetAPI.startNodeDiscovery(): Promise<void>
CitinetAPI.getDiscoveredNodes(): Promise<HubNode[]>
```

---

## Development Workflow

### Running Locally
```bash
# Install dependencies
npm install

# Run in development mode (hot reload)
npm run tauri dev

# Build for production
npm run tauri build
```

### Adding New Tauri Commands

1. **Define command in Rust** (`src-tauri/src/lib.rs`)
```rust
#[tauri::command]
fn my_command(state: State<AppState>) -> Result<String, String> {
    Ok("Hello".to_string())
}
```

2. **Register command**
```rust
.invoke_handler(tauri::generate_handler![my_command])
```

3. **Add TypeScript binding** (`src/api/tauri.ts`)
```typescript
static async myCommand(): Promise<string> {
    return await invoke<string>("my_command");
}
```

4. **Use in frontend**
```typescript
const result = await CitinetAPI.myCommand();
```

---

## Future Architecture Evolution

### Phase 2: Local Services
- SQLite database for persistent storage
- Local web server for Hub services
- File sharing between nodes
- Sync engine for Personal Nodes

### Phase 3: Federation
- WireGuard mesh networking
- ActivityPub for social networking
- Matrix for messaging
- Distributed identity (DIDs)

### Phase 4: Global Mesh
- Internet-wide node discovery (DHT)
- Content addressing (IPFS-like)
- Censorship resistance
- Offline-first capabilities

---

## Raspberry Pi Deployment

### Detection
Backend automatically detects Raspberry Pi:
```rust
cfg!(target_arch = "aarch64") || cfg!(target_arch = "arm")
```

### Optimizations for Pi
- Lightweight Hub mode (no heavy compute)
- Services: Identity provider, gateway, mDNS broadcaster
- Offload storage/compute to connected workstation
- Low power consumption

### Installation (Future)
```bash
# Download Citinet ARM binary
curl -L citinet.cloud/pi -o citinet-arm64

# Install as systemd service
sudo cp citinet-arm64 /usr/local/bin/citinet
sudo systemctl enable citinet
sudo systemctl start citinet
```

---

## Contributing

When adding features:
1. Update both Rust backend and TypeScript frontend
2. Add Tauri commands for new capabilities
3. Update this ARCHITECTURE.md
4. Test on Windows, macOS, Linux
5. Consider Raspberry Pi implications

---

## Questions & Decisions

### Why mDNS instead of DHT?
- **Phase 1**: Local-first, simple, zero-config
- **Phase 3**: Add DHT for internet-wide discovery

### Why separate mobile app?
- Tauri doesn't support iOS/Android yet
- React Native shares design, different runtime
- Mobile = Client Node only (no hosting)

### Why Rust backend?
- Performance for network operations
- System-level access (sysinfo)
- Native binary, no runtime dependencies
- Cross-platform compilation

---

**Last Updated**: Web-first pivot — February 2026
