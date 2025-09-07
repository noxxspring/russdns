//! Configuration handling for the russdns daemon.

use serde::Deserialize;
use std::fmt;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

/// The action to take when a domain is blocked.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum BlockAction {
    /// Respond with a sinkhole IP address.
    #[serde(alias = "sinkhole", alias = "Sinkhole")]
    Sinkhole,
    /// Respond with an NXDOMAIN (non-existent domain) error.
    #[serde(alias = "nxdomain", alias = "Nxdomain")]
    Nxdomain,
}

// Implement Display for BlockAction
impl fmt::Display for BlockAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockAction::Sinkhole => write!(f, "Sinkhole"),
            BlockAction::Nxdomain => write!(f, "NXDOMAIN"),
        }
    }
}

/// The main configuration struct for the application.
/// This is derived from the TOML config file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The socket address to bind the DNS server to (e.g., "0.0.0.0:53")
    pub listen_addr: String,
    /// The upstream DNS server to forward queries to (e.g., "1.1.1.1:53")
    pub upstream_dns_addr: String,
    /// The action to take for blocked requests.
    pub block_action: BlockAction,
    /// The IP address to return for blocked requests if action is "Sinkhole".
    pub sinkhole_ip: String,
    /// Path to the file containing blocked domains, one per line.
    pub blocklist_file: PathBuf,
    /// Path to the log file.
    pub log_file: PathBuf,
    /// Log level: trace, debug, info, warn, error
    pub log_level: String,
}

impl Config {
    /// Loads the configuration from a TOML file at the given path.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let config_content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path, e))?;
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", path, e))?;
        Ok(config)
    }

    // Helper method to parse listen_addr (we'll use this later in the server)
    pub fn listen_socket_addr(&self) -> anyhow::Result<SocketAddr> {
        self.listen_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid listen address '{}': {}", self.listen_addr, e))
    }

    // Helper method to parse upstream_dns_addr
    pub fn upstream_socket_addr(&self) -> anyhow::Result<SocketAddr> {
        self.upstream_dns_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid upstream address '{}': {}", self.upstream_dns_addr, e))
    }
}