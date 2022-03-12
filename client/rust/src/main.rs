extern crate queues;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::SubscriptionRequest;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use config_client::protos::ssd_git::juniper::net::contrail::cn2::contrail::pkg::apis::core::v1alpha1;
use std::collections::HashMap;
use tonic::transport::Channel;
use std::error::Error;
use std::env;
use std::vec::Vec;
mod resources;
//use std::sync::mpsc;
//use std::sync::mpsc::{Sender, Receiver};
use std::{thread, time::Duration};
use futures::join;
use tokio::sync::watch;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
//use tokio::sync::watch::{Sender, Receiver};
use tonic::transport::Endpoint;
//use crossbeam_utils::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Endpoint::from_static("http://127.0.0.1:20443")
        .connect()
        .await?;

    //let (sender, receiver): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel();

    let (sender, mut receiver): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel(1);
    let mut subscription_client = ConfigControllerClient::new(channel.clone());
    let mut virtual_network_client = ConfigControllerClient::new(channel.clone());
    //let cloned_receiver = receiver.clone();
    let virtual_network_controller_thread = virtual_network_controller(&mut virtual_network_client,&mut receiver);
    let subscribe_thread = subscribe(&mut subscription_client, sender);
    futures::join!(subscribe_thread, virtual_network_controller_thread);

    Ok(())
}

async fn subscribe(client: &mut ConfigControllerClient<Channel>, sender: Sender<v1::Resource>) -> Result<(), Box<dyn Error>> {
    println!("started subscriber_controller");
    let request = tonic::Request::new(SubscriptionRequest {
        name: get_node(),
    });

    let mut queue_map: HashMap<String,ResourceQueue> = HashMap::new();
    queue_map.insert("VirtualNetwork".to_string(), ResourceQueue::new());

    let mut stream = client
        .subscribe_list_watch(request)
        .await?
        .into_inner();

    while let Some(resource) = stream.message().await? {
        //let resource_copy = resource.clone();
        //println!("resource = {:?}", resource_copy);
        if let Some(queue) = queue_map.get_mut(resource.kind.as_str()) {
            if queue.push(resource){
                sender.send(queue.pop()).await;
            }
        }
    }
    Ok(())
}

async fn virtual_network_controller(client: &mut ConfigControllerClient<Channel>, receiver: &mut Receiver<v1::Resource>) -> Result<(), Box<dyn Error>>  {
    println!("started virtual_network_controller");
    while let Some(res) = receiver.recv().await {
        let b = res.clone();
        println!("received = {:?}", b);
        let vn_result: Result<tonic::Response<v1alpha1::VirtualNetwork>, tonic::Status> = client.get_virtual_network(res).await;
        let vn_resp: &mut tonic::Response<v1alpha1::VirtualNetwork> = &mut vn_result.unwrap();
        let vn: &mut v1alpha1::VirtualNetwork = vn_resp.get_mut();
        println!("{}/{}", vn.metadata.as_ref().unwrap().namespace(), vn.metadata.as_ref().unwrap().name());
        println!("labels {:?}", vn.metadata.as_ref().unwrap().labels);
        thread::sleep(Duration::from_secs(20));
    }
    /*
    match receiver.recv() {
        Ok(resource) => println!("ok"),
        Err(e) => println!("err"),
    }
    */
    /*
    while let res = receiver.recv().unwrap() {
        let b = res.clone();
        println!("{} {}/{}", b.kind, b.namespace, b.name);
        let vn_result: Result<tonic::Response<v1alpha1::VirtualNetwork>, tonic::Status> = client.get_virtual_network(res).await;
        /*
        let vn_resp: &mut tonic::Response<v1alpha1::VirtualNetwork> = &mut vn_result.unwrap();
        let vn: &mut v1alpha1::VirtualNetwork = vn_resp.get_mut();
        println!("{}/{}", vn.metadata.as_ref().unwrap().namespace(), vn.metadata.as_ref().unwrap().name());
        */
    }
    */
    Ok(())
}

fn get_node() -> String {
    if env::args().len() > 0 {
        let args: Vec<String> = env::args().collect();
        args[1].to_string()
    } else {
        "5b3s30".to_string()
    }
}



struct ResourceQueue {
    queue: Vec<v1::Resource>,
}

impl ResourceQueue {
    pub fn new() -> ResourceQueue {
        ResourceQueue{
            queue: Vec::new(),
        }
    }
    pub fn push(&mut self, resource: v1::Resource) -> bool {
        let mut pushed: bool = false;
        if !self.exists(&resource){
            self.queue.push(resource);
            pushed = true;
        }
        pushed
    }

    pub fn pop(&mut self) -> v1::Resource{
        let option_resource = self.queue.pop();
        let resource = option_resource.unwrap();
        resource
    }

    pub fn exists(&mut self, resource: &v1::Resource) -> bool {
        let mut found: bool = false;
        for res in &self.queue {
            if res.name == resource.name && res.namespace == resource.namespace && res.kind == resource.kind {
                found = true;
                break;
            }
        }
        found
    }
}