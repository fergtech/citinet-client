
# Citinet Engine‑First Plan — **Agent‑Aware** (Docker + PostgreSQL + Tunnel Fallback)

**Date:** 2026‑02‑14  
**Owner:** Zuriel (Citinet)  
**Audience:** Local code‑aware Agent + Engineers  
**Scope:** Same as the engine‑first plan, but **explicitly written for a code‑aware local agent** that has **full access** to the workspace, repositories, Docker, and OS. The agent will *build, run, migrate, test, and modify code* locally while the `citinet.io` DNS remains blocked.

> This document **assumes the agent can read the entire codebase**, run shell commands, build containers/apps, write files, open PRs, and manage local secrets. It provides **guardrails, tasks, and checklists** that the agent can execute autonomously.

---

## 0) Trust boundaries & guardrails (Agent)

1. **Local‑only execution by default.** Do not exfiltrate code or secrets. All telemetry is local unless explicitly enabled by the operator.  
2. **Secrets** (Cloudflare token, DB password) are generated into `.env` files and/or the OS keychain; **never commit** to Git.  
3. **Idempotent operations.** All steps are re‑runnable without breaking state.  
4. **Rollback safety.** Keep prior compose/env versions; support `down` + restart.  
5. **Human override.** On destructive actions (volume wipe), require explicit user confirmation.

---

## 1) Operating assumptions (what the Agent knows)

- Repos present locally (monorepo or multi‑repo):
  - **desktop** (Tauri/Electron app + wizard)
  - **hub‑api** (backend service to run in Docker)
  - **webapp** (Vercel‑deployed start portal)
  - **docs** (optional)
- **Docker** is installed; the agent can run `docker`, `docker compose`.
- The agent can install/update **cloudflared** if missing.
- The agent can generate files, run migrations, and open PRs/branches.

---

## 2) End‑to‑end target (recap)

On this machine, the agent must produce a working **Hub**:
- `docker compose up -d` → **Postgres 16** + **hub‑api** reachable at `http://localhost:8080`.
- **Cloudflare Tunnel** started; report **`https://<uuid>.cfargotunnel.com`** as the remote URL.
- Wizard completes; dashboard shows **Local + Remote (temporary)** URLs; DNS status = **Pending**.

---

## 3) File/Directory layout the Agent should (create/verify)

```
/infra
  docker-compose.yml
  .env.example
  Makefile
  cloudflared/
    bin/ (optional cached binary)
    config.yml (generated)
/docs
  Citinet-Engine-First-Plan.md
  Citinet-Engine-First-Plan-AGENT.md  ← this file
```

> If repos are split, the agent ensures an `/infra` folder exists at the root of the workspace that manages containers for the **local development Hub**.

---

## 4) Compose specification (Agent responsibilities)

- Create (or update) **`/infra/docker-compose.yml`** with services:
  - `citinet-db` → `postgres:16-alpine` (named volume `citinet_db_data`)
  - `citinet-api` → image built from `hub-api/` Dockerfile
- Generate **`/infra/.env`** with a strong `CITINET_DB_PASSWORD` if not present.
- Ensure **network** `citinet_net` exists; create if missing.
- Expose API to host only on `127.0.0.1:8080`.

**Compose (baseline)** — same as main plan; the agent can merge/update.

---

## 5) Database & migrations (Agent)

1. Build `hub-api` image; run `migrate up` (SQLx/Prisma/Flyway—detect from repo).  
2. If no migration tool is found, the agent writes a bootstrap SQL file from the **MVP schema** and executes it via `psql` inside the DB container.  
3. Verify readiness: retry `SELECT 1` until success; cache DB connection string.  
4. Seed minimal rows for `nodes`, `node_runtime` once the wizard creates identity.

---

## 6) cloudflared lifecycle (Agent)

1. **Install/verify** `cloudflared` (cache under `/infra/cloudflared/bin`).  
2. **Create tunnel**: `cloudflared tunnel create <node_slug>`; capture `tunnelId`.  
3. **Run tunnel** with a generated config that proxies `https://<uuid>.cfargotunnel.com → http://localhost:8080`.  
4. Parse/record **fallback hostname**; write to DB `node_runtime.tunnel_hostname`.  
5. Mark DNS bind intent (`{slug}.citinet.io`) as **`pending`**.

