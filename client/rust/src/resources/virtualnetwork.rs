use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::resources::resource::{ResourceInterface};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use std::error::Error;

#[derive(Copy,Clone)]
pub struct VirtualNetworkController {}

impl VirtualNetworkController {
    pub fn new() -> Self {
        Self{}
    }
}

impl ResourceInterface for VirtualNetworkController{
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>) -> Result<(), Box<dyn Error + Send >> {
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualNetwork>, tonic::Status> = client.get_virtual_network(resource.clone()).await;
            match res_result {
                Ok(mut res) => {
                    let res: &mut v1alpha1::VirtualNetwork = res.get_mut();
                    println!("##########VirtualNetwork##########");
                    println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
                    println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
                    let mut worker_queue_lock = worker_queue_mutex.lock().await;
                    worker_queue_lock.retain(|x| *x != resource.clone());
                    println!("done");
                },
                Err(err) => {
                    let mut worker_queue_lock = worker_queue_mutex.lock().await;
                    worker_queue_lock.retain(|x| *x != resource.clone());
                    println!("err {:?}", err);
                },
            }
        });
        Ok(())
    }
}
