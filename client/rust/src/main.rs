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
mod queue;
use std::{thread, time::Duration};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tonic::transport::Endpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Endpoint::from_static("http://127.0.0.1:20443")
        .connect()
        .await?;

    let mut sender_map: HashMap<String,Sender<v1::Resource>> = HashMap::new();

    let (virtual_network_sender, virtual_network_receiver): (Sender<v1::Resource>, Receiver<v1::Resource>) = mpsc::channel(100);
    sender_map.insert("VirtualNetwork".to_string(), virtual_network_sender);

    let mut subscription_client = ConfigControllerClient::new(channel.clone());


    
    let mut virtual_network_controller = resources::virtualnetwork::VirtualNetworkController::new(channel.clone(), virtual_network_receiver);
    let virtual_network_controller_thread = virtual_network_controller.run();

    let subscribe_thread = subscribe(&mut subscription_client, &mut sender_map);

    futures::join!(subscribe_thread, virtual_network_controller_thread);

    Ok(())
}

async fn subscribe(client: &mut ConfigControllerClient<Channel>, sender_map: &mut HashMap<String,Sender<v1::Resource>>) -> Result<(), Box<dyn Error>> {
    println!("started subscriber_controller");
    let request = tonic::Request::new(SubscriptionRequest {
        name: get_node(),
    });

    let mut stream = client
        .subscribe_list_watch(request)
        .await?
        .into_inner();

    while let Some(resource) = stream.message().await? {
        println!("got resource");
        if let Some(sender) = sender_map.get(resource.kind.as_str()) {
            println!("sending resource");
            sender.send(resource.clone()).await.unwrap();
        }
    }
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