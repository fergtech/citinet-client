# ADR-000: Repo Survey & Architecture Foundation

**Status:** Accepted
**Date:** 2026-02-10
**Author:** Citinet Team

## Context

Citinet is a Tauri 2 + React 19 desktop application for building community-owned mesh networks. Before adding new features (auth, database, file sync, discussions, Hub mode), we need to document the current architecture and establish a feature-flag system that all subsequent milestones can build on.

## Current Folder Layout

```
citinet/
├── src/
│   ├── api/
│   │   └── tauri.ts              # Tauri IPC bindings (8 commands)
│   ├── components/
│   │   ├── dashboard/            # Main app views (9 components)
│   │   ├── onboarding/           # Profile setup flow (8 components)
│   │   ├── ui/                   # Reusable primitives (Button, Card, Toggle, ProgressBar)
│   │   └── wizard/               # Install wizard (7 components)
│   ├── stores/                   # Zustand state management
│   │   ├── appStore.ts           # Phase routing + theme
│   │   ├── configStore.ts        # Node config + NodeType
│   │   ├── onboardingStore.ts    # Onboarding step state
│   │   └── wizardStore.ts        # Wizard step state
│   ├── lib/                      # Shared utilities (new)
│   │   └── features.ts           # Feature-flag system (new)
│   ├── App.tsx                   # Root: phase-based routing
│   ├── main.tsx                  # React entry point
│   └── main.css                  # Tailwind 4 theme + CSS variables
├── src-tauri/                    # Rust backend
│   ├── src/                      # Tauri commands, system monitor, mDNS
│   └── tauri.conf.json           # App config (v0.1.0)
├── docs/adr/                     # Architecture decision records (new)
├── package.json
├── tsconfig.json
├── vite.config.ts
├── ARCHITECTURE.md
└── README.md
```

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Frontend framework | React | 19 |
| Build tool | Vite | 7 |
| CSS framework | Tailwind CSS | 4 |
| State management | Zustand | 5 |
| Desktop runtime | Tauri | 2 |
| Backend language | Rust | stable |
| Icons | Lucide React | 0.563 |
| Routing | react-router-dom | 7 (imported, phase-based) |

## UI Routing

The app uses **phase-based routing** via `appStore.phase`:

```
wizard → onboarding → dashboard
```

- `wizard`: Install wizard (license, location, service selection, progress)
- `onboarding`: Profile creation (name, identity keys, network check, community)
- `dashboard`: Main app (metrics, files, community, settings, help)

Phase is persisted in `localStorage` under `citinet-phase`.

## State Management

Four Zustand stores manage all client state:

| Store | Key | Persistence |
|-------|-----|-------------|
| `appStore` | phase, theme | localStorage (manual) |
| `configStore` | nodeType, nodeName, contribution, installPath | localStorage (zustand/persist) |
| `wizardStore` | step progress, license acceptance | In-memory only |
| `onboardingStore` | step progress, display name, key state | In-memory only |

## Existing Storage Layers

- **Active:** localStorage only (config + phase)
- **Declared:** `rusqlite` in Cargo.toml (not yet used)
- **No** server-side database, no file persistence, no encrypted storage

## Existing Tauri Commands (8)

| Command | Purpose |
|---------|---------|
| `get_system_metrics` | CPU, memory, disk, network, uptime |
| `get_hardware_info` | Hostname, OS, CPU count, memory, disk, RPi detection |
| `start_hub_broadcasting` | Begin mDNS service advertisement |
| `stop_hub_broadcasting` | Stop mDNS advertisement |
| `start_node_discovery` | Begin scanning for mDNS services |
| `get_discovered_nodes` | Return list of discovered hub nodes |
| `get_hub_service_info` | Get local hub service details |
| `greet` | Test/hello command |

## Existing Mocks

Dashboard components contain inline mock data:
- `StatusCard`: Hardcoded "Online" + "2h 34m" uptime
- `ContributionCard`: Mock contribution stats
- `ImpactCard`: Mock impact metrics
- `FilesPanel`: Mock file list
- `CommunityPanel`: Mock community posts
- `HelpPanel`: Static help content

`MetricsPanel` and `SettingsPanel` use real Tauri API data.

## Gaps for Windows MVP

| Gap | Milestone |
|-----|-----------|
| Feature flags / profile-based UI | **M1 (this)** |
| Encrypted SQLite database | M2 |
| Identity & key management | M3 |
| Authentication & node pairing | M4 |
| File persistence & sync | M5 |
| Discussion posts & threads | M6 |
| Hub mode (cloudflared, Docker) | M7 |
| Discover tab (curated resources) | M8 |
| Windows installer (WiX) | M9 |
| Diagnostics export | M10 |

## Decision

**Adopt profile-based feature flags using Zustand + Tauri config.**

### Rationale

1. **NodeType already exists** in `configStore.ts` — reuse it as the profile key
2. **Zustand is already the state layer** — no new dependencies for flag resolution
3. **Compile-time safety** — TypeScript union types ensure flags are exhaustive
4. **Incremental rollout** — each milestone enables its flags; disabled flags hide UI gracefully
5. **No server dependency** — flags resolve locally from the profile, suitable for offline-first mesh nodes

### Implementation

- `src/lib/features.ts` defines `FeatureFlag` type, `PROFILE_FLAGS` mapping, `useFeatureFlags()` hook, `useFeature(flag)` hook, and `<FeatureGate>` component
- Sidebar and Dashboard conditionally render tabs using `<FeatureGate>`
- SettingsPanel shows active flags in a Diagnostics section

### Consequences

- All new UI features must register a flag in `features.ts`
- Profile changes immediately reflect in the UI (Zustand reactivity)
- No runtime flag override mechanism yet (can be added in M2 with database-backed overrides)
