use addressing;
use sphinx::route::{Destination, DestinationAddressBytes, SURBIdentifier};
use sphinx::SphinxPacket;
use std::net::SocketAddr;
use topology::NymTopology;

pub const LOOP_COVER_MESSAGE_PAYLOAD: &[u8] = b"The cake is a lie!";

pub fn loop_cover_message<T: NymTopology>(
    our_address: DestinationAddressBytes,
    surb_id: SURBIdentifier,
    topology: &T,
) -> (SocketAddr, SphinxPacket) {
    let destination = Destination::new(our_address, surb_id);

    encapsulate_message(destination, LOOP_COVER_MESSAGE_PAYLOAD.to_vec(), topology)
}

pub fn encapsulate_message<T: NymTopology>(
    recipient: Destination,
    message: Vec<u8>,
    topology: &T,
) -> (SocketAddr, SphinxPacket) {
    let mut providers = topology.get_mix_provider_nodes();
    let provider = providers.pop().unwrap().into();

    let route = topology.route_to(provider).unwrap();

    // Set average packet delay to an arbitrary but at least not super-slow value for testing.
    let average_delay = 0.1;
    let delays = sphinx::header::delays::generate(route.len(), average_delay);

    // build the packet
    let packet = sphinx::SphinxPacket::new(message, &route[..], &recipient, &delays).unwrap();

    let first_node_address =
        addressing::socket_address_from_encoded_bytes(route.first().unwrap().address.to_bytes());

    (first_node_address, packet)
}
