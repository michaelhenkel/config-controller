use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::resources::resource::{ResourceInterface};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;

#[derive(Copy,Clone)]
pub struct VirtualMachineInterfaceController {}

impl VirtualMachineInterfaceController {
    pub fn new() -> Self {
        Self{}
    }
}

impl ResourceInterface for VirtualMachineInterfaceController{
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>) {
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualMachineInterface>, tonic::Status> = client.get_virtual_machine_interface(resource.clone()).await;
            match res_result {
                Ok(mut res) => {
                    //let res_resp: &mut tonic::Response<v1alpha1::VirtualMachineInterface> = &mut resource.unwrap();
                    let res: &mut v1alpha1::VirtualMachineInterface = res.get_mut();
                    println!("##########VirtualMachineInterface##########");
                    println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
                    println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
                    let mut worker_queue_lock = worker_queue_mutex.lock().await;
                    worker_queue_lock.retain(|x| *x != resource.clone());
                    println!("done");
                },
                Err(err) => {
                    println!("err {:?}", err);
                },
            }

        });
    }
}

