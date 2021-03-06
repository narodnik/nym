use addressing;
use sphinx::route::NodeAddressBytes;
use std::error::Error;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug)]
pub struct MixPeer {
    connection: SocketAddr,
}

impl MixPeer {
    // note that very soon `next_hop_address` will be changed to `next_hop_metadata`
    pub fn new(next_hop_address: NodeAddressBytes) -> MixPeer {
        let next_hop_socket_address =
            addressing::socket_address_from_encoded_bytes(next_hop_address.to_bytes());
        MixPeer {
            connection: next_hop_socket_address,
        }
    }

    pub async fn send(&self, bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let next_hop_address = self.connection.clone();
        let mut stream = tokio::net::TcpStream::connect(next_hop_address).await?;
        stream.write_all(&bytes).await?;
        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.connection.to_string()
    }
}
