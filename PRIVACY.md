# Privacy Policy

**Citinet** is a community mesh networking desktop application built on [Tauri](https://tauri.app).

## Data Collection

This program does not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

Specifically:

- **No telemetry or analytics** are collected or transmitted.
- **No crash reports** are sent to external servers.
- **No usage data** is shared with the developers or any third party.

## Data Stored Locally

All data Citinet stores (node configuration, messages, files, user accounts, system metrics) is kept exclusively on the user's own machine in a SQLite database at the install path chosen during setup. This data never leaves the device unless the user explicitly configures a tunnel or shares files through the application.

## Cloudflare Tunnel (Optional)

If the user chooses to enable a Cloudflare tunnel, Citinet communicates with the Cloudflare API (`api.cloudflare.com`) using credentials supplied by the user. This is a user-initiated feature. No credentials or tunnel metadata are shared with the Citinet developers.

The `cloudflared` binary used to establish tunnels is published by Cloudflare and downloaded from their official GitHub releases (`github.com/cloudflare/cloudflared`) when the user initiates installation of the tunnel client.

Cloudflare's own privacy policy governs any data processed by their infrastructure:
https://www.cloudflare.com/privacypolicy/

## Third-Party Components

Citinet uses the following third-party open source components that may have their own privacy considerations:

- **cloudflared** (Apache 2.0) — Cloudflare tunnel client. See https://www.cloudflare.com/privacypolicy/
- **SQLite** (Public Domain) — Local database engine. No network access.
- All other dependencies are local libraries with no network communication of their own.

## Contact

For privacy questions, open an issue at https://github.com/fergtech/citinet-client/issues