---

## 7) Wizard hooks (what the desktop app calls)

- `POST /api/hub/init` → idempotent bootstrap (DB rows; identity from `node_identity.json`).
- `POST /api/tunnel/setup` → orchestrates steps in §6 and returns `{ tunnelId, tunnelHostname }`.
- `GET  /api/hub/info` → used by Complete step to display URLs + bind status.

The agent ensures these routes exist in `hub-api` and wires the desktop app to them.

---

## 8) Pairing & auth (Agent)

- Generate an **owner pairing code** (6‑digit) stored server‑side with TTL.  
- Gate **write** endpoints until pairing succeeds.  
- For the public tunnel hostname, **expose only** health/info pre‑pairing.

---

## 9) Developer ergonomics (Agent creates)

### Makefile shortcuts
```make
up:    
	docker compose -f infra/docker-compose.yml --env-file infra/.env up -d

down:
	docker compose -f infra/docker-compose.yml --env-file infra/.env down

ps:
	docker compose -f infra/docker-compose.yml --env-file infra/.env ps

logs:
	docker compose -f infra/docker-compose.yml --env-file infra/.env logs -f --tail=200

backup:
	@ts=$$(date +"%Y%m%d-%H%M%S"); \
	docker exec citinet-db pg_dump -U citinet citinet > backups/backup-$$ts.sql
```

### VS Code tasks (optional)
- `Dev: Up`, `Dev: Down`, `Dev: Logs`, `Dev: Migrate`, `Dev: Open Dashboard`.

---

## 10) Observability (Agent)

- Tail `citinet-api` logs + `cloudflared` logs in a split pane.  
- Provide a local HTML report (status page) under `/infra/_status/index.html` that polls `GET /api/health`.

---

## 11) Safety checks (Agent)

Before marking the hub **Ready**:
1. `docker compose ps` shows `citinet-db` and `citinet-api` **healthy**.  
2. `GET /api/health` returns **200** with version/uptime.  
3. Tunnel returns a reachable **fallback URL**.  
4. Disk space ≥ configured minimum; volumes mounted.  
5. `.env` is present; **no secrets** are in Git changes.

---

## 12) “Bind DNS Later” workflow (Agent)

- Periodically check for clearance (operator toggles a flag).  
- When allowed: create CNAME `{slug}.citinet.io → {tunnelId}.cfargotunnel.com`.  
- Update DB to `dns_bind_status='bound'` and surface canonical URL in UI.

---

## 13) Milestones (Agent‑executable tasks)

**M1 — Engine Online**
- Build `hub-api` image; run compose; run migrations.  
- Implement `/api/hub/init`, `/api/hub/info`, `/api/tunnel/setup`.  
- Install/run tunnel; record fallback URL.  
- Wire Wizard → Complete screen with URLs.  

**M2 — Admin & Spaces**
- Space create/list endpoints and minimal UI.  
- Logs/backup controls; contribution sliders persisted.  
- QR code for remote URL.  

**M3 — First App Feature**
- Select **one** feature to implement end‑to‑end inside a Space (Discussions/Files/Events).

---

## 14) Open questions the Agent should ask/resolve

1) Backend stack in `hub-api` (Rust/Actix vs Node/Fastify vs Go/Fiber).  
2) Migrations tool of record (SQLx, Prisma, Flyway) and where schema files live.  
3) Secrets store preference (OS keychain vs .env only).  
4) Packaging of `cloudflared` (download on first run vs bundle).  
5) Windows service / macOS launchd integration for auto‑start.

---

## 15) Definition of Done (Automated)

From a clean machine with Docker installed, the **agent** can:
1. Scaffold `/infra` (compose, env, Makefile).  
2. Build `hub-api`; start DB+API; run migrations.  
3. Generate `node_identity.json`; call `/api/hub/init`.  
4. Install `cloudflared`; create/run tunnel; obtain fallback URL.  
5. Open dashboard at `http://localhost:8080` and verify remote URL reachability.  
6. Persist everything; recover after reboot; idempotency proven.

**Result:** A real, stable Hub engine runs locally and remotely (temporary URL), fully controlled by the desktop app. DNS binding is a non‑blocking enhancement applied later.
