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
use std::collections::HashMap;
use std::vec::Vec;


impl ProcessResource for v1alpha1::VirtualNetwork {
    fn get(&self, client: &mut ConfigControllerClient<Channel>) -> String { 
        "VirtualNetwork".to_string() 
    }
}


pub struct VirtualNetworkController {
    receiver: tokio::sync::mpsc::Receiver<v1::Resource>,
    channel: tonic::transport::Channel,
    //client: ConfigControllerClient<tonic::transport::Channel>,
}

impl VirtualNetworkController {
    pub fn new(channel: tonic::transport::Channel, receiver: tokio::sync::mpsc::Receiver<v1::Resource>) -> Self {
        //let client = ConfigControllerClient::new(channel.clone());
        Self{
            receiver: receiver,
            //client: client,
            channel: channel,
        }
    }
    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        println!("starting virtual_network_controller");

        tokio::spawn(async move {
            let _queue_watcher_thread = self.queue_watcher().await;
        });

        Ok(())
    }

    pub async fn queue_watcher(&mut self) {
        println!("starting queue_watcher");
        let worker_queue: Vec<v1::Resource> = Vec::new();
        let worker_queue_mutex = Arc::new(Mutex::new(worker_queue));
        let resource_queue: VecDeque<v1::Resource> = VecDeque::new();
        let resource_queue_mutex = Arc::new(Mutex::new(resource_queue));
        let resource_queue_lock_clone = resource_queue_mutex.clone();
        let client = ConfigControllerClient::new(self.channel.clone());
        let worker_queue_mutex_clone = worker_queue_mutex.clone();
        let client_clone = client.clone();
        let (result_sender, mut result_receiver): (tokio::sync::mpsc::Sender<v1::Resource>,tokio::sync::mpsc::Receiver<v1::Resource>) = tokio::sync::mpsc::channel(100);
        let result_sender_clone = result_sender.clone();
        tokio::spawn(async move {
            loop {
                let result = result_receiver.recv().await.unwrap();
                println!("got result {:?}", result);
                let mut resource_queue_lock = resource_queue_lock_clone.lock().await;
                if !resource_queue_lock.is_empty(){
                    let resource = resource_queue_lock.pop_front().unwrap();
                    process(worker_queue_mutex_clone.clone(), resource.clone(), client_clone.clone(), resource_queue_lock_clone.clone(), result_sender_clone.clone()).await;
                } else {
                    println!("resource queue empty, no further processing");
                }
            }

        });
        loop {
            let resource = self.receiver.recv().await.unwrap();
            process(worker_queue_mutex.clone(), resource.clone(), client.clone(), resource_queue_mutex.clone(), result_sender.clone()).await;

        }
        
    }
}

pub async fn process(worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>, resource: v1::Resource, client: ConfigControllerClient<tonic::transport::Channel>, resource_queue_mutex: Arc<Mutex<VecDeque<v1::Resource>>>, result_sender: tokio::sync::mpsc::Sender<v1::Resource> ) {
    let mut worker_queue_lock = worker_queue_mutex.lock().await;
    let worker_queue_clone = worker_queue_mutex.clone();
    let resource_queue_clone = resource_queue_mutex.clone();
    if !worker_queue_lock.contains(&resource){
        worker_queue_lock.push(resource.clone());
        println!("got {:?}", resource);
        let mut client = client.clone();
        tokio::spawn(async move {
            get_resource(&mut client, resource.clone(), result_sender).await;
            let mut worker_queue_lock = worker_queue_clone.lock().await;
            worker_queue_lock.retain(|x| *x != resource.clone());
        });
    } else {
        println!("resource {:?} already processed, trying to add it to resource queue", resource.clone());
        let mut resource_queue_lock = resource_queue_clone.lock().await;
        if !resource_queue_lock.contains(&resource){
            println!("resource {:?} not in resource queue, adding it", resource.clone());
            resource_queue_lock.push_back(resource.clone());
        } else {
            println!("resource {:?} already in resource queue, skipping", resource.clone());
        }
    }
}

/*
pub async fn process2(worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>, resource: v1::Resource, client: ConfigControllerClient<tonic::transport::Channel>, resource_queue_mutex: Arc<Mutex<VecDeque<v1::Resource>>> ) {
    let mut worker_queue_lock = worker_queue_mutex.lock().await;
    let worker_queue_clone = worker_queue_mutex.clone();
    let resource_queue_clone = resource_queue_mutex.clone();
    if !worker_queue_lock.contains(&resource){
        worker_queue_lock.push(resource.clone());
        println!("got {:?}", resource);
        let mut client = client.clone();
        tokio::spawn(async move {
            get_resource(&mut client, resource.clone()).await;
            let mut worker_queue_lock = worker_queue_clone.lock().await;
            worker_queue_lock.retain(|x| *x != resource.clone());
        });
    } else {
        println!("resource {:?} already processed, trying to add it to resource queue", resource.clone());
        let mut resource_queue_lock = resource_queue_clone.lock().await;
        if !resource_queue_lock.contains(&resource){
            println!("resource {:?} not in resource queue, adding it", resource.clone());
            resource_queue_lock.push_back(resource.clone());
        } else {
            println!("resource {:?} already in resource queue, skipping", resource.clone());
        }
    }
}
*/

pub async fn get_resource(client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, result_sender: tokio::sync::mpsc::Sender<v1::Resource>){
    let mut client = client.clone();
    let res_result: Result<tonic::Response<v1alpha1::VirtualNetwork>, tonic::Status> = client.get_virtual_network(resource.clone()).await;
    let res_resp: &mut tonic::Response<v1alpha1::VirtualNetwork> = &mut res_result.unwrap();
    let res: &mut v1alpha1::VirtualNetwork = res_resp.get_mut();
    println!("{}/{}", res.metadata.as_ref().unwrap().namespace(), res.metadata.as_ref().unwrap().name());
    println!("labels {:?}", res.metadata.as_ref().unwrap().labels);
    println!("sleeping for 30 sec");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    println!("done");
    result_sender.send(resource.clone()).await;

}

/*
    pub async fn queue_watcher(&mut self) {
        println!("starting queue_watcher");
        let client = ConfigControllerClient::new(self.channel.clone());
        let receiver = self.receiver.clone();
        let mut resource_queue: VecDeque<v1::Resource> = VecDeque::new();    
        let _guard = tokio::spawn(async move {
            let worker_queue: Vec<v1::Resource> = Vec::new();
            let worker_queue_mutex = Arc::new(Mutex::new(worker_queue));
            loop{
                match receiver.try_recv() {
                    Ok(resource) => { 
                        println!("received resource 2 {:?}", resource);
                        if !resource_queue.contains(&resource){
                            resource_queue.push_back(resource);
                        }
                    },
                    Err(TryRecvError::Empty) => {
                        if !resource_queue.is_empty(){
                            let resource = resource_queue.pop_front().unwrap();
                            let worker_queue_clone = worker_queue_mutex.clone();
                            let mut worker_queue_lock = worker_queue_mutex.lock().unwrap();
                            if !worker_queue_lock.contains(&resource){
                                worker_queue_lock.push(resource.clone());
                                println!("found resource {:?} in queue", resource.clone());
                                let mut client = client.clone();
                                let _join_handler = tokio::spawn(async move {
                                    println!("consuming {:?} for 30 seconds", resource.clone());
                                    /*
                                    let result = get_resource(&mut client, resource.clone()).await;
                                    thread::sleep(time::Duration::from_secs(30));
                                    println!("done");
                                    let mut worker_queue_lock = worker_queue_clone.lock().unwrap();
                                    worker_queue_lock.retain(|x| *x != resource.clone());
                                    */
                                });
                            } else {
                                if !resource_queue.contains(&resource){
                                    resource_queue.push_back(resource);
                                }
                                thread::sleep(time::Duration::from_millis(1));
                            }
                        } else {
                            thread::sleep(time::Duration::from_millis(1));
                            continue;
                        }
                    },
                    _ => { continue; },
                }
            }
        });
    }
*/