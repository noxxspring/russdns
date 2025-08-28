//! The DNS server implementation.

use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use trust_dns_proto::op::{Message, Query};
use trust_dns_proto::serialize::binary::{BinDecodable, BinDecoder, BinEncodable, BinEncoder, EncodeMode};
use crate::resolver::Resolver;


/// Starts the DNS server on the specified address.
#[instrument(skip(resolver))]
pub async fn start_server(listen_addr: SocketAddr, mut resolver: Resolver) -> anyhow::Result<()> {

    //Bind to UDP socket    
    let socket = UdpSocket::bind(listen_addr).await?;
    let socket = Arc::new(socket);
    info!("Server listening on {}", listen_addr);

    let mut buf = [0; 512]; // Standard DNS packet size

    loop {
        // Wait for DNS query to arrive 
        match socket.recv_from(&mut buf).await {
            Ok((size, src_addr)) => {
                debug!("Received {} bytes from {}", size, src_addr);

                // Clone the Arc for the spawned task
                let socket_clone = Arc::clone(&socket);
                let data = buf[..size].to_vec(); // copy the data to the task


                // Handle the query in a separate async task
                tokio::spawn(async move {
                    if let Err(e) = handle_query(socket_clone, &data, src_addr).await {
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

///Handle Individula DNS query
#[instrument(skip(socket, data))]
async fn handle_query(socket: Arc<UdpSocket>, data: &[u8], src_addr: SocketAddr) -> anyhow::Result<()> {
    // Parse the DNS message
    let message = match Message::from_bytes(data) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to parse DNS message from {}: {}",src_addr, e);
            return Ok(());
        }
    };

    // Extract the first query (most DNS query has only one)
    if let Some(query) = message.queries().first() {
        let query_name = query.name().to_utf8();
        info!(
            "DNS Query from {}: {} (type: {:?})",
            src_addr, query_name, query.query_type()
        );

         // TODO: Step 3 - Check blocklist and resolve properly
         // lets send a simple message
         let response = create_simple_response(&message, query)?;

         // Send the response back to the client
         send_response(socket, &response, src_addr).await?;

    } else {
        debug!("Received message with no queries from {}", src_addr);
    }
    Ok(())
}

/// create a simple response for the query (placeholder for now)
fn create_simple_response(message: &Message, query: &Query) -> anyhow::Result<Vec<u8>> {
    let mut response = Message::new();

    // Set the response header
    response.set_id(message.id());
    response.set_message_type(trust_dns_proto::op::MessageType::Response);
    response.set_op_code(message.op_code());
    response.set_authoritative(message.authoritative());
    response.set_recursion_desired(message.recursion_desired());
    response.set_recursion_available(true);
    response.set_response_code(trust_dns_proto::op::ResponseCode::NoError);

    // Add the original query
    response.add_query(query.clone());


    //TODO: add proper answer based on bloacklist/resolution


    // Serialize the response to bytes
    let mut buf = Vec::with_capacity(512);
    let mut encoder = BinEncoder::new(&mut buf);
    response.emit(&mut encoder)?;

    Ok(buf)

}

/// Send a DNS response back to the client
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