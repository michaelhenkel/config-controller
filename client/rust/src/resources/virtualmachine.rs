use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::resources::resource::{ResourceInterface, ResourceInterface2};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use std::error::Error;

#[derive(Copy,Clone)]
pub struct VirtualMachineController {}

impl VirtualMachineController {
    pub fn new() -> Self {
        Self{}
    }
}

impl ResourceInterface for VirtualMachineController{
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>) -> Result<(), Box<dyn Error + Send >> {
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualMachine>, tonic::Status> = client.get_virtual_machine(resource.clone()).await;
            match res_result {
                Ok(mut res) => {
                    let res: &mut v1alpha1::VirtualMachine = res.get_mut();
                    println!("##########VirtualMachine##########");
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

impl ResourceInterface2 for VirtualMachineController{
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, sender: crossbeam_channel::Sender<v1::Resource>, resource: v1::Resource){
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualMachine>, tonic::Status> = client.get_virtual_machine(resource.clone()).await;
            match res_result {
                Ok(mut res) => {
                    let res: &mut v1alpha1::VirtualMachine = res.get_mut();
                    println!("##########VirtualMachine##########");
                    println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
                    println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
                    println!("done");
                    let resource = v1::Resource{
                        name: resource.name,
                        namespace: resource.namespace,
                        kind: resource.kind,
                        action: i32::from(v1::resource::Action::Del),
                    };
                    sender.send(resource).unwrap();
                },
                Err(err) => {
                    println!("err {:?}", err);
                    let resource = v1::Resource{
                        name: resource.name,
                        namespace: resource.namespace,
                        kind: resource.kind,
                        action: i32::from(v1::resource::Action::Retry),
                    };
                    sender.send(resource).unwrap();
                },
            }
        });
    }
}