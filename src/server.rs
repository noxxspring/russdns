//! The DNS server implementation.

use tokio::net::UdpSocket;

use crate::resolver::Resolver;



/// Starts the DNS server on the specified address.
pub async fn start_server(listen_addr: &str, mut resolver: Resolver) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(listen_addr).await?;
    println!("Server listening on {}", listen_addr);

    let mut buf = [0; 512]; // Standard DNS packet size

    loop {
        // Wait for DNS query to arrive 
        let (size, src_addr) = socket.recv_from(&mut buf).await?;
        println!("Received query from {}", src_addr);

        // TODO: Handle the query by passing it to the resolver
        // let response = resolver.resolve(&buf[..size]).await?;

        // TODO: Send the response back to the client
        // socket.send_to(&response, src_addr).await?;
    }
}