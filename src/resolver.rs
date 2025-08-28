//! The core DNS resolution logic.

use std::net::SocketAddr;

use crate::{blocklist::Blocklist, cache::DnsCache};


/// Handles the core logic of resolving DNS queries, including
/// checking the blocklist, cache, and forwarding to upstream.
pub struct Resolver {
    upstream_dns_addr: SocketAddr,
    cache: DnsCache,
    blocklist: Blocklist,
}

impl Resolver {
    /// create a new resolver
    pub fn new(upstream_dns_addr: SocketAddr, cache: DnsCache, blocklist: Blocklist) -> Self {
        Self {
            upstream_dns_addr,
            cache,
            blocklist,
        }
        
    }
    
    /// Resolves a DNS query
    pub async fn resolve(&mut self, query: &[u8]) -> anyhow::Result<Vec<u8>> {
         // TODO: Implement the full resolution logic
        // 1. Parse the query to get the domain name
        // 2. Check blocklist -> return sinkhole if blocked
        // 3. Check cache -> return cached response if exists
        // 4. Forward to upstream DNS
        // 5. Cache the response before returning
        Ok(vec![]) // placeholder
    }

    }
