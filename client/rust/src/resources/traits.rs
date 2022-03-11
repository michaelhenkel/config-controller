use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use tonic::transport::Channel;

pub trait ProcessResource {
    fn get(&self, client: &mut ConfigControllerClient<Channel>) -> String;
}