use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::{Arc};
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;


pub trait ProcessResource {
    fn process_resource(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);
    fn get_resource(&self, resource: v1::Resource) -> Box<dyn ProcessResource>{
        match resource.kind.as_str() {
            "VirtualNetwork" => Box::new(v1alpha1::VirtualNetwork::default()),
            _ => Box::new(v1alpha1::VirtualNetwork::default()),
            /* 
            v1::resource::Resource::VirtualNetwork(res) => Box::new(res),
            v1::resource::Resource::VirtualMachineInterface(res) =>  Box::new(res),
            v1::resource::Resource::VirtualRouter(res) =>  Box::new(res),
            v1::resource::Resource::InstanceIp(res) => Box::new(res),
            v1::resource::Resource::VirtualMachine(res) => Box::new(res),
            v1::resource::Resource::RoutingInstance(res) => Box::new(res),
            v1::resource::Resource::Namespace(res) => Box::new(res),
            */
        }
    }
}