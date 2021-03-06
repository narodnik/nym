use crate::result::HealthCheckResult;
use directory_client::requests::presence_topology_get::PresenceTopologyGetRequester;
use directory_client::DirectoryClient;
use log::{debug, error, info, trace};
use std::fmt::{Error, Formatter};
use std::time::Duration;
use topology::NymTopologyError;

pub mod config;
mod path_check;
mod result;
mod score;

#[derive(Debug)]
pub enum HealthCheckerError {
    FailedToObtainTopologyError,
    InvalidTopologyError,
}

// required by std::error::Error
impl std::fmt::Display for HealthCheckerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // just have implementation equivalent to derived debug
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HealthCheckerError {}

impl From<topology::NymTopologyError> for HealthCheckerError {
    fn from(_: NymTopologyError) -> Self {
        use HealthCheckerError::*;
        InvalidTopologyError
    }
}

pub struct HealthChecker {
    directory_client: directory_client::Client,
    interval: Duration,
    num_test_packets: usize,
    resolution_timeout: Duration,
}

impl HealthChecker {
    pub fn new(config: config::HealthCheck) -> Self {
        debug!(
            "healthcheck will be using the following directory server: {:?}",
            config.directory_server
        );
        let directory_client_config = directory_client::Config::new(config.directory_server);
        HealthChecker {
            directory_client: directory_client::Client::new(directory_client_config),
            interval: Duration::from_secs_f64(config.interval),
            resolution_timeout: Duration::from_secs_f64(config.resolution_timeout),
            num_test_packets: config.num_test_packets,
        }
    }

    pub async fn do_check(&self) -> Result<HealthCheckResult, HealthCheckerError> {
        trace!("going to perform a healthcheck!");
        let current_topology = match self.directory_client.presence_topology.get() {
            Ok(topology) => topology,
            Err(err) => {
                error!("failed to obtain topology - {:?}", err);
                return Err(HealthCheckerError::FailedToObtainTopologyError);
            }
        };
        trace!("current topology: {:?}", current_topology);

        let mut healthcheck_result = HealthCheckResult::calculate(
            current_topology,
            self.num_test_packets,
            self.resolution_timeout,
        )
        .await;
        healthcheck_result.sort_scores();
        Ok(healthcheck_result)
    }

    pub async fn run(self) -> Result<(), HealthCheckerError> {
        debug!(
            "healthcheck will run every {:?} and will send {} packets to each node",
            self.interval, self.num_test_packets
        );

        loop {
            match self.do_check().await {
                Ok(health) => info!("current network health: \n{}", health),
                Err(err) => error!("failed to perform healthcheck - {:?}", err),
            };

            tokio::time::delay_for(self.interval).await;
        }
    }
}
