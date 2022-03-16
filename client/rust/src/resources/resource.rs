use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use tonic::transport::Channel;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::error::Error;
use tokio::sync::mpsc::{Sender, Receiver};
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::queue::queue::ResourceQueue;
use core::task::Poll;
use core::task::{Context, Waker};
use std::{thread, time};
use futures;
use futures::executor;
use std::collections::VecDeque;
use std::sync::{Arc};
use tokio::sync::Mutex;
use crossbeam_channel::{unbounded, bounded, TryRecvError};
use std::collections::HashMap;
use std::vec::Vec;




pub struct ResourceController {
    receiver: crossbeam_channel::Receiver<v1::Resource>,
    channel: tonic::transport::Channel
}

impl ResourceController {
    pub fn new(channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>) -> Self {
        Self{
            channel: channel,
            receiver: receiver,
        }
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("starting virtual_network_controller");
    
        let queue_watcher_thread = self.queue_watcher();
        futures::join!(queue_watcher_thread);
        Ok(())
    }
    pub async fn queue_watcher(&mut self) {
        println!("starting queue_watcher");
        let receiver_clone = self.receiver.clone();
        let client = ConfigControllerClient::new(self.channel.clone());
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
                                process_resource(&mut client, resource.clone(), worker_queue_mutex_clone).await;
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
}

pub async fn process_resource(client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>){
    
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
    //result_sender.send(resource.clone()).await;
}