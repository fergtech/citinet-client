
# Citinet Client (Windows-first MVP)

**Digital infrastructure for hyperlocal communities — owned by neighbors, not corporations.**

---

## Current MVP Status (Phase 1)

This is a **working prototype** focused on core functionality:

### ✅ Fully Functional
- **Installation Wizard** - First-run setup with node type selection (Hub/Client/Personal)
- **Resource Allocation** - Configure storage, bandwidth, and CPU contribution limits
- **System Monitoring** - Real-time CPU, memory, disk, and network metrics
- **File Storage** - Upload, download, delete files with quota enforcement
- **Cloudflare Tunnel** - Configure and manage public HTTPS access to Hub nodes
- **Docker Management** - Monitor and control Docker containers
- **Settings Panel** - Adjust resource limits, themes, and view hardware info

### 🚧 Coming Later
- Network discovery (mDNS infrastructure ready)
- Community features and social networking
- Peer-to-peer file sharing
- Marketplace and local services

---

## What is Citinet?
Citinet (Citizens' Inter‑networking) is a local‑first, community‑owned network made of **independently operated nodes** that interconnect using open standards. Each node is run by a household, a small business, a library, a school, or a neighborhood group. Together they form **a network of citizen‑owned networks**.

Citinet does **not** replace the physical Internet. It replaces the *centralized platform model* that currently dominates it by giving communities the tooling to run their **own cloud**: identity, storage, feeds, messaging, marketplace, and discovery — all locally controlled and privacy‑respecting.

---

## Current MVP Architecture (2026)
This repository contains the **Citinet Client** — a Tauri + React desktop application that acts as the universal installer, dashboard, and sync engine for Citinet.

- **Windows‑first** (Windows 10/11). Linux/macOS and mobile will follow.
- **One installer → three roles:**
  - **Hub** – community micro–data center (runs services & DB in containers).
  - **Client** – participant device (lightweight, optional contribution of storage/compute).
  - **Personal** – sovereign device (user’s primary store, with optional sync to a home node).
- **Ingress**: Hubs use **Cloudflare Tunnel** (managed by the client) to expose `https://{node}.citinet.io` securely — no router changes required.
- **Dashboard**: a universal UX surface for Files, Discussions, Marketplace, Social, Events, Resources, and Settings. For MVP, **Files** and **Discussions** are functional; others may deep‑link to web routes until their native modules land.

> Earlier drafts assumed a Raspberry Pi gateway plus a workstation as the baseline. That hardware is still compatible and valuable, but the **MVP path centers on a single Windows machine** running the client in **Hub mode**. Pi and dedicated gateways are now **optional advanced deployments**, not requirements for first‑time hubs.

---

## Node Types (Citinet Roles)

### 1) Hub Node (Community Micro–Data Center)
Runs the community’s services. In MVP it’s typically **one Windows machine** with Docker containers for the Citinet backend (reverse proxy, API, Postgres, Redis, media storage). Public access is provided via **Cloudflare Tunnel** to `https://{slug}.citinet.io`.

### 2) Client Node (Participant Device)
A laptop/desktop that connects to a hub to use services. The client keeps an **encrypted local cache** for offline use and may **contribute** a capped slice of storage/CPU when idle & on AC power (opt‑in).

### 3) Personal Node (Optional Sovereign Device)
A device that treats **local data as the primary source of truth**. The client stores encrypted data locally and **syncs** changes to the user’s home hub when online.

---

## What the Client Does

### Core MVP Features (Working Now)

- **Role-aware installation wizard** - Choose Hub/Client/Personal with capability checks
- **Real file storage system** - Upload/download files with quota enforcement and progress tracking
- **Resource contribution management** - Set and enforce disk, bandwidth, and CPU limits
- **Cloudflare Tunnel integration** - Expose Hub nodes via HTTPS without port forwarding
- **Docker container management** - Start/stop/monitor containers for Hub services
- **Live system monitoring** - Real-time metrics for CPU, memory, disk, network
- **Secure local database** - SQLite for node config, settings, and file metadata
- **Theme support** - Light/dark/system themes with persistent preferences

### Infrastructure Ready (Backend Implemented)

- **mDNS service discovery** - Hub broadcasting and Client discovery (UI pending)
- **Secure storage** - Encrypted SQLite for configuration and credentials
- **Windows integration** - DPAPI for credential storage, auto-start support

### Planned Features

- **Pairing & auth** - Connect clients to hubs with OAuth-style flow
- **P2P file sharing** - Direct transfer between nodes when on same network  
- **Community discussions** - Local forums and messaging
- **Updates & diagnostics** - Auto-update with signed releases

---

## Citinet Domains & Separation of Concerns
- `https://citinet.io` – Public information & docs (marketing site). Not a hub.
- `https://start.citinet.io` – Onboarding wizard (join a node or create one). Not a hub.
- `https://{node}.citinet.io` – **The actual Citinet experience** for a community (sign‑in, dashboard, files, feeds, marketplace, etc.).

The client routes users to node sub‑spaces as paths, e.g. `https://merryweather.citinet.io/cradleway-library`. Public user profiles live at `/{@handle}` (e.g., `/@sarah`). Private dashboards are accessed at `/dash` after authentication.

---

## Getting Started (Developers)

### Prerequisites
- Node.js 18+
- Rust (for Tauri)
- Windows 10/11 (for Windows‑first dev & packaging)

### Install dependencies
```bash
npm install
```

### Run the web dev server
```bash
npm run dev
```

### Run the desktop app (Tauri)
```bash
npm run tauri dev
```

### Build for production
```bash
npm run build        # Frontend only
npm run tauri build  # Full desktop app
```

### Troubleshooting
If you see “localhost refused to connect”:
1) Check terminal logs after Vite reports readiness.
2) Try `http://127.0.0.1:1420/`.
3) Verify the actual port (Vite may have picked a free port).
4) Clear cache: delete `node_modules/.vite` and retry.
5) Ensure Windows Firewall is not blocking Node.js.
6) On corporate devices, check antivirus/endpoint controls.

---

## Hub Mode (Windows) — How it Works

1. **Choose Hub** in the first‑run wizard.
2. **Pre‑flight checks**: Windows version, admin rights for services, ports 80/443 availability, free disk path for data, Docker Desktop presence (we prompt with a link; we do not silently install).
3. **Provision**: Enter the **provision token** from `start.citinet.io`. The client registers the node, writes `cloudflared` config, and sets up the local reverse proxy.
4. **Launch services**: Start Docker containers (reverse proxy, API, DB, cache) and bring the node online.
5. **Tunnel**: `cloudflared` runs as a Windows service; the node is reachable at `https://{slug}.citinet.io` via Cloudflare Tunnel.
6. **Health**: The Admin panel shows green checks for Tunnel, Proxy, API, DB, and Storage.

### Example: `cloudflared` config (Windows)
```yaml
# C:\\ProgramData\\Citinet\\cloudflared\\config.yml

# unique tunnel id for this node
tunnel: slug-node-uuid
credentials-file: C:\\ProgramData\\Citinet\\cloudflared\\slug-node-uuid.json

ingress:
  - hostname: slug.citinet.io
    service: http://localhost:8080
  - service: http_status:404
```

> Security: the local reverse proxy binds to **localhost** only; the tunnel is the **only** public ingress. Rotate tunnel credentials from the Admin panel as needed.

---

## Profiles & Feature Matrix (MVP)

### Hub
- Docker‑based backend (reverse proxy, API, Postgres, Redis)
- Cloudflare Tunnel service management
- Health checks, backups (roadmap), admin panel

### Client
- Encrypted local cache (SQLite)
- Pairing & auth to any node
- Optional contribution of storage/compute (capped, idle‑only)

### Personal
- Local‑first primary data store (encrypted)
- Optional sync to a home node
- Same dashboard UI; sovereignty‑first defaults

---

## Security & Privacy
- **Secrets** stored with **Windows Credential Manager/DPAPI**.
- **Local DB** is encrypted; WAL mode; safe migrations.
- **HTTPS required**; reject clear‑text except during local dev.
- **No arbitrary code execution** from nodes; background tasks are signed and sandboxed.
- **Telemetry is opt‑in**, minimal, and anonymous. Easy to disable.

---

## Roadmap (High‑level)
1) **Windows MVP (this repo)**: profiles, secure storage, pairing/auth, Files + Discussions, Tunnel, installer, auto‑update, diagnostics.
2) **Linux/macOS** packaging; abstract OS‑specific keychain and services.
3) **Mobile** clients (Tauri Mobile / Capacitor), background sync.
4) **Real Node backend** for `https://{node}.citinet.io` (registry, API, DB schema, media store).
5) **Federation** for public content; **Matrix** (optional) for E2EE DMs.
6) **Resource Units** (add machines to a node cluster); MinIO for media; DB replicas.
7) **Extensions (Labs)**: minimal, permissioned extension API; local catalog; later a federated directory.

---

## Technology Stack

### Frontend
- React + TypeScript
- Vite
- Tailwind CSS
- Zustand (or Redux/RTK — see repo)
- Lucide React (icons)

### Desktop Shell
- Tauri 2
- Rust (Tauri commands, background tasks, OS integration)
- `rusqlite` (encrypted local DB)
- `sysinfo` (resource metrics), `tokio` (async)

---

## Contributing
PRs welcome! Before large changes, open an issue to discuss scope and design.

Please see:
- `docs/ARCHITECTURE.md` — profiles, storage, auth, tunnel, reverse proxy, feature flags
- `docs/INSTALLER.md` — system requirements, first‑run wizard, Hub prerequisites
- `docs/SECURITY.md` — secret storage, encryption, permissions, telemetry

---

## License
See `ATTRIBUTIONS.md` for third‑party notices.

Citizens' Digital Infrastructure Project — Citinet
