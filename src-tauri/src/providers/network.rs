use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub local_address: String,
    pub remote_address: String,
    pub protocol: String,
    pub state: String,
    pub process_id: Option<u32>,
}

pub struct NetworkProvider {
    system: System,
}

impl NetworkProvider {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        NetworkProvider { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_connections(&self) -> Vec<NetworkConnection> {
        // Note: sysinfo doesn't provide detailed network connection info
        // This would need platform-specific implementations
        // For now, return empty vector
        vec![]
    }

    pub fn get_interfaces(&self) -> Vec<NetworkInterface> {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();
        let mut interfaces = Vec::new();
        
        for (name, network) in networks.list() {
            interfaces.push(NetworkInterface {
                name: name.to_string(),
                received: network.received(),
                transmitted: network.transmitted(),
            });
        }
        
        interfaces
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
}

impl Default for NetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

