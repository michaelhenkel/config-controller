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
use tokio::sync::mpsc::error::TryRecvError;
use core::task::{Context, Waker};
use std::{thread, time};
use futures;
use futures::executor;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

impl ProcessResource for v1alpha1::VirtualNetwork {
    fn get(&self, client: &mut ConfigControllerClient<Channel>) -> String { 
        "VirtualNetwork".to_string() 
    }
}


pub struct VirtualNetworkController {
    receiver: Receiver<v1::Resource>,
    channel: tonic::transport::Channel
}

impl VirtualNetworkController {
    pub fn new(channel: tonic::transport::Channel, receiver: Receiver<v1::Resource>) -> VirtualNetworkController {
        VirtualNetworkController{
            channel: channel,
            receiver: receiver,
        }
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("starting virtual_network_controller");
        let queue_watcher_thread = queue_watcher(&mut self.receiver);
        futures::join!(queue_watcher_thread);
        Ok(())
    }
}

pub async fn queue_watcher(receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting queue_watcher");

    let resource_queue: VecDeque<v1::Resource> = VecDeque::new();
    let resource_queue_mutex = Arc::new(Mutex::new(resource_queue));
    let resource_queue_mutex_clone = Arc::clone(&resource_queue_mutex);
    
    let worker_queue: VecDeque<v1::Resource> = VecDeque::new();
    let worker_queue_mutex = Arc::new(Mutex::new(worker_queue));
    
    tokio::spawn(async move {
        loop {
           
            let mut resource_queue_lock = resource_queue_mutex_clone.lock().await;
            let worker_queue_lock = worker_queue_mutex.lock().await;
            if resource_queue_lock.is_empty() {
                continue;
            }
            let resource = resource_queue_lock.pop_front().unwrap();
            println!("sending resource {:?} to consumer", resource.clone()); 

            
            if worker_queue_lock.len() < 2 && !worker_queue_lock.contains(&resource.clone()) {
                let worker_queue_mutex_clone = Arc::clone(&resource_queue_mutex_clone);
                
                tokio::spawn(async move {
                    let mut worker_queue_mutex_clone_lock = worker_queue_mutex_clone.lock().await;
                    worker_queue_mutex_clone_lock.push_back(resource.clone());
                    let mut cloned_resource = resource.clone();
                    let result = resource_consumer(&mut cloned_resource);
                    worker_queue_mutex_clone_lock.pop_front();
                    println!("done");
                });
                
                
            } else {
                println!("worker queue full or resource is currently processed, pushing back");
                resource_queue_lock.push_back(resource.clone());
            }
            //consumer_sender_clone.send(resource.clone()).await;
        }
    });

    loop {
        let resource = receiver.recv().await.unwrap();
        println!("received resource {:?}", resource);
        let resource_queue_mutex_clone = Arc::clone(&resource_queue_mutex);
        let mut lock = resource_queue_mutex_clone.lock().await;
        println!("got lock for resource {:?}", resource);
        if !lock.contains(&resource) {
            println!("resource {:?} not in queue, adding it", resource);
            lock.push_back(resource.clone());
        }
    }
}

pub fn resource_consumer(resource: &mut v1::Resource) -> Result<(), Box<dyn Error>> {
    println!("consuming {:?} for 30 seconds", resource.clone());
    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}