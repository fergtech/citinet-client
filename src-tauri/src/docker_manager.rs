use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStatus {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub names: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: String,
    pub created: String,
}

pub fn check_docker() -> DockerStatus {
    let output = Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                DockerStatus {
                    installed: true,
                    running: true,
                    version: Some(version),
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                // Docker installed but daemon not running
                DockerStatus {
                    installed: true,
                    running: false,
                    version: None,
                    error: Some(stderr),
                }
            }
        }
        Err(_) => DockerStatus {
            installed: false,
            running: false,
            version: None,
            error: Some("Docker not found in PATH".to_string()),
        },
    }
}

pub fn list_containers() -> Result<Vec<DockerContainer>, String> {
    let output = Command::new("docker")
        .args([
            "ps", "-a",
            "--format", "{{json .}}",
        ])
        .output()
        .map_err(|e| format!("Failed to run docker ps: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("docker ps failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Docker outputs JSON with capital-letter keys
        let raw: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("Failed to parse docker output: {}", e))?;

        containers.push(DockerContainer {
            id: raw.get("ID").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            names: raw.get("Names").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            image: raw.get("Image").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            status: raw.get("Status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            state: raw.get("State").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            ports: raw.get("Ports").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            created: raw.get("CreatedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        });
    }

    Ok(containers)
}

pub fn start_container(id: &str) -> Result<(), String> {
    run_docker_command(&["start", id])
}

pub fn stop_container(id: &str) -> Result<(), String> {
    run_docker_command(&["stop", id])
}

pub fn restart_container(id: &str) -> Result<(), String> {
    run_docker_command(&["restart", id])
}

fn run_docker_command(args: &[&str]) -> Result<(), String> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run docker {}: {}", args[0], e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("docker {} failed: {}", args[0], stderr))
    }
}
