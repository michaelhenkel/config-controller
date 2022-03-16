use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::{Arc};
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;

pub trait ProcessResource {
    fn process_resource(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);
}