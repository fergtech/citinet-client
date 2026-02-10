# Citizens Inter-networking (Citinet)

### Local Mesh Node Application

**Digital infrastructure for hyperlocal communities. Owned by neighbors, not corporations.**

---

## What Is Citinet?

Citizens Inter-networking, codename Citi-Net or Citinet, is a community-driven, independent network system composed of many small, locally operated "micro-data centers." These nodes communicate using open, standard Internet protocols but operate outside the control of traditional monopolies, big corporations, and large government agencies.

Citinet provides alternative layers, platforms, and digital resources that ordinary people can run, own, and use — without surveillance, centralized control, or extractive business models.

Citinet is not a replacement for the physical Internet — it is a replacement for the centralized platforms that currently dominate it. It transforms houses into micro data centers, neighborhoods into interconnected digital communities, and citizens into owners of their digital world.

**A network of citizen-owned networks.**

---

## Purpose

To return digital power, data sovereignty, communication tools, and online community spaces back to citizens. From careful research and practical development, this will be most effectively achieved at the local level.

Today's Internet is dominated by large corporations, government agencies, massive cloud platforms, exploitative algorithms, and centralized data harvesting systems. Citinet provides an alternative — not by reinventing the entire Internet from scratch, but by building a new layer on top of it: **a layer owned and operated by the people who use it.**

---

## This Application's Role

This Local Mesh Node Application is the desktop client that allows users to dedicate portions of their personal device's storage and networking resources to the Citinet infrastructure. It provides:

- **Familiar, polished UX** borrowed from apps billions already use
- **Local ownership** and transparent governance instead of corporate control
- **Privacy by default** — pseudonyms welcome, no tracking, local-only visibility
- **The software foundation** for physical mesh network infrastructure

The web portal at citinet.io houses the full social experience — discussions, marketplace, community spaces, and more. This desktop client is the node management and resource contribution layer.

We're not trying to reinvent how people interact with apps. We're reinventing **what those apps are for** and **who controls them.**

---

## Node Types

Citinet supports three distinct modes of operation:

### Hub Node (Community Micro-Data Center)
- **Purpose**: Host services for your local community
- **Hardware**: Raspberry Pi + workstation, or dedicated PC
- **Location**: Home, library, community center, school
- **Features**: mDNS broadcasting, service hosting, resource management
- **For**: Community organizers, libraries, makerspaces

### Client Node (Participant Device)
- **Purpose**: Connect to nearby hubs and use services
- **Hardware**: Phone, laptop, desktop computer
- **Location**: Anywhere within network range
- **Features**: Zero-config discovery, minimal resource usage
- **For**: Most users—students, residents, community members

### Personal Node (Optional Sovereign Device)
- **Purpose**: Your primary data store that syncs to hubs
- **Hardware**: Desktop, laptop, or dedicated device
- **Location**: Your home or office
- **Features**: Local-first storage, offline-capable, full data sovereignty
- **For**: Users who want complete control over their data

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

---

## The Citinet Ecosystem

The complete Citinet ecosystem consists of:

| Component | Description |
|-----------|-------------|
| Web Browsers | Open-source or trusted browsers |
| Search | Independent search options |
| Social Networking | Community-hosted platforms (citinet.io portal) |
| Messaging | Decentralized, encrypted messaging |
| Email & Identity | Peer-run email and identity systems |
| Marketplace | Community exchange and resource sharing (citinet.io portal) |
| Storage | Local-first data storage and file sharing |
| Citinet OS | Custom operating system and dashboard |
| Desktop Client | Node management and resource contribution (this application) |

---

## Technical Philosophy

Citinet operates on the principle of inter-networking — exactly what the original Internet was meant to be:

> *"A network of networks, freely connecting independent systems using shared protocols."*

Each Citinet node is independently owned, independently configured, and independently governed. All nodes communicate with each other as peers.

---

## Physical Infrastructure

### Traditional Corporate/Government Data Centers

Massive, centralized facilities typically include hundreds or thousands of enterprise-grade servers, multi-room racks of storage, dozens of firewalls and networking appliances, redundant power systems, and industrial cooling — all at multi-million-dollar annual operating costs. These structures are effective for scale, but they centralize data, power, control, surveillance, and economic dependence.

### Citinet Micro-Data Point Architecture

A fully functional Citinet node — enough to serve a household, library, community center, dorm, or small neighborhood — requires only:

| Component | Purpose |
|-----------|---------|
| 5- or 8-port network switch | Local LAN backbone |
| Raspberry Pi | Gateway, identity provider, security layer, local services |
| PC tower / workstation | Primary compute + storage |
| Ethernet cables | Wired connections |
| Citinet OS | Operating system (coming soon) |
| Wi-Fi extender (optional) | Wireless local topology |

This setup provides local cloud features, social networking, messaging, storage, small-scale hosting, community identity, dashboards, admin tools, and future mesh capabilities — all at a fraction of the cost of traditional systems.

### Node Locations

Citinet is designed for citizens' homes, apartment bedrooms, student dorms, public libraries, community centers, coworking spaces, small businesses, makerspaces, and neighborhood hubs. Each location becomes a micro data point participating in the greater Citinet mesh.

---

## Software Layer

### Traditional Corporate Software

Platforms like AWS, Google Cloud, Azure, Microsoft 365, Gmail, and Meta offer powerful tools, but at the cost of surveillance, data extraction, centralized control, dependency, algorithmic manipulation, and censorship risks.

### Citinet Software Layer

Citinet replaces centralized accounts, social media, storage, identity, messaging, local marketplaces, and discovery with:

- Open-source or trusted web apps
- Community-hosted services
- Peer-to-peer nodes
- Encrypted messaging
- Local-first social feeds
- Decentralized storage
- Transparent governance
- Citizen-operated "local clouds"

Compatible platforms include Society+, Mastodon/ActivityPub, PeerTube, ProtonMail, Nextcloud, and Matrix. Citinet nodes may run native apps or integrate existing open-source services.

---

## Citinet Protocol Stack

Here's how a solo setup maps to the full Citinet architecture:

### Layer 1 — Hardware

- **Raspberry Pi** — gateway, identity provider, initial local services
- **PC/workstation** — compute + storage + app hosting
- **Switch** — local LAN backbone

This is a micro-data point matching the minimal hardware footprint envisioned in the whitepaper.

### Layer 2 — Network

The first Citinet node uses the existing LAN (the switch), exposes services via local IP addresses, and optionally exposes a secure endpoint over the internet (WireGuard tunnel) for remote federation later. Clients connect via Wi-Fi or Ethernet to the switch, through the Pi, and into Citinet services.

### Layer 3 — Secure Transport & Overlay

Optional at launch, but even on Day 1 you can run WireGuard on the Pi for encrypted tunnels or Yggdrasil for a peer-to-peer overlay without complex networking configuration. Setting this up early means future nodes can join seamlessly.

### Layer 4 — Federation & Discovery

At the hyperlocal stage, only local discovery is needed: mDNS (Avahi) so Citinet clients auto-discover the Pi, and DNS-SD so services appear automatically. No federation is required until node #2 comes online, but the Pi can publish basic federation endpoints (ActivityPub, Matrix, OIDC) so it's ready when the time comes.

### Layer 5 — Identity & Trust

The first and most critical piece — everything else depends on it. The Pi serves as the identity server using something like Keycloak (OIDC) or a lightweight DID method. Community accounts are issued to early users. When another node comes online, identities federate between nodes, giving each community self-governance while following shared Citinet identity rules.

### Layer 6 — Service & Data

Day 1 apps a single node can run:

- Local social feed (ActivityPub micro-instance)
- Local messaging (Matrix Synapse server)
- Local storage (Nextcloud)
- Local community bulletin (simple web app)
- Local alerts (push notifications or MQTT topic)

Users access everything through the Citinet dashboard. Even a single workstation + Pi creates a full "local cloud."

### Layer 7 — Application

The user-facing layer. The Citinet dashboard becomes the portal, the social layer, the messaging interface, the services entry point, the community awareness hub, and the identity hub. Users don't need to know anything about networks — they just log in and see community, posts, chat, events, and alerts.

This is where mass adoption starts.

---

## MVP Launch Path

### Phase 1 — Founding Node (You + Your Hardware)

Run identity, messaging, social microfeed, local storage, and dashboard UI. Invite a neighbor, a friend, a test group. They install the Citinet client, access the node's dashboard, and start posting, checking events, and messaging. You now have a real micro-community online.

### Phase 2 — Onboarding a Local Partner (e.g., a Library)

The library installs the Citinet client and connects to your node. Library moderators use it for announcements, events, room reservations, local updates, community discussion, file sharing for workshops, and digital literacy programs. The whole library becomes active users — and they invite locals. **This is the moment Citinet becomes real.**

### Phase 3 — A Second Node Goes Live

A high school, community center, neighbor, or local nonprofit installs their own Pi + workstation. Your node federates with theirs. Now data flows between communities following Citinet protocols, with independent governance, identical service protocols, local identity with cross-node trust, and the beginning of a mesh of citizen micro-clouds.

### Why This Works Solo

The MVP depends on open protocols, small-scale services, commodity hardware, local-first architecture, and modular layers. Each part can be built one at a time, and the system still functions. You don't need global federation to launch locally, thousands of nodes to build a micro-community, or perfect UI early on. If your local library finds value, the model works.

---

## Development Roadmap

### Phase 1 — Core Platform & Real Backend (In Progress)

**UI Complete:**
- Two-path onboarding (join existing network or create your own)
- Node discovery and creation wizard
- Civic onboarding flow — orientation, not signup
- Local Commons dashboard
- Chronological feed — no algorithms, no ranking, no manipulation
- Community discussions organized by type (Discussion, Announcement, Project, Request)
- Local marketplace for community exchange and resource sharing
- Network status with active members and node health
- Privacy by default — pseudonyms welcome, no tracking, local-only visibility
- Responsive design — desktop sidebar navigation, mobile bottom bar
- Smart routing with URL-based navigation and browser history support
- Persistent state — page refreshes maintain your current location
- In-app navigation with back buttons on all screens
- Multi-node support with selected node name displayed throughout

**Backend Implemented:**
- ✅ Real system resource monitoring (CPU, memory, disk, network bandwidth)
- ✅ Hardware detection (automatically detects Raspberry Pi)
- ✅ mDNS broadcasting for Hub Nodes
- ✅ mDNS discovery for Client Nodes
- ✅ Node type configuration (Hub, Client, Personal)
- ✅ Resource contribution settings (disk space, bandwidth, CPU limits)
- ✅ Tauri IPC commands connecting frontend to Rust backend

**In Progress:**
- Resource limit enforcement in backend
- Persistent configuration storage
- File storage management for Hub Nodes
- Personal Node sync engine
- Service health monitoring

### Phase 2 — Physical Mesh Nodes (Next)

Physical mesh network nodes that community members can install as access points. Implementation includes:
- Encrypted peer-to-peer connections (WireGuard)
- Federation protocols (ActivityPub, Matrix)
- Identity management (Keycloak/OIDC)
- File storage services (Nextcloud integration)
- Raspberry Pi system image with auto-configuration

### Phase 3 — ISP Sharing & Internet Independence (Future)

Resilient, community-owned, surveillance-free local internet. Users will set up physical nodes in their homes and businesses, creating a mesh network that provides internet access independent of traditional ISPs. Includes bandwidth sharing, mesh WiFi, and community-owned infrastructure.

---

## Design Philosophy

**What we borrow from commercial apps:**
- Clean, modern design language
- Intuitive navigation patterns users already understand
- Mobile-first, responsive layouts
- Smooth animations and polished micro-interactions

**What we reject:**
- Algorithmic feeds designed to maximize engagement
- Surveillance-based business models
- Extractive data practices
- Corporate ownership of community infrastructure

**The result:** An app that feels as natural to use as Instagram or Twitter, but serves your actual neighbors instead of distant shareholders.

---

## Why Citinet Matters

Citinet enables citizen-controlled data, community-hosted spaces, meaningful local identity, human-centered digital experiences, censorship-resistant content, organic network growth, and easy digital infrastructure building.

It returns independence, decentralization, locality, privacy, and public digital ownership — with modern UX, simplicity, and accessibility.

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (for Tauri backend)

### Install Dependencies

```bash
npm install
```

### Run Development Server

```bash
npm run dev
```

The Vite dev server will start at `http://localhost:1420`.

### Run Desktop App (Tauri)

```bash
npm run tauri dev
```

### Build for Production

```bash
npm run build          # Frontend only
npm run tauri build    # Full desktop app
```

### Troubleshooting

If you see "localhost refused to connect":

1. Check your terminal for error messages after "VITE ready"
2. Try `http://localhost:1420/` or `http://127.0.0.1:1420/`
3. Check the terminal for the actual port (it may use a different one if 1420 is busy)
4. Delete `node_modules/.vite` and restart to clear cache
5. Ensure Windows Firewall isn't blocking Node.js
6. Try running your terminal as administrator

---

## Technology Stack

### Frontend
| Technology | Purpose |
|-----------|---------|
| React 19 | UI framework |
| TypeScript | Type safety |
| Vite 7 | Build tool |
| Tailwind CSS 4 | Styling |
| Zustand | State management |
| Lucide React | Icons |

### Backend
| Technology | Purpose |
|-----------|---------|
| Tauri 2 | Desktop app runtime |
| Rust | System-level programming |
| sysinfo | Real-time system metrics (CPU, memory, disk, network) |
| mdns-sd | Network discovery (mDNS/DNS-SD) |
| tokio | Async runtime |
| rusqlite | SQLite database (future: data persistence) |
| anyhow | Error handling |

### Network & Discovery
- **Local Discovery**: mDNS broadcasting and discovery (`_citinet._tcp.local.`)
- **Future**: WireGuard tunnels, ActivityPub federation, Matrix messaging

---

## License

See ATTRIBUTIONS.md for third-party licenses and credits.

Citizens' Digital Infrastructure Project — Citinet
