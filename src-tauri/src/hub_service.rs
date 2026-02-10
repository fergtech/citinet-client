use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE_TYPE: &str = "_citinet._tcp.local.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubNode {
    pub id: String,
    pub name: String,
    pub addresses: Vec<String>,
    pub port: u16,
    pub node_type: String,
    pub services: Vec<String>,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubServiceInfo {
    pub service_name: String,
    pub port: u16,
    pub services: Vec<String>,
    pub is_running: bool,
}

pub struct HubService {
    daemon: Option<ServiceDaemon>,
    discovered_nodes: Arc<Mutex<HashMap<String, HubNode>>>,
    info: Arc<Mutex<HubServiceInfo>>,
}

impl HubService {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            daemon: None,
            discovered_nodes: Arc::new(Mutex::new(HashMap::new())),
            info: Arc::new(Mutex::new(HubServiceInfo {
                service_name: String::new(),
                port: 9090,
                services: vec![],
                is_running: false,
            })),
        })
    }

    pub fn start_broadcasting(&mut self, node_name: String, services: Vec<String>) -> Result<(), String> {
        let daemon = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        
        let port = 9090u16;
        let hostname = format!("{}.local.", node_name.replace(' ', "-").to_lowercase());
        
        // Create service properties
        let mut properties = HashMap::new();
        properties.insert("version".to_string(), "0.1.0".to_string());
        properties.insert("node_type".to_string(), "hub".to_string());
        properties.insert("services".to_string(), services.join(","));
        
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &node_name,
            &hostname,
            "",
            port,
            Some(properties),
        ).map_err(|e| format!("Failed to create service info: {}", e))?;

        daemon.register(service_info)
            .map_err(|e| format!("Failed to register service: {}", e))?;

        let mut info = self.info.lock().map_err(|e| e.to_string())?;
        info.service_name = node_name;
        info.port = port;
        info.services = services;
        info.is_running = true;

        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn stop_broadcasting(&mut self) -> Result<(), String> {
        if let Some(daemon) = self.daemon.take() {
            daemon.shutdown().map_err(|e| format!("Failed to stop broadcasting: {}", e))?;
            
            let mut info = self.info.lock().map_err(|e| e.to_string())?;
            info.is_running = false;
        }
        Ok(())
    }

    pub fn start_discovery(&mut self) -> Result<(), String> {
        let daemon = ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
        
        let discovered_nodes = Arc::clone(&self.discovered_nodes);
        let receiver = daemon.browse(SERVICE_TYPE)
            .map_err(|e| format!("Failed to browse services: {}", e))?;

        // Spawn a thread to handle discoveries
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        let addresses: Vec<String> = info.get_addresses()
                            .iter()
                            .map(|addr| addr.to_string())
                            .collect();

                        let services = info.get_properties()
                            .get("services")
                            .map(|s| s.val_str().split(',').map(|x| x.to_string()).collect())
                            .unwrap_or_else(Vec::new);

                        let node_type = info.get_properties()
                            .get("node_type")
                            .map(|s| s.val_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        let hub_node = HubNode {
                            id: info.get_fullname().to_string(),
                            name: info.get_hostname().to_string(),
                            addresses,
                            port: info.get_port(),
                            node_type,
                            services,
                            last_seen: timestamp,
                        };

                        if let Ok(mut nodes) = discovered_nodes.lock() {
                            nodes.insert(hub_node.id.clone(), hub_node);
                        }
                    }
                    mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                        if let Ok(mut nodes) = discovered_nodes.lock() {
                            nodes.remove(&fullname);
                        }
                    }
                    _ => {}
                }
            }
        });

        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn get_discovered_nodes(&self) -> Result<Vec<HubNode>, String> {
        let nodes = self.discovered_nodes.lock().map_err(|e| e.to_string())?;
        Ok(nodes.values().cloned().collect())
    }

    pub fn get_service_info(&self) -> Result<HubServiceInfo, String> {
        let info = self.info.lock().map_err(|e| e.to_string())?;
        Ok(info.clone())
    }
}

impl Default for HubService {
    fn default() -> Self {
        Self::new().expect("Failed to create HubService")
    }
}

impl Drop for HubService {
    fn drop(&mut self) {
        let _ = self.stop_broadcasting();
    }
}
