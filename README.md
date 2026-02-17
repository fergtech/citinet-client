
# Citinet

**People-powered cloud — digital infrastructure owned by neighbors, not corporations.**

Citinet (Citizens' Inter-networking) lets a community run its own cloud. A hub admin installs this desktop app, allocates storage, and optionally connects a Cloudflare Tunnel so members can access their hub from anywhere — similar to how Jellyfin + Cloudflare Tunnel works, but for community file storage and services.

---

## How It Works

```
Hub Admin installs Citinet desktop app
  → 10-step wizard configures the node
  → Embedded HTTP API starts on port 9090
  → (Optional) Cloudflare Tunnel exposes the hub publicly
  → Community members access via web app or desktop login
```

The desktop app is both the **admin dashboard** and the **backend server**. It runs an embedded axum HTTP API that serves files and handles authentication. When a Cloudflare Tunnel is connected, the hub becomes reachable at a public URL (e.g., `https://your-hub.trycloudflare.com` or a custom domain like `hub.citinet.io`).

---

## Features

### Working Now
- **10-step installation wizard** — node naming, install location, storage allocation, admin account creation, Cloudflare Tunnel setup
- **User authentication** — bcrypt password hashing, JWT tokens, login/logout flow
- **File storage** — upload, download, delete with per-user ownership and public/private visibility
- **Cloudflare Tunnel** — two modes: Quick Tunnel (temporary trycloudflare.com URL) and Custom Domain (API-managed, permanent)
- **User management** — admin can list users, promote/demote admins, delete accounts
- **HTTP API** — embedded axum server on port 9090 with REST endpoints for auth, files, and node status
- **System monitoring** — real-time CPU, memory, disk, and network metrics
- **Dashboard** — tabbed UI with Files, Admin (users + tunnel), Settings, and Help panels
- **Theme support** — light, dark, and system-follow themes

### Architecture
- **SQLite database** — stores node config, tunnel config, users, spaces, and file metadata
- **Local file storage** — files stored on disk under the configured install path with quota enforcement
- **Tauri IPC** — 25 commands bridging the React frontend to the Rust backend
- **Shared state** — `Arc<Mutex<T>>` allows both Tauri commands and the axum API server to access the same StorageManager and TunnelManager

---

## Tech Stack

### Frontend
- React 19 + TypeScript
- Vite 7
- Tailwind CSS 4
- Zustand 5 (state management)
- Lucide React (icons)

### Backend (Rust)
- Tauri 2 (desktop shell + IPC)
- axum 0.8 (embedded HTTP API server)
- rusqlite (SQLite database)
- bcrypt + jsonwebtoken (authentication)
- sysinfo (system metrics)
- tokio (async runtime)
- tower-http (CORS)

---

## HTTP API Endpoints

The embedded axum server runs on `0.0.0.0:9090` and exposes:

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/health` | No | Health check |
| GET | `/api/info` | No | Node ID, name, type, storage quota |
| GET | `/api/status` | No | Uptime, storage usage, online status |
| POST | `/api/auth/register` | No | Create a new user account |
| POST | `/api/auth/login` | No | Authenticate and receive JWT |
| GET | `/api/files` | JWT | List files visible to the authenticated user |
| POST | `/api/files` | JWT | Upload a file (multipart/form-data) |
| GET | `/api/files/{name}` | JWT | Download a file |
| DELETE | `/api/files/{name}` | JWT | Delete a file |

---

## Project Structure

```
src/                          # React frontend
├── api/tauri.ts              # Tauri IPC wrapper (CitinetAPI class)
├── stores/
│   ├── appStore.ts           # Phase routing (wizard/login/dashboard) + theme
│   ├── authStore.ts          # Current user + localStorage persistence
│   ├── configStore.ts        # Node configuration (zustand/persist)
│   └── wizardStore.ts        # Wizard flow state
├── components/
│   ├── LoginScreen.tsx       # Login form
│   ├── wizard/               # 10-step installation wizard
│   │   ├── Wizard.tsx        # Step router
│   │   ├── WizardLayout.tsx  # Shared layout with progress indicator
│   │   ├── WelcomeStep.tsx
│   │   ├── LicenseStep.tsx
│   │   ├── NodeIdentityStep.tsx
│   │   ├── LocationStep.tsx
│   │   ├── ContributionSlider.tsx
│   │   ├── ServiceStep.tsx
│   │   ├── AdminAccountStep.tsx
│   │   ├── ProgressStep.tsx  # Runs backend initialization
│   │   ├── TunnelStep.tsx    # Cloudflare tunnel setup
│   │   └── CompleteStep.tsx  # Summary + auto-login
│   ├── dashboard/
│   │   ├── Dashboard.tsx     # Tab container
│   │   ├── Sidebar.tsx       # Navigation + user info + logout
│   │   ├── FilesPanel.tsx    # File upload/download/delete
│   │   ├── AdminPanel.tsx    # User management + tunnel control
│   │   └── SettingsPanel.tsx # Resource limits + theme
│   └── ui/                   # Reusable UI components
│       ├── Button.tsx
│       ├── ProgressBar.tsx
│       └── Toggle.tsx
src-tauri/src/                # Rust backend
├── lib.rs                    # Tauri app setup, 25 IPC commands, AppState
├── storage_manager.rs        # SQLite DB, file I/O, user CRUD
├── auth.rs                   # bcrypt hashing, JWT generation/validation
├── hub_api.rs                # axum HTTP server (port 9090)
├── tunnel_manager.rs         # Cloudflare tunnel orchestration
└── system_monitor.rs         # CPU, memory, disk, network metrics
```

---

## Getting Started

### Prerequisites
- Node.js 18+
- Rust 1.77+
- Windows 10/11

### Development
```bash
npm install
npm run tauri dev
```

This starts both the Vite dev server (port 1420) and the Tauri desktop app. The app will not work if launched directly from the debug binary — it needs the Vite dev server running.

### Production Build
```bash
npm run tauri build
```

Produces a standalone `.exe` installer in `src-tauri/target/release/bundle/`.

### Tests
```bash
npm run test          # Run once
npm run test:watch    # Watch mode
```

---

## App Flow

1. **First launch** — the wizard runs: name your node, pick install location, set storage quota, create admin account, optionally configure Cloudflare Tunnel
2. **Installation** — ProgressStep creates directories, initializes SQLite, saves config, creates the admin user
3. **Auto-login** — after the wizard, the admin is automatically logged in and lands on the dashboard
4. **Subsequent launches** — the app auto-loads the previously configured node and shows the login screen
5. **Dashboard** — Files panel for managing stored files, Admin panel for user and tunnel management, Settings for resource limits and theme
6. **Logout** — clears auth state and returns to the login screen

---

## License
Citizens' Digital Infrastructure Project — Citinet
