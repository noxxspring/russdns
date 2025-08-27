//! The main entry point for the russdns daemon.

mod config;
mod server;
mod resolver;
mod cache;
mod blocklist;

use crate::config::Config;
use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main()-> Result<()> {
    // TODO: Step 1: - initialize logging based on config

    //TODO: step 2: Load the configuration from file
    let config = Config::load("config.toml").expect("Failed to load configuration");

    // TODO: Step 3: initialize the blocklist from file
    info!("Loading blocklist from {}", config.blocklist_file);
    let blocklist = blocklist::Blocklist::load(&config.blocklist_file)?;

    // TODO: Step 4 - Initialize the cache
    let cache = cache::DnsCache::new(1000); // Example: 1000 item cache

    // TODO: Step 5: Initialize the resolver with the cache and blocklist
    let resolver = resolver::Resolver::new(config.upstream_dns_addr, cache, blocklist);

    // TODO: Step 6: start the DNS server
    info!("Starting russdns server on {}", config.listen_addr);
    server::start_server(&config.listen_addr, resolver).await?;

    Ok(())

}
