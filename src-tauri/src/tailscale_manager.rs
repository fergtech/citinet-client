use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const TAILSCALE_WINDOWS_PATH: &str = r"C:\Program Files\Tailscale\tailscale.exe";
const TAILSCALE_MSI_URL: &str =
    "https://pkgs.tailscale.com/stable/tailscale-setup-latest-amd64.msi";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailscaleStatus {
    pub installed: bool,
    pub logged_in: bool,
    pub machine_name: Option<String>,
    pub funnel_url: Option<String>, // https://machinename.tailXXXX.ts.net
    pub funnel_active: bool,
}

// Partial deserialization of `tailscale status --json`
#[derive(Debug, Deserialize)]
struct TsStatusJson {
    #[serde(rename = "BackendState")]
    backend_state: Option<String>,
    #[serde(rename = "Self")]
    self_node: Option<TsNode>,
}

#[derive(Debug, Deserialize)]
struct TsNode {
    #[serde(rename = "DNSName")]
    dns_name: Option<String>,
}

pub struct TailscaleManager;

impl TailscaleManager {
    pub fn new() -> Self {
        TailscaleManager
    }

    /// Returns the path to the tailscale binary, preferring the well-known Windows install path.
    fn cmd(&self) -> String {
        if std::path::Path::new(TAILSCALE_WINDOWS_PATH).exists() {
            TAILSCALE_WINDOWS_PATH.to_string()
        } else {
            "tailscale".to_string()
        }
    }

