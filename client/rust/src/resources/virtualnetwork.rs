use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use tonic::transport::Channel;
use tokio::sync::mpsc;
use std::error::Error;
use tokio::sync::mpsc::{Sender, Receiver};
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use crate::queue::queue::ResourceQueue;
use core::task::Poll;
use tokio::sync::mpsc::error::TryRecvError;
use core::task::{Context, Waker};
use std::{thread, time};
use futures;
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
        let queue_watcher_thread = queue_watcher(&mut queue_receiver);
        futures::join!(channel_watcher_thread, queue_watcher_thread);
        Ok(())
    }
}

pub async fn channel_watcher(queue_sender: &mut Sender<v1::Resource>, receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting channel_watcher");
    loop {
        let result = receiver.recv().await;
        println!("got resource2");
        queue_sender.send(result.unwrap().clone()).await;
    }
    Ok(())
}

pub async fn queue_watcher(receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting queue_watcher");

    let vec_deque: VecDeque<v1::Resource> = VecDeque::new();
    let mutex = Arc::new(Mutex::new(vec_deque));
    let mutex_clone = Arc::clone(&mutex);
    tokio::spawn(async move {
        loop {
            let mut lock = mutex.lock().await;
            if lock.is_empty() {
                continue;
            }
            let resource = lock.pop_front().unwrap();
            println!("consuming resource {:?}", resource); 
        }
    });

    loop {
        let resource = receiver.recv().await.unwrap();
        println!("got resource {:?}", resource);
        let mut lock = mutex_clone.lock().await;
        if !lock.contains(&resource) {
            lock.push_back(resource.clone());
        }
    }
    Ok(())
}

pub async fn resource_consumer(receiver: &mut Receiver<v1::Resource>) {

}