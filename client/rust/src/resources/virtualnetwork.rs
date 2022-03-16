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


impl ProcessResource for v1alpha1::VirtualNetwork {
    fn get(&self, client: &mut ConfigControllerClient<Channel>) -> String { 
        "VirtualNetwork".to_string() 
    }
}


pub struct VirtualNetworkController {
    receiver: crossbeam_channel::Receiver<v1::Resource>,
    channel: tonic::transport::Channel
}

impl VirtualNetworkController {
    pub fn new(channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>) -> VirtualNetworkController {
        VirtualNetworkController{
            channel: channel,
            receiver: receiver,
        }
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("starting virtual_network_controller");
    
        let queue_watcher_thread = queue_watcher(&mut self.receiver, self.channel.clone());
        //let resource_consumer_thread = resource_consumer(&mut consumer_receiver);
        futures::join!(queue_watcher_thread);
        Ok(())
    }
}

pub async fn queue_watcher(receiver: &mut crossbeam_channel::Receiver<v1::Resource>, channel: tonic::transport::Channel) {
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
                    //println!("received resource 2 {:?}", resource);
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    if !resource_queue_lock.contains(&resource){
                        //println!("{:?} pushing to resource queue", resource.clone());
                        resource_queue_lock.push_back(resource);
                    }
                },
                Err(TryRecvError::Empty) => {
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    let worker_queue_mutex_clone = worker_queue_mutex.clone();
                    if !resource_queue_lock.is_empty(){
                        //println!("resource queue not empty");
                        let resource = resource_queue_lock.pop_front().unwrap();
                        //if resource_queue_lock.is_empty(){
                        //    println!("resource queue is now empty");
                        //}
                        //let resource = resource_queue_lock.get(0).unwrap();
                        let mut worker_queue_lock = worker_queue_mutex.lock().await;
                        if !worker_queue_lock.contains(&resource){
                            worker_queue_lock.push(resource.clone());
                            let mut client = client.clone();
                            get_resource(&mut client, resource.clone(), worker_queue_mutex_clone).await;
                        } else {
                            //println!("{:?} already in worker queue, trying to push it to resource queue", resource.clone());
                            if !resource_queue_lock.contains(&resource){
                                //println!("{:?} not in resource queue, pushing it", resource.clone());
                                resource_queue_lock.push_back(resource.clone());
                                //println!("{:?} pushed to resource queue", resource.clone());
                            } else {
                                //println!("{:?} already in resource queue, skipping", resource.clone());
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            //thread::sleep(time::Duration::from_millis(1));
                        }
                    } else {
                        //println!("resource queue empty");
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        //continue;
                    }
                },
                _ => { continue; },
            }
        }
    });
}

pub async fn test() {
    tokio::spawn(async move {
        println!("sleeping for 30 sec");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        println!("done");
    });
}


pub async fn get_resource(client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>){
    
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

pub async fn resource_consumer(receiver: &mut crossbeam_channel::Receiver<v1::Resource>) {
    //let mut join_handler_map: HashMap<v1::Resource,std::thread::JoinHandle<()>> = HashMap::new();
    //let mut join_vec: Vec<std::thread::JoinHandle<()> = Vec::new();
    let receiver_clone = receiver.clone();
    let _guard = thread::spawn(move || {
        loop {
            let resource = receiver_clone.recv().unwrap();
            let _join_handler = thread::spawn(move || {
                println!("consuming {:?} for 30 seconds", resource.clone());
                thread::sleep(time::Duration::from_secs(30));
                println!()
            });

        }
    });
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