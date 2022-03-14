use crate::resources::traits::{ProcessResource};
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use tonic::transport::Channel;
use tokio::sync::mpsc;
use std::error::Error;
use tokio::sync::mpsc::{Sender, Receiver};
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;

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
        let channel_watcher_thread = channel_watcher(&mut self.receiver);
        futures::join!(channel_watcher_thread);
        Ok(())
    }
}

pub async fn channel_watcher(receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("starting channel_Watcher");
    loop {
        let result = receiver.recv().await;
        println!("got resource2")
    }
    Ok(())
}