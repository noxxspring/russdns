# RussDNS 

A high-performance, corporate-standard DNS filtering daemon built in Rust. RussDNS provides network-wide content filtering by blocking access to specified domains and their subdomains, logging all queries, and serving custom responses for blocked requests.

## Features

*   **DNS Filtering:** Block access to domains based on a customizable blocklist.
*   **Subdomain Blocking:** Blocking a domain (e.g., `facebook.com`) automatically blocks all its subdomains (e.g., `www.facebook.com`, `m.facebook.com`).
*   **Sinkholing:** Respond to blocked requests with a configurable IP address (e.g., `0.0.0.0` or a custom "Blocked" page).
*   **Detailed Logging:** Logs all DNS queries with client IP, timestamp, domain, and action taken (Allowed/Blocked) in structured JSON format.
*   **Performance:** Built on Tokio for async, high-concurrency handling of requests. Includes a DNS response cache to reduce latency and upstream traffic.
*   **Dynamic Configuration:** Reload the blocklist without restarting the daemon (Planned Feature).


## Configuration

RussDNS is configured via a `config.toml` file.

```toml
# The IP and port for russdns to listen on.
listen_addr = "0.0.0.0:53"

# The upstream DNS server to forward allowed requests to.
upstream_dns_addr = "1.1.1.1:53"

# The action to take for blocked requests: "Sinkhole" or "Nxdomain"
block_action = "Sinkhole"

# The IP address to return for blocked requests if action is "Sinkhole".
sinkhole_ip = "0.0.0.0"

# Path to the file containing blocked domains, one per line.
blocklist_file = "./blocklist.txt"

# Path to the log file.
log_file = "./russdns.log"

# Log level: trace, debug, info, warn, error
log_level = "info"