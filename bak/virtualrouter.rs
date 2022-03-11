use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;

impl ProcessResource for v1alpha1::VirtualRouter {
    fn get(&self) -> String { "VirtualRouter".to_string() }
}