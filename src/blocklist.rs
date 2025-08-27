//! Blocklist management for the DNS filter.

use std::{collections::HashSet, fs::File, io::{BufRead, BufReader}, sync::Arc};

use arc_swap::ArcSwap;



/// A thread-safe, reloadable list of blocked domains.
pub struct Blocklist {
    // ArcSwap allows us to atomically update the entire list without locking
    domains: ArcSwap<HashSet<String>>, 
}

impl Blocklist {
    /// Load the blocklist from the file, with one domain per line
    pub fn load(path: &str) -> anyhow::Result<Self>{
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut domains = HashSet::new();

        for line in reader.lines(){
            let line = line?;
            let domain = line.trim();
            //Skip empty lines and comments
            if domain.is_empty() || domain.starts_with('#') {
                continue;
            }
            domains.insert(domain.to_lowercase());
        }
        
        Ok(Self {
            domains: ArcSwap::new(Arc::new(domains))
        })
    }

    /// Check if domain is blocked. this will also check for subdomains
    pub fn is_blocked(&self, domain: &str) -> bool {
        let domains = self.domains.load();
        // TODO: Implement subdomain matching logic
        // For now, just do a simple exact match.
        domains.contains(&domain.to_lowercase())
    }
}