    pub fn is_installed(&self) -> bool {
        if std::path::Path::new(TAILSCALE_WINDOWS_PATH).exists() {
            return true;
        }
        Command::new("tailscale")
            .args(["version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn fetch_json_status(&self) -> Option<TsStatusJson> {
        let output = Command::new(self.cmd())
            .args(["status", "--json"])
            .output()
            .ok()?;
        serde_json::from_slice(&output.stdout).ok()
    }

    /// Low-level raw JSON parse — used during login probing so we can log
    /// every field and handle any schema differences between Tailscale versions.
    fn fetch_raw_status(&self) -> Option<serde_json::Value> {
        let output = Command::new(self.cmd())
            .args(["status", "--json"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&text).ok()
    }

    /// Pull `AuthURL` out of a raw status Value.
    fn extract_auth_url(v: &serde_json::Value) -> Option<String> {
        v.get("AuthURL")
            .and_then(|u| u.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    pub fn get_status(&self) -> TailscaleStatus {
        let installed = self.is_installed();
        if !installed {
            return TailscaleStatus {
                installed: false,
                logged_in: false,
                machine_name: None,
                funnel_url: None,
                funnel_active: false,
            };
        }

        let json = self.fetch_json_status();

        let logged_in = json
            .as_ref()
            .and_then(|j| j.backend_state.as_deref())
            .map(|s| s == "Running")
            .unwrap_or(false);

        // DNSName has a trailing dot (e.g. "machinename.tail12345.ts.net.") — strip it
        let dns_name = json
            .as_ref()
            .and_then(|j| j.self_node.as_ref())
            .and_then(|n| n.dns_name.as_deref())
            .map(|s| s.trim_end_matches('.').to_string());

        let funnel_url = dns_name
            .as_ref()
            .filter(|_| logged_in)
            .map(|n| format!("https://{}", n));

        let machine_name = dns_name
            .as_ref()
            .map(|n| n.split('.').next().unwrap_or(n).to_string());

        let funnel_active = logged_in && self.is_funnel_active();

        TailscaleStatus {
            installed,
            logged_in,
            machine_name,
            funnel_url,
            funnel_active,
        }
    }

    /// Check whether a Tailscale Funnel is currently active by inspecting `tailscale funnel status`.
    pub fn is_funnel_active(&self) -> bool {
        let output = Command::new(self.cmd())
            .args(["funnel", "status"])
            .output();
        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                // An active funnel listing contains port mapping lines
                text.contains(":443")
                    || text.contains("http://127.0.0.1")
                    || text.contains("https://localhost")
                    || text.contains("http://localhost")
            }
            Err(_) => false,
        }
    }

    /// Download and silently install Tailscale MSI.
    pub fn install(&self) -> Result<String> {
        log::info!("Downloading Tailscale MSI from {}", TAILSCALE_MSI_URL);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let resp = client
            .get(TAILSCALE_MSI_URL)
            .send()
            .context("Failed to download Tailscale MSI")?;

        if !resp.status().is_success() {
            anyhow::bail!("Download failed: HTTP {}", resp.status());
        }

        let bytes = resp.bytes().context("Failed to read MSI bytes")?;
        let msi_path = std::env::temp_dir().join("tailscale-setup.msi");
        std::fs::write(&msi_path, &bytes).context("Failed to write Tailscale MSI")?;

        log::info!("Running silent Tailscale install...");
        let status = Command::new("msiexec.exe")
            .args([
                "/i",
                msi_path.to_str().unwrap_or("tailscale-setup.msi"),
                "/qn",
                "TS_NOLAUNCH=1",
                "TS_CHECKUPDATES=never",
            ])
            .status()
            .context("Failed to run msiexec")?;

        let _ = std::fs::remove_file(&msi_path);

        if !status.success() {
            anyhow::bail!("Tailscale installation failed (exit code: {:?})", status.code());
        }

        log::info!("Tailscale installed successfully — waiting for service to start...");
        // Give the Tailscale Windows service a moment to initialise before CLI calls
        std::thread::sleep(std::time::Duration::from_secs(3));
        Ok("installed".to_string())
    }

    /// Trigger Tailscale authentication and return the login URL.
    ///
    /// Strategy (runs all three in parallel, first URL wins):
    ///
    /// 1. `tailscale up` with captured stderr — on many Windows setups this is
    ///    the command that actually prints "To authenticate, visit: https://…"
    /// 2. `tailscale login` with captured stderr — fallback for the login subcommand
    /// 3. Polling `tailscale status --json` → `AuthURL` — the daemon stores the URL
    ///    in its state; we log the BackendState on every iteration for diagnostics.
    ///
    /// We open the browser ourselves because the Tailscale service runs as
    /// LocalSystem (session 0) on Windows and cannot open interactive browser windows.
    pub fn start_login(&self) -> Result<String> {
        let cmd = self.cmd();

        // --- Pre-flight: log current daemon state ---
        if let Some(v) = self.fetch_raw_status() {
            let state = v.get("BackendState").and_then(|s| s.as_str()).unwrap_or("?");
            log::info!("tailscale pre-login BackendState: {}", state);
            // Already have an auth URL sitting in the daemon state?
            if let Some(url) = Self::extract_auth_url(&v) {
                log::info!("Auth URL already in daemon state: {}", url);
                Self::open_url_in_browser(&url);
                return Ok(url);
            }
            if state == "Running" {
                log::info!("Tailscale already logged in");
                return Ok(String::new());
            }
        } else {
            log::warn!("tailscale status --json returned nothing — service may not be running");
        }

        let (tx, rx) = mpsc::channel::<String>();

        // --- Strategy 1: tailscale up (no pipe capture — lets tray/service open browser normally) ---
        {
            let cmd = cmd.clone();
            std::thread::spawn(move || {
                log::info!("Trying: tailscale up (no-capture)");
                match Command::new(&cmd)
                    .args(["up"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(mut c) => {
                        let status = c.wait();
                        log::info!("tailscale up exited: {:?}", status.map(|s| s.code()));
                    }
                    Err(e) => log::warn!("tailscale up spawn failed: {}", e),
                }
            });
        }

        // --- Strategy 2: tailscale login — read stdout AND stderr in parallel sub-threads ---
        {
            let tx = tx.clone();
            let cmd = cmd.clone();
            std::thread::spawn(move || {
                log::info!("Trying: tailscale login");
                let child = Command::new(&cmd)
                    .args(["login"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::null())
                    .spawn();
                let mut child = match child {
                    Ok(c) => c,
                    Err(e) => { log::warn!("tailscale login spawn failed: {}", e); return; }
                };

                // Stderr sub-thread (runs concurrently with stdout read below)
                let tx2 = tx.clone();
                let stderr_pipe = child.stderr.take();
                let stderr_thread = std::thread::spawn(move || {
                    if let Some(pipe) = stderr_pipe {
                        for line in BufReader::new(pipe).lines().flatten() {
                            log::info!("tailscale login stderr: {}", line);
                            if let Some(url) = TailscaleManager::find_url_in_line(&line) {
                                let _ = tx2.send(url);
                                return;
                            }
                        }
                    }
                });

                // Stdout in this thread (parallel with stderr sub-thread)
                if let Some(pipe) = child.stdout.take() {
                    for line in BufReader::new(pipe).lines().flatten() {
                        log::info!("tailscale login stdout: {}", line);
                        if let Some(url) = TailscaleManager::find_url_in_line(&line) {
                            let _ = tx.send(url);
                            let _ = child.wait();
                            let _ = stderr_thread.join();
                            return;
                        }
                    }
                }

                let _ = stderr_thread.join();
                let status = child.wait();
                log::info!("tailscale login exited: {:?}", status.map(|s| s.code()));
            });
        }

        // --- Strategy 3: poll `tailscale status --json` for AuthURL ---
        // Fast (100 ms) for the first 5 s — catches a brief AuthURL window set by the tray.
        // Slow (1 s) for the remainder.
        {
            let tx = tx.clone();
            let cmd = cmd.clone();
            std::thread::spawn(move || {
                let start = Instant::now();
                let mut last_state = String::new();
                let mut last_auth_url = String::from("__unset__");
                let mut is_first = true;
                loop {
                    if start.elapsed() > Duration::from_secs(60) { break; }
                    let sleep_ms = if start.elapsed() < Duration::from_secs(5) { 100 } else { 1000 };
                    std::thread::sleep(Duration::from_millis(sleep_ms));

                    let out = Command::new(&cmd).args(["status", "--json"]).output();
                    let text = match out {
                        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                        Err(e) => { log::warn!("tailscale status poll error: {}", e); continue; }
                    };
                    let v: serde_json::Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(_) => { log::warn!("tailscale status non-JSON: {}", &text[..text.len().min(200)]); continue; }
                    };

                    // On the first successful parse, log all top-level JSON keys
                    if is_first {
                        is_first = false;
                        if let Some(obj) = v.as_object() {
                            let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                            log::info!("tailscale status JSON top-level keys: {:?}", keys);
                        }
                    }

                    let state = v.get("BackendState")
                        .and_then(|s| s.as_str())
                        .unwrap_or("?")
                        .to_string();

                    if state != last_state {
                        log::info!("tailscale BackendState: {}", state);
                        last_state = state.clone();
                    }

                    if state == "Running" { break; }

                    // Log whenever the AuthURL field changes (distinguishes absent vs empty vs set)
                    let auth_url_present = v.get("AuthURL").is_some();
                    let auth_url = v.get("AuthURL")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                    if auth_url != last_auth_url {
                        log::info!("tailscale AuthURL changed: {:?} (key_present={})", auth_url, auth_url_present);
                        last_auth_url = auth_url.clone();
                    }
                    if !auth_url.is_empty() {
                        log::info!("tailscale AuthURL ready: {}", auth_url);
                        let _ = tx.send(auth_url);
                        return;
                    }
                }
            });
        }

        // Wait up to 65 seconds for any strategy to produce a URL
        match rx.recv_timeout(Duration::from_secs(65)) {
            Ok(url) if !url.is_empty() => {
                log::info!("Got Tailscale auth URL: {}", url);
                Self::open_url_in_browser(&url);
                Ok(url)
            }
            _ => {
                log::warn!("Could not obtain Tailscale auth URL after 65 s — user may need to authenticate manually via the Tailscale tray/CLI");
                Ok(String::new())
            }
        }
    }

    /// Scan a line of text for a Tailscale login URL.
    fn find_url_in_line(line: &str) -> Option<String> {
        // Tailscale prints "https://login.tailscale.com/a/…" in auth prompts
        line.find("https://login.tailscale.com")
            .map(|pos| {
                line[pos..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('.')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
    }

    /// Open a URL in the default system browser.
    /// Must be called from a user-session process (not a Windows service).
    fn open_url_in_browser(url: &str) {
        #[cfg(target_os = "windows")]
        {
            // The empty `""` is the window title — required when the URL contains
            // shell-special characters (&, =, ?) that cmd.exe would misparse.
            let _ = Command::new("cmd")
                .args(["/C", "start", "", url])
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(url).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(url).spawn();
        }
    }

    /// Check whether the Tailscale daemon is in the Running (logged-in) state.
    pub fn poll_login(&self) -> bool {
        self.fetch_json_status()
            .and_then(|j| j.backend_state)
            .map(|s| s == "Running")
            .unwrap_or(false)
    }

    /// Enable Tailscale Funnel for the given port (background mode) and return the stable HTTPS URL.
    pub fn enable_funnel(&self, port: u16) -> Result<String> {
        log::info!("Enabling Tailscale Funnel on port {}", port);

        let output = Command::new(self.cmd())
            .args(["funnel", "--bg", &port.to_string()])
            .output()
            .context("Failed to run tailscale funnel")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("tailscale funnel failed: {} {}", stderr, stdout);
        }

        // Give the Tailscale daemon a moment to register the funnel
        std::thread::sleep(std::time::Duration::from_millis(500));

        let status = self.get_status();
        status.funnel_url.ok_or_else(|| {
            anyhow::anyhow!(
                "Funnel started but URL not available — ensure Tailscale is logged in and your tailnet name is configured"
            )
        })
    }

    /// Disable the Tailscale Funnel.
    pub fn disable_funnel(&self) -> Result<()> {
        log::info!("Disabling Tailscale Funnel");

        let output = Command::new(self.cmd())
            .args(["funnel", "off"])
            .output()
            .context("Failed to run tailscale funnel off")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tailscale funnel off failed: {}", stderr);
        }

        Ok(())
    }
}
