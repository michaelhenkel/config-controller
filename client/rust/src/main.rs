extern crate queues;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::SubscriptionRequest;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::HashMap;
use tonic::transport::Channel;
use std::error::Error;
use std::env;
use std::vec::Vec;
mod resources;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ConfigControllerClient::connect("http://127.0.0.1:20443").await.unwrap();

    consume_response(&mut client).await?;
    println!("Connected...now sleeping for 2 seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    drop(client);

    println!("Disconnected...");
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

async fn consume_response(client: &mut ConfigControllerClient<Channel>) -> Result<(), Box<dyn Error>> {
    let mut queue_map: HashMap<String,ResourceQueue> = HashMap::new();
    queue_map.insert("VirtualNetwork".to_string(), ResourceQueue::new());

    let (tx, rx): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel();

    let request = tonic::Request::new(SubscriptionRequest {
        name: get_node(),
    });

    let virtual_network_thread = thread::spawn(move || {
        //virtual_network_controller(rx, client);
        virtual_network_controller(rx);
    });

    let mut stream = client
        .subscribe_list_watch(request)
        .await?
        .into_inner();


    while let Some(resource) = stream.message().await? {
        if let Some(queue) = queue_map.get_mut(resource.kind.as_str()) {
            if queue.push(resource){
                tx.send(queue.pop()).unwrap();
            }
        } 
    }
    drop(stream);
    virtual_network_thread.join().unwrap();
    Ok(())
}

//fn virtual_network_controller(rx: Receiver<v1::Resource>, client: &mut ConfigControllerClient<Channel>) {
fn virtual_network_controller(rx: Receiver<v1::Resource>) {
    println!("started virtual_network_controller");
    while let res = rx.recv().unwrap() {
        //client.get_virtual_network(res);
        println!("{} {}/{}", res.kind, res.namespace, res.name);
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