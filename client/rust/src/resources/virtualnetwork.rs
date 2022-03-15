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
use std::sync::Arc;
use tokio::sync::Mutex;
use crossbeam_channel::{unbounded, bounded, TryRecvError};

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
        let (mut queue_sender, mut queue_receiver): (crossbeam_channel::Sender<v1::Resource>, crossbeam_channel::Receiver<v1::Resource>) = bounded(1);
        let channel_watcher_thread = channel_watcher(&mut self.receiver, &mut queue_sender);
        let queue_watcher_thread = queue_watcher(&mut queue_receiver);
        futures::join!(channel_watcher_thread, queue_watcher_thread);
        Ok(())
    }
}

pub async fn queue_watcher(receiver: &mut crossbeam_channel::Receiver<v1::Resource>) {
    println!("starting queue_watcher");
    let receiver_clone = receiver.clone();
    let mut resource_queue: VecDeque<v1::Resource> = VecDeque::new();
    let _guard = thread::spawn(move || {
        loop{
            match receiver_clone.try_recv() {
                Ok(resource) => { 
                    println!("received resource 2 {:?}", resource);
                    if !resource_queue.contains(&resource){
                        resource_queue.push_back(resource);
                    }
                },
                Err(TryRecvError::Empty) => {
                    if !resource_queue.is_empty(){
                        let mut resource = resource_queue.pop_front().unwrap();
                        println!("found resource {:?} in queue", resource);
                    } else {
                        continue;
                    }
                },
                _ => { continue; },
            }
            /*
            if let Ok(resource) =  receiver_clone.try_recv(){
                println!("received resource 2 {:?}", resource);
            }
            */
        }
    });
}
        /*
        match receiver.try_recv() {
            Ok(resource) => { 
                println!("received resource 2 {:?}", resource);
                if !resource_queue.contains(&resource){
                    resource_queue.push_back(resource);
                }
            },
            Err(TryRecvError::Empty) => {
                if !resource_queue.is_empty(){
                    let mut resource = resource_queue.pop_front().unwrap();
                    println!("found resource {:?} in queue", resource);
                } else {
                    continue;
                }
            },
            _ => { continue; },
        }
        */




pub async fn channel_watcher(receiver: &mut Receiver<v1::Resource>, queue_send: &mut crossbeam_channel::Sender<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting channel_watcher");

    let resource_queue: VecDeque<v1::Resource> = VecDeque::new();
    let resource_queue_mutex = Arc::new(Mutex::new(resource_queue));
    let resource_queue_mutex_clone = Arc::clone(&resource_queue_mutex);
    
    loop {
        let resource = receiver.recv().await.unwrap();
        println!("received resource 1 {:?}", resource);
        queue_send.send(resource).unwrap();
    
    }
}

pub fn resource_consumer(resource: &mut v1::Resource) -> Result<(), Box<dyn Error>> {
    println!("consuming {:?} for 30 seconds", resource.clone());
    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}

/*
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
    */