use crate::node;
use directory_client::presence::MixNodePresence;
use directory_client::requests::presence_mixnodes_post::PresenceMixNodesPoster;
use directory_client::DirectoryClient;
use std::thread;
use std::time::Duration;

pub struct Notifier {
    pub net_client: directory_client::Client,
    presence: MixNodePresence,
}

impl Notifier {
    pub fn new(node_config: &node::Config) -> Notifier {
        let config = directory_client::Config {
            base_url: node_config.directory_server.clone(),
        };
        let net_client = directory_client::Client::new(config);
        let presence = MixNodePresence {
            host: node_config.socket_address.to_string(), // note: the directory server determines the real incoming IP itself, but uses the socket. Host here is just a placeholder.
            pub_key: node_config.public_key_string(),
            layer: node_config.layer as u64,
            last_seen: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        Notifier {
            net_client,
            presence,
        }
    }

    pub fn notify(&self) {
        self.net_client
            .presence_mix_nodes_post
            .post(&self.presence)
            .unwrap();
    }

    pub fn run(&self) {
        loop {
            self.notify();
            thread::sleep(Duration::from_secs(5));
        }
    }
}
