//! The core DNS resolution logic.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, info, instrument};
use trust_dns_proto::op::{Message, Query, ResponseCode};
use trust_dns_proto::rr::record_type::RecordType;
use trust_dns_proto::rr::{ RData, Record};
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable, BinEncoder};


use crate::config::BlockAction;
use crate::{blocklist::Blocklist, cache::DnsCache};


/// Handles the core logic of resolving DNS queries, including
/// checking the blocklist, cache, and forwarding to upstream.

#[derive(Clone)]
pub struct Resolver {
    upstream_dns_addr: SocketAddr,
    cache: DnsCache,
    blocklist: Blocklist,
    block_action: BlockAction,
    sinkhole_ip: String,
}

impl Resolver {
    /// create a new resolver
    pub fn new(upstream_dns_addr: SocketAddr, cache: DnsCache, blocklist: Blocklist, block_action: BlockAction, sinkhole_ip: String) -> Self {
        Self {
            upstream_dns_addr,
            cache,
            blocklist,
            block_action,
            sinkhole_ip,
        }
        
    }
    
    /// Resolves a DNS query by checking blocklist, cache or forwarding upstream
    #[instrument(skip(self, query))]
    pub async fn resolve(&mut self, query: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Parse the incoming query to check if it's blocked
        let request = match Message::from_bytes(query) {
            Ok(msg) => msg,
            Err(e) => {
                // If we can't parse it, just forward it upstream
                debug!("Failed to parse query, forwarding upstream: {}", e);
                return self.forward_to_upstream(query).await;
            }
        };


       if let Some(query_obj) = request.queries().first() {
        let query_name = query_obj.name().to_utf8();

        //DEBUG: Log whether domain is blocked
        debug!("Checking if '{}' is blocked", query_name);
        debug!("Blocklist contains facebook.com: {}", self.blocklist.is_blocked("facebook.com"));
        debug!("Blocklist contains facebook.com.: {}", self.blocklist.is_blocked("facebook.com."));
        debug!("Is '{}' blocked: {}", query_name, self.blocklist.is_blocked(&query_name));
    

        // Check if the domain is blocked
        if self.blocklist.is_blocked(&query_name) {
            info!("Blocking query for: {}", query_obj);
            return self.create_blocked_response(&request, query_obj).await;
        }else {
            debug!("Domain '{}' is NOT blocked, forwarding", query_name);
        }

        //TODO: Check cache 
        // for now forwardto upstream DNS
        debug!("Forwarding query for: {}", query_name);
        return self.forward_to_upstream(query).await
       }

       // If no queries, return error response
       self.create_error_response(&request, ResponseCode::FormErr).await
    }


    /// create a response for blocked domains based on cnfigured action
    #[instrument(skip(self, request, query))]
    async fn create_blocked_response(
        &self,
        request: &Message,
        query: &Query,
    ) -> anyhow::Result<Vec<u8>>{
        let mut response = Message::new();

        // copy header fro request
         response.set_id(request.id());
        response.set_message_type(trust_dns_proto::op::MessageType::Response);
        response.set_op_code(request.op_code());
        response.set_recursion_desired(request.recursion_desired());
        response.set_recursion_available(true);
        response.add_query(query.clone());

        match self.block_action {
            BlockAction::Sinkhole => {
                response.set_response_code(ResponseCode::NoError);
                self.add_sinkhole_answer(&mut response, query)?;
            }
            BlockAction::Nxdomain => {
                response.set_response_code(ResponseCode::NXDomain);
            }
        }

        self.serialize_response(response)
    }

    /// Adds a sinkhole answer record to the response
    fn add_sinkhole_answer(&self, response: &mut Message, query: &Query) -> anyhow::Result<()> {
        if query.query_type() == RecordType::A {
            let name = query.name().clone();
            let sinkhole_ip = self.sinkhole_ip.parse()?;
            let record = Record::from_rdata(name, 60, // TTL of 60 seconds
                 RData::A(sinkhole_ip),
                );
                response.add_answer(record);
        }
        Ok(())
    }

    /// forward a query to the Upstream DNS server
   /// Forwards a query to the upstream DNS server.
#[instrument(skip(self, query))]
async fn forward_to_upstream(&self, query: &[u8]) -> anyhow::Result<Vec<u8>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(query, self.upstream_dns_addr).await?;

    let mut buf = [0; 512];
    let (size, _) = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for upstream DNS response"))??;

    Ok(buf[..size].to_vec())
}


    /// Create an error response
    async fn create_error_response(
        &self,
        request: &Message,
        response_code: ResponseCode,
    ) -> anyhow::Result<Vec<u8>>{
        let mut response = Message::new();
         response.set_id(request.id());
        response.set_message_type(trust_dns_proto::op::MessageType::Response);
        response.set_response_code(response_code);
        response.set_recursion_available(true);

        self.serialize_response(response)
    }

    /// Serialize a DNS message to bytes
    fn serialize_response(&self, response: Message) -> anyhow::Result<Vec<u8>>{
        let mut buf = Vec::with_capacity(512);
        let mut encoder = BinEncoder::new(&mut buf);
        response.emit(&mut encoder)?;
        Ok(buf)
    }

    }
