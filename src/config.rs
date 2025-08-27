//! Configuration handling for the russdns daemon.
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;

use crate::config;



/// The main configuration struct for the application.
/// This is derived from the TOML config file.
#[derive(Debug, Deserialize)]
pub struct Config{
    /// The socket address to bind the DNS server to (e.g, 0.0.0.0:53)
    pub listen_addr: String,
    
    /// The Upstream DNS server to forward queries to (e.g, 1.1.1.1:53)
    pub upstream_dns_addr: String,

    /// The action to take for blocked request: "Sinkhole" or "Nxdomain"
    pub block_action: String,

    /// The IP address to return to the sinkhole responses
    pub sinkhole_ip: String,

    /// Path to the bloacklist file
    pub blocklist_file: String,

    /// Path to the log file
    pub log_file: String,

    /// Log level: trace, debug, info, warn, error
    pub log_level: String

}

impl Config {
    ///Load the configuration from the TOML file at the given path
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}