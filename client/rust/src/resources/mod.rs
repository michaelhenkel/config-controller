use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;


pub mod traits;
pub mod resource;
pub mod virtualnetwork;
/*
pub mod virtualrouter;
pub mod virtualmachine;
pub mod routinginstance;
pub mod instanceip;
pub mod namespace;
pub mod virtualmachineinterface;
*/

pub fn get_resource(resource: v1::Resource) -> Box<dyn traits::ProcessResource>{
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