use crate::resources::traits::{ProcessResource};
use config_client::protos::k8s::io::api::core::v1;

impl ProcessResource for v1::Namespace {
    fn get(&self) -> String { "RoutingInstance".to_string() }
}