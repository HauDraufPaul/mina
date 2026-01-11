use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::process::Command;
use std::str;

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
        #[cfg(target_os = "macos")]
        {
            Self::get_connections_macos()
        }
        
        #[cfg(target_os = "linux")]
        {
            Self::get_connections_linux()
        }
        
        #[cfg(target_os = "windows")]
        {
            Self::get_connections_windows()
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            vec![]
        }
    }
    
    #[cfg(target_os = "macos")]
    fn get_connections_macos() -> Vec<NetworkConnection> {
        let mut connections = Vec::new();
        
        // Use netstat command on macOS
        if let Ok(output) = Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()
        {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                for line in output_str.lines().skip(2) {
                    // Skip header lines
                    if line.trim().is_empty() || line.starts_with("Proto") {
                        continue;
                    }
                    
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let protocol = parts[0].to_string();
                        let local_addr = parts[3].to_string();
                        let state = if parts.len() > 4 {
                            parts[4].to_string()
                        } else {
                            "UNKNOWN".to_string()
                        };
                        
                        // Try to extract remote address (may not always be present)
                        let remote_addr = if parts.len() > 3 && !state.is_empty() && state != "LISTEN" {
                            parts.get(4).map(|s| s.to_string()).unwrap_or_else(|| "".to_string())
                        } else {
                            "".to_string()
                        };
                        
                        // Try to get process ID using lsof (more reliable on macOS)
                        let process_id = Self::get_process_id_for_connection_macos(&local_addr);
                        
                        connections.push(NetworkConnection {
                            local_address: local_addr,
                            remote_address: remote_addr,
                            protocol,
                            state,
                            process_id,
                        });
                    }
                }
            }
        }
        
        connections
    }
    
    #[cfg(target_os = "macos")]
    fn get_process_id_for_connection_macos(local_addr: &str) -> Option<u32> {
        // Extract port from address (format: IP:PORT)
        if let Some(port_part) = local_addr.split(':').last() {
            if let Ok(port) = port_part.parse::<u16>() {
                // Use lsof to find process using this port
                if let Ok(output) = Command::new("lsof")
                    .args(&["-i", &format!(":{}", port), "-t"])
                    .output()
                {
                    if let Ok(pid_str) = str::from_utf8(&output.stdout) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            return Some(pid);
                        }
                    }
                }
            }
        }
        None
    }
    
    #[cfg(target_os = "linux")]
    fn get_connections_linux() -> Vec<NetworkConnection> {
        let mut connections = Vec::new();
        
        // Parse /proc/net/tcp for TCP connections
        if let Ok(content) = std::fs::read_to_string("/proc/net/tcp") {
            for line in content.lines().skip(1) {
                // Skip header
                if let Some(conn) = Self::parse_proc_net_line(line, "TCP") {
                    connections.push(conn);
                }
            }
        }
        
        // Parse /proc/net/udp for UDP connections
        if let Ok(content) = std::fs::read_to_string("/proc/net/udp") {
            for line in content.lines().skip(1) {
                if let Some(conn) = Self::parse_proc_net_line(line, "UDP") {
                    connections.push(conn);
                }
            }
        }
        
        connections
    }
    
    #[cfg(target_os = "linux")]
    fn parse_proc_net_line(line: &str, protocol: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }
        
        // Parse local and remote addresses (format: IP:PORT in hex)
        let local = Self::parse_proc_net_address(parts.get(1)?)?;
        let remote = Self::parse_proc_net_address(parts.get(2)?)?;
        
        // Parse state (for TCP)
        let state = if protocol == "TCP" {
            if let Some(state_hex) = parts.get(3) {
                Self::parse_tcp_state(state_hex)
            } else {
                "UNKNOWN".to_string()
            }
        } else {
            "UNCONN".to_string() // UDP is connectionless
        };
        
        // Get process ID from /proc/net lookup (requires parsing inode)
        let process_id = if parts.len() > 9 {
            // inode is in position 9, we can use it to find the process
            parts.get(9).and_then(|inode| {
                Self::find_process_by_inode_linux(inode.parse().ok()?)
            })
        } else {
            None
        };
        
        Some(NetworkConnection {
            local_address: local,
            remote_address: remote,
            protocol: protocol.to_string(),
            state,
            process_id,
        })
    }
    
    #[cfg(target_os = "linux")]
    fn parse_proc_net_address(addr: &str) -> Option<String> {
        // Format: IP:PORT in hex (e.g., "0100007F:0016" = 127.0.0.1:22)
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let ip_hex = parts[0];
        let port_hex = parts[1];
        
        // Parse IP (little-endian hex)
        if ip_hex.len() == 8 {
            let ip_bytes: Vec<u8> = (0..4)
                .map(|i| u8::from_str_radix(&ip_hex[i*2..i*2+2], 16).ok())
                .collect::<Option<Vec<_>>>()?;
            
            let ip = format!("{}.{}.{}.{}", ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]);
            
            // Parse port (big-endian hex)
            let port = u16::from_str_radix(port_hex, 16).ok()?;
            
            Some(format!("{}:{}", ip, port))
        } else {
            None
        }
    }
    
    #[cfg(target_os = "linux")]
    fn parse_tcp_state(state_hex: &str) -> String {
        // TCP states from /proc/net/tcp
        match u8::from_str_radix(state_hex, 16).unwrap_or(0) {
            0x01 => "ESTABLISHED",
            0x02 => "SYN_SENT",
            0x03 => "SYN_RECV",
            0x04 => "FIN_WAIT1",
            0x05 => "FIN_WAIT2",
            0x06 => "TIME_WAIT",
            0x07 => "CLOSE",
            0x08 => "CLOSE_WAIT",
            0x09 => "LAST_ACK",
            0x0A => "LISTEN",
            0x0B => "CLOSING",
            _ => "UNKNOWN",
        }.to_string()
    }
    
    #[cfg(target_os = "linux")]
    fn find_process_by_inode_linux(inode: u64) -> Option<u32> {
        // Search /proc/*/fd/* for the inode
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(pid_str) = entry.file_name().into_string() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        // Check this process's file descriptors
                        let fd_dir = format!("/proc/{}/fd", pid);
                        if let Ok(fd_entries) = std::fs::read_dir(&fd_dir) {
                            for fd_entry in fd_entries.flatten() {
                                if let Ok(link) = std::fs::read_link(fd_entry.path()) {
                                    if let Some(link_str) = link.to_str() {
                                        // Check if this is a socket with our inode
                                        if link_str.contains(&format!("socket:[{}]", inode)) {
                                            return Some(pid);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    #[cfg(target_os = "windows")]
    fn get_connections_windows() -> Vec<NetworkConnection> {
        let mut connections = Vec::new();
        
        // Use netstat command on Windows
        if let Ok(output) = Command::new("netstat")
            .args(&["-ano"])
            .output()
        {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                for line in output_str.lines().skip(3) {
                    // Skip header lines
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let protocol = parts[0].to_string();
                        let local_addr = parts[1].to_string();
                        let remote_addr = parts[2].to_string();
                        let state = if parts.len() > 3 {
                            parts[3].to_string()
                        } else {
                            "UNKNOWN".to_string()
                        };
                        
                        // Process ID is the last column
                        let process_id = parts.last()
                            .and_then(|s| s.parse::<u32>().ok());
                        
                        connections.push(NetworkConnection {
                            local_address: local_addr,
                            remote_address: remote_addr,
                            protocol,
                            state,
                            process_id,
                        });
                    }
                }
            }
        }
        
        connections
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

