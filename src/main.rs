//! The main entry point for the russdns daemon.

mod config;
mod server;
mod resolver;
mod cache;
mod blocklist;

// Use statement for modules
use crate::config::Config;
use anyhow::Result;

// Use staement for dependencies
use tracing::{info, debug,};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use std::fs::OpenOptions;


#[tokio::main]
async fn main()-> Result<()> {
    // TODO: Step 1: - Load the configuration from file
    // hardcoded for simplicity for now
    let config = Config::load("config.toml").expect("Failed to load configuration");


    //TODO: step 2: Initialize logging based on config
    let log_filter = match config.log_level.to_lowercase().as_str() {
        "trace" => tracing_subscriber::filter::LevelFilter::TRACE,
        "debug" => tracing_subscriber::filter::LevelFilter::DEBUG,
        "warn" => tracing_subscriber::filter::LevelFilter::WARN,
        "error" => tracing_subscriber::filter::LevelFilter::ERROR,
        _ => tracing_subscriber::filter::LevelFilter::INFO,   // default to info
    };

    // Create a log we can append to .
    let log_file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&config.log_file)?;


    // Create a formatting layer that writes JSON to the log file
    let file_layer = tracing_subscriber::fmt::layer()
    .with_target(false)  //Dont include target (module path) for cleaner log
    .with_level(true)    // Include the log level
    .with_thread_ids(false) // Simpler logs without thread IDs
    .with_thread_names(false)
    .with_span_events(FmtSpan::NONE)
    .with_writer(log_file)
    .json();  // Ouput as json


    // Create a formatting layer that writes to stdout
    let stdout_layer = tracing_subscriber::fmt::layer()
    .with_target(true)  // Do include target for console debugging
    .with_level(true)
    .with_thread_ids(false)
    .with_thread_names(false)
    .with_span_events(FmtSpan::NONE)
    .with_writer(std::io::stdout); // write to the console


    // Combine the layers and set the global default subscriber
    tracing_subscriber::registry()
    .with(file_layer)
    .with(stdout_layer.with_filter(log_filter))
    .init();

    info!("Loaded configuration from config.toml");
    debug!("Full config:{:?}", config);

    // TODO: Step 3: initialize the blocklist from file
    info!("Loading blocklist from {}", config.blocklist_file);
    // let blocklist = blocklist::Blocklist::load(&config.blocklist_file)?;

    // TODO: Step 4 - Initialize the cache
    // let cache = cache::DnsCache::new(1000); // Example: 1000 item cache

    // TODO: Step 5: Initialize the resolver with the cache and blocklist
    // let resolver = resolver::Resolver::new(config.upstream_dns_addr, cache, blocklist);

    // TODO: Step 6: start the DNS server
    info!("RussDNS daemon starting up....");
    info!("Server will listen on: {}", config.listen_addr);
    info!("Upstream DNS server: {}", config.upstream_dns_addr);
    info!("Block action: {}", config.block_action);
    info!("Sinkhole IP {}", config.sinkhole_ip);
    info!("Log file: {:?}", config.log_file);

    // server::start_server(&config.listen_addr, resolver).await?;

    // for now, just sleep to keep the program running so we can see the logs
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Exiting.");

    Ok(())

}
