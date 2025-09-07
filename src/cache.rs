//! DNS response caching module.

use std::num::NonZeroUsize;

use lru::LruCache;


/// A cache for DNS responses to improve performance.

pub struct DnsCache {
    // Using an LRU cache 
    cache: LruCache<String, Vec<u8>>,
}

impl Clone for DnsCache {
    fn clone(&self) -> Self {
        // Create a new empty cache with the same capacity
        // We don't want to clone the actual cache contents as that would be inefficient
        let cap = self.cache.cap();
        Self {
            cache: LruCache::new(cap),
        }
    }
}


impl DnsCache {
    /// Create a new DNS cache 
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap();
        Self {
            cache: LruCache::new(cap.into()),
        }
    }

    /// Retrieve a cached response from a query, if it exists
    pub fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        self.cache.get(key)
    }

    /// Store a respone in the cache for future queries
    pub fn put(&mut self, key: String, value: Vec<u8>) {
        self.cache.put(key, value);
    }
}