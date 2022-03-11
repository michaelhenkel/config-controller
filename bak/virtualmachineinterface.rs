use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;

impl ProcessResource for v1alpha1::VirtualMachineInterface {
    fn get(&self) -> String { "VirtualMachineInterface".to_string() }
}