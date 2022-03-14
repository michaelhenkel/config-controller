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
        let (mut queue_sender, mut queue_receiver): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel(1);
        let (mut consumer_sender, mut consumer_receiver): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel(1);
        let channel_watcher_thread = channel_watcher(&mut queue_sender, &mut self.receiver);
        let queue_watcher_thread = queue_watcher(&mut queue_receiver, &mut consumer_sender);
        let resource_consumer_thread = resource_consumer(&mut consumer_receiver);
        futures::join!(channel_watcher_thread, queue_watcher_thread, resource_consumer_thread);
        Ok(())
    }
}

pub async fn channel_watcher(queue_sender: &mut Sender<v1::Resource>, receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting channel_watcher");
    loop {
        let resource = receiver.recv().await;
        println!("sending resource {:?} to queue", resource.clone());
        queue_sender.send(resource.unwrap().clone()).await;
    }
}

pub async fn queue_watcher(receiver: &mut Receiver<v1::Resource>, consumer_sender: &mut Sender<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting queue_watcher");

    let vec_deque: VecDeque<v1::Resource> = VecDeque::new();
    let mutex = Arc::new(Mutex::new(vec_deque));
    let mutex_clone = Arc::clone(&mutex);
    let consumer_sender_clone = consumer_sender.clone();
    tokio::spawn(async move {
        loop {
            let mut lock = mutex.lock().await;
            if lock.is_empty() {
                continue;
            }
            let resource = lock.pop_front().unwrap();
            println!("sending resource {:?} to consumer", resource.clone()); 
            consumer_sender_clone.send(resource.clone()).await;
        }
    });

    loop {
        let resource = receiver.recv().await.unwrap();
        let mut lock = mutex_clone.lock().await;
        if !lock.contains(&resource) {
            println!("resource {:?} not in queue, adding it", resource);
            lock.push_back(resource.clone());
        }
    }
}

pub async fn resource_consumer(receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    let vec_deque: VecDeque<v1::Resource> = VecDeque::new();
    let mutex = Arc::new(Mutex::new(vec_deque));
    
    loop {
        let resource = receiver.recv().await.unwrap();
        let mut lock = mutex.lock().await;
        if lock.len() < 2 {
            let mutex_clone = Arc::clone(&mutex);
            lock.push_back(resource.clone());
            tokio::spawn(async move {
                println!("consuming {:?} for 30 seconds", resource.clone());
                thread::sleep(time::Duration::from_secs(30));
                let mut newlock = mutex_clone.lock().await;
                newlock.pop_front();
                println!("done");
            });
        }
    }
}