use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::VecDeque;
use std::sync::{Arc};
use tokio::sync::Mutex;
use crossbeam_channel::TryRecvError;
use std::vec::Vec;
use async_trait::async_trait;

#[async_trait]
pub trait ResourceController {
    async fn process_resource(&mut self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);
}

pub async fn queue_watcher(channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>, resource_controller: Arc<Mutex<Box<dyn ResourceController + Send + Sync>>>) {
    println!("starting queue_watcher");
    let receiver_clone = receiver.clone();
    let client = ConfigControllerClient::new(channel.clone());
    let resource_queue: VecDeque<v1::Resource> = VecDeque::new();
    let resource_queue_mutex: Arc<Mutex<VecDeque<v1::Resource>>> = Arc::new(Mutex::new(resource_queue));
    tokio::spawn(async move {
        let worker_queue: Vec<v1::Resource> = Vec::new();
        let worker_queue_mutex = Arc::new(Mutex::new(worker_queue));

        loop{
            match receiver_clone.try_recv() {
                Ok(resource) => { 
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    if !resource_queue_lock.contains(&resource){
                        resource_queue_lock.push_back(resource);
                    }
                },
                Err(TryRecvError::Empty) => {
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    let worker_queue_mutex_clone = worker_queue_mutex.clone();
                    if !resource_queue_lock.is_empty(){
                        let resource = resource_queue_lock.pop_front().unwrap();
                        let mut worker_queue_lock = worker_queue_mutex.lock().await;
                        if !worker_queue_lock.contains(&resource){
                            worker_queue_lock.push(resource.clone());
                            let mut client = client.clone();
                            let mut resource_controller_lock = resource_controller.lock().await;
                            resource_controller_lock.process_resource(&mut client, resource.clone(), worker_queue_mutex_clone).await;
                        } else {
                            if !resource_queue_lock.contains(&resource){
                                resource_queue_lock.push_back(resource.clone());
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                },
                _ => { continue; },
            }
        }
    });
}