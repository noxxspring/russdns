//! The DNS server implementation.

use crate::resolver::Resolver;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, instrument};
use trust_dns_proto::{op::Message, serialize::binary::BinDecodable};

/// Starts the DNS server on the specified address.
#[instrument(skip(resolver))]
pub async fn start_server(listen_addr: SocketAddr, resolver: Resolver) -> anyhow::Result<()> {
    // Bind to the UDP socket and wrap it in an Arc for shared ownership
    let socket = UdpSocket::bind(listen_addr).await?;
    let socket = Arc::new(socket);
    info!("Server listening on UDP {}", listen_addr);

    let mut buf = [0; 512]; // Standard DNS packet size

    loop {
        // Wait for a DNS query to arrive
        match socket.recv_from(&mut buf).await {
            Ok((size, src_addr)) => {
                debug!("Received {} bytes from {}", size, src_addr);
                
                // Clone the Arc and Resolver for the spawned task
                let socket_clone = Arc::clone(&socket);
                let data = buf[..size].to_vec(); // Copy the data for the task
                let resolver_clone = resolver.clone(); // Clone the resolver
                
                // Handle the query in a separate async task
                tokio::spawn(async move {
                    if let Err(e) = handle_query(socket_clone, &data, src_addr, resolver_clone).await {
                        error!("Failed to handle query from {}: {}", src_addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to receive data: {}", e);
            }
        }
    }
}

/// Handles an individual DNS query
#[instrument(skip(socket, data, resolver))]
async fn handle_query(
    socket: Arc<UdpSocket>,
    data: &[u8],
    src_addr: SocketAddr,
    mut resolver: Resolver, // Now takes ownership of the cloned resolver
) -> anyhow::Result<()> {
    // Parse the DNS message to get the query name for logging
    if let Ok(message) = Message::from_bytes(data) {
        if let Some(query) = message.queries().first() {
            let query_name = query.name().to_utf8();
            info!(
                "DNS Query from {}: {} (type: {:?})",
                src_addr, query_name, query.query_type()
            );
        }
    }

    // Use the resolver to handle the query
    let response = resolver.resolve(data).await?;
    
    // Send the response back to the client
    send_response(socket, &response, src_addr).await?;

    Ok(())
}

/// Sends a DNS response back to the client
async fn send_response(socket: Arc<UdpSocket>, response: &[u8], dst_addr: SocketAddr) -> anyhow::Result<()> {
    match socket.send_to(response, dst_addr).await {
        Ok(sent) => {
            debug!("Sent {} bytes to {}", sent, dst_addr);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send response to {}: {}", dst_addr, e);
            Err(e.into())
        }
    }
}