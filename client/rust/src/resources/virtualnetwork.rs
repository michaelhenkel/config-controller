use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use std::sync::{Arc};
use tokio::sync::Mutex;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;


impl ProcessResource for v1alpha1::VirtualNetwork {
    fn process_resource(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>) { 
        let mut client = client.clone();
        tokio::spawn(async move {
            let res_result: Result<tonic::Response<v1alpha1::VirtualNetwork>, tonic::Status> = client.get_virtual_network(resource.clone()).await;
            let res_resp: &mut tonic::Response<v1alpha1::VirtualNetwork> = &mut res_result.unwrap();
            let res: &mut v1alpha1::VirtualNetwork = res_resp.get_mut();
            println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
            println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
            println!("sleeping for 30 sec");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let mut worker_queue_lock = worker_queue_mutex.lock().await;
            worker_queue_lock.retain(|x| *x != resource.clone());
            println!("done");
        });
    }
}