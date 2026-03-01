# Citinet Infrastructure Overview

This document describes the full infrastructure landscape for the Citinet project — how each component is hosted, how they connect, and what role this repo plays.

---

## Components

### 1. Info Site (`fergtech/citinet-info`)
- **Framework:** Astro 5 (static output) + React components + Tailwind CSS v4
- **Hosting:** Vercel (auto-deploys on `git push`)
- **Domains:** `citinet.cloud`, `www.citinet.cloud`
- **Purpose:** Public-facing landing/marketing site.

### 2. Web Portal (`fergtech/citinet`)
- **Framework:** React 18 + Vite SPA + PWA
- **Hosting:** Vercel (primary) + CF Worker fallback for `*.citinet.cloud` wildcard
- **Domains:**
  - `start.citinet.cloud` → Vercel — onboarding (join/create hub)
  - `*.citinet.cloud` → CF Worker — hub interface (e.g. `riverdale.citinet.cloud`)
- **Purpose:** Browser-based client for all hub interactions.

### 3. Hub Registry (`fergtech/citinet-registry`)
- **Runtime:** Cloudflare Worker + Workers KV
- **Domain:** `registry.citinet.cloud`
- **Deploy:** `npx wrangler deploy` (manual, requires CF API token)
- **Purpose:** Central directory of active hubs — stores slug, name, tunnel URL, online status.

### 4. Hub Management App — this repo (`fergtech/citinet-client`)
- **Framework:** Tauri 2 (Rust backend) + React 19 (frontend) + Tailwind CSS
- **Platform:** Windows (.msi installer, WiX toolset)
- **Distribution:** GitHub Releases
- **Auto-update endpoint:** `https://github.com/fergtech/citinet-client/releases/latest/download/update.json`
- **Signing:** minisign keypair — `TAURI_SIGNING_PRIVATE_KEY` env var required at build time (CRLF-stripped)
- **Purpose:** Hub operator management tool. Manages Docker containers (the hub stack), configures public access (Tailscale Funnel recommended, Cloudflare Tunnel, or custom gateway), exposes a local API on port 9090, and auto-registers the hub with the registry. Community members access hubs through the web portal — this app is for operators only.
- **Coming soon:** Simplified one-click launcher for non-technical operators.

---

## Desktop Client Architecture

### Backend (Rust / Tauri)

| Module | File | Purpose |
|--------|------|---------|
| Storage | `src-tauri/src/storage_manager.rs` | SQLite + dir management, node config CRUD |
| Docker | `src-tauri/src/docker_manager.rs` | Stateless Docker CLI wrapper |
| Tunnel | `src-tauri/src/tunnel_manager.rs` | Cloudflare quick/named tunnel orchestration |
| Tailscale | `src-tauri/src/tailscale_manager.rs` | Tailscale install, login, and Funnel management |
| Hub API | `src-tauri/src/hub_api/` | Local HTTP API server on port 9090 |
| Commands | `src-tauri/src/lib.rs` | 29 Tauri IPC commands exposed to frontend |

### Frontend (React)

| Store | Purpose |
|-------|---------|
| `src/stores/configStore.ts` | Node type + config (Zustand + persist) |
| `src/stores/appStore.ts` | Phase routing + theme (manual localStorage) |
| `src/lib/features.ts` | Feature flag system (profile-based per node type) |
| `src/api/tauri.ts` | Typed wrappers for all 29 Tauri IPC commands |

### Phase Routing
```
wizard → onboarding → dashboard
```
- **wizard**: initial setup (node type selection, config)
- **onboarding**: Docker/tunnel setup
- **dashboard**: operational hub management

---

## Hub Tunnel Flow

```
Hub operator configures public access in dashboard:

  Option A — Tailscale Funnel (recommended):
    → Tailscale installed + logged in
    → tailscale funnel --bg 9090
    → returns stable https://name.tailXXXX.ts.net (IPv4 + IPv6)
    → URL auto-registered with registry.citinet.cloud

  Option B — Cloudflare Quick Tunnel:
    → cloudflared spawns quick tunnel → returns something.trycloudflare.com
    → URL auto-registered with registry.citinet.cloud
    → Note: trycloudflare.com is IPv6-only; use Tailscale or custom domain for IPv4 support

  Option C — Custom Domain (Cloudflare):
    → API-managed tunnel → permanent {name}.citinet.cloud URL
    → URL auto-registered with registry.citinet.cloud

All options:
    → POST /hubs { id, name, slug, tunnel_url, online: true }
       Authorization: Bearer <REGISTRY_SECRET baked at compile time>

User visits start.citinet.cloud in browser
    → fetches registry, sees hub listed
    → joins hub → navigates to slug.citinet.cloud
    → web portal fetches registry to confirm current tunnel URL
    → connects to hub API
```

---

## DNS Configuration (`citinet.cloud` — Cloudflare nameservers)

| Record | Type | Target | CF Proxy | Purpose |
|--------|------|--------|----------|---------|
| `citinet.cloud` | CNAME | `cname.vercel-dns.com` | OFF | Info site |
| `www` | CNAME | `cname.vercel-dns.com` | OFF | Info site www |
| `start` | CNAME | `cname.vercel-dns.com` | OFF | Web portal onboarding |
| `*` | CNAME | `citinet-web.tdyfrvr.workers.dev` | ON | Hub subdomains (CF Worker) |
| `registry` | Worker route | `citinet-registry` Worker | ON | Hub registry API |

---

## Build & Release Process

### Development build
```powershell
npm run tauri dev
```

### Production build (requires signing key)
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content C:\Users\tdyfr\.tauri\citinet.key -Raw).Replace("`r`n", "`n").TrimEnd()
$env:REGISTRY_SECRET = "<secret>"
npm run tauri build
```
Artifacts in `src-tauri/target/release/bundle/msi/`:
- `citinet_x.x.x_x64_en-US.msi` — installer
- `citinet_x.x.x_x64_en-US.msi.zip` — updater bundle (Tauri-produced, do not re-zip manually)
- `citinet_x.x.x_x64_en-US.msi.zip.sig` — minisign signature

### GitHub Release
1. Tag: `vx.x.x`
2. Upload: `.msi`, `.msi.zip`, `.msi.zip.sig`, `update.json`
3. `update.json` must reference the `.msi.zip` URL and contain the `.sig` content

---

## Known Limitations

- **Quick Tunnel IPv6-only:** `trycloudflare.com` quick tunnels have no IPv4 A records. Use Tailscale Funnel (recommended) or a custom Cloudflare domain for full IPv4+IPv6 support.
- **REGISTRY_SECRET baked at compile time:** changing the secret requires a new build and release.
- **Wildcard `*.citinet.cloud` stays on CF Workers:** Vercel Free does not support wildcard custom domains.
- **Registry is CF-only:** `registry.citinet.cloud` cannot move off Cloudflare without migrating the KV store.
- **Registrar transfer lock:** citinet.cloud subject to 60-day ICANN lock from registration (~April 2026).
- **Windows-only:** Desktop app currently targets Windows only. Hub stack itself (Docker Compose) runs on any OS.

---

## Deployment Summary

| Component | How to deploy |
|-----------|---------------|
| Info site | `git push` → Vercel auto-deploys |
| Web portal | `git push` → Vercel + CF Workers auto-deploy |
| Registry | `npx wrangler deploy` (manual) |
| Desktop client (this repo) | `npm run tauri build` with env vars set, upload artifacts to GitHub release |
