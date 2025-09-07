//! Blocklist management for the DNS filter.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use arc_swap::ArcSwap;
use tracing::{debug, info};

/// A thread-safe, reloadable list of blocked domains.
pub struct Blocklist {
    // ArcSwap allows us to atomically update the entire list without locking
    domains: ArcSwap<HashSet<String>>,
}

impl Clone for Blocklist {
    fn clone(&self) -> Self {
        // Create a new Blocklist with a clone of the current domains
        let current_domains = self.domains.load();
        Self {
            domains: ArcSwap::new(Arc::clone(&current_domains)),
        }
    }
}

impl Blocklist {
    /// Load the blocklist from the file, with one domain per line
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut domains = HashSet::new();

        for line in reader.lines() {
            let line = line?;
            let domain = line.trim();
            // Skip empty lines and comments
            if domain.is_empty() || domain.starts_with('#') {
                continue;
            }
            domains.insert(domain.to_lowercase());
        }

        info!("Loaded {} domains into blocklist", domains.len());
        debug!("Blocked domains: {:?}", domains);
        
        Ok(Self {
            domains: ArcSwap::new(Arc::new(domains))
        })
    }

    /// New empty blocklist for testing 
    pub fn new_empty() -> Self {
        Self { 
            domains: ArcSwap::new(Arc::new(HashSet::new())) 
        }
    }

    /// Check if domain is blocked. This will also check for subdomains
    pub fn is_blocked(&self, domain: &str) -> bool {
        let domains = self.domains.load();
        let mut domain = domain.to_lowercase();


        // remove trailing dot if present (FQDN format)
        if domain.ends_with('.'){
            domain.pop(); // remove the trailing dot
        }

        // Check exact match first
        if domains.contains(&domain) {
            return true;
        }

        // Check subdomain matches by splitting the domain and checking each part
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 1..parts.len() {
            let parent_domain = parts[i..].join(".");
            if domains.contains(&parent_domain) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocklist_loading() {
        let blocklist = Blocklist::load("./blocklist.txt").unwrap();
        // Should contain domains from our test blocklist
        assert!(blocklist.is_blocked("facebook.com"));
        assert!(blocklist.is_blocked("youtube.com"));
    }

    #[test]
    fn test_subdomain_matching() {
        let mut domains = HashSet::new();
        domains.insert("example.com".to_string());
        let blocklist = Blocklist {
            domains: ArcSwap::new(Arc::new(domains)),
        };

        assert!(blocklist.is_blocked("example.com"));
        assert!(blocklist.is_blocked("www.example.com"));
        assert!(blocklist.is_blocked("api.sub.example.com"));
        assert!(!blocklist.is_blocked("example.org"));
        assert!(!blocklist.is_blocked("com"));
    }
}