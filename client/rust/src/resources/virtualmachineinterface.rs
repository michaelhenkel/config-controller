use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::resources::resource::{ResourceController, queue_watcher};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use std::error::Error;
use async_trait::async_trait;

#[derive(Copy,Clone)]
pub struct VirtualMachineInterfaceController {}

impl VirtualMachineInterfaceController {
    pub fn new() -> Self {
        Self{}
    }
}

#[async_trait]
impl ResourceController for VirtualMachineInterfaceController{
    fn to_trait(self) -> Box<dyn ResourceController + Send + Sync> {
        Box::new(self)
    }
    async fn process_resource(&mut self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>){
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualMachineInterface>, tonic::Status> = client.get_virtual_machine_interface(resource.clone()).await;
            let res_resp: &mut tonic::Response<v1alpha1::VirtualMachineInterface> = &mut res_result.unwrap();
            let res: &mut v1alpha1::VirtualMachineInterface = res_resp.get_mut();
            println!("##########VirtualMachineInterface##########");
            println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
            println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
            //println!("sleeping for 1 sec");
            //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let mut worker_queue_lock = worker_queue_mutex.lock().await;
            worker_queue_lock.retain(|x| *x != resource.clone());
            println!("done");
        });
    }
    async fn run(&mut self, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
        println!("starting virtual_machine_interface_controller");
        let res = self.to_trait();
        let self_mutex = Arc::new(Mutex::new(res));
        let queue_watcher_thread = queue_watcher(channel, receiver, self_mutex);
        futures::join!(queue_watcher_thread);
        Ok(())
    }
}
