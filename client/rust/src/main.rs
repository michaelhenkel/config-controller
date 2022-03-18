use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::SubscriptionRequest;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::HashMap;
use tonic::transport::Channel;
use std::error::Error;
use std::env;
use std::vec::Vec;
use std::pin::Pin;
mod resources;
mod queue;
use tonic::transport::Endpoint;
use crossbeam_channel::bounded;
use crate::resources::resource::{get_res, res_list, ResourceController};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures;
use futures::future::TryFutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Endpoint::from_static("http://127.0.0.1:20443")
        .connect()
        .await?;

    let mut sender_map: Arc<Mutex<HashMap<String,crossbeam_channel::Sender<v1::Resource>>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut join_handles = Vec::new();
    for r in res_list(){
        let (sender, receiver): (crossbeam_channel::Sender<v1::Resource>, crossbeam_channel::Receiver<v1::Resource>) = bounded(1);
        let sender_map = sender_map.clone();
        let mut sender_map = sender_map.lock().await;
        sender_map.insert(r.to_string(), sender);
        let rc = ResourceController::new();
        let res = get_res(r);
        let run_res = rc.run(channel.clone(), receiver, res).map_err(|_| "Unable to get book".to_string());
        let join_handle = tokio::task::spawn(run_res);
        join_handles.push(join_handle);
    }
    


    //let mut subscription_client = ConfigControllerClient::new(channel.clone());
    let subscribe_thread = subscribe(channel.clone(), sender_map).map_err(|_| "Unable to get book".to_string());
    let join_handle = tokio::task::spawn(subscribe_thread);

    join_handles.push(join_handle);
    futures::future::join_all(join_handles).await;
    //futures::join!(subscribe_thread, virtual_network_controller_thread, virtual_machine_interface_controller_thread);


    Ok(())
}



//async fn subscribe(client: &mut ConfigControllerClient<Channel>, sender_map: &mut HashMap<String,crossbeam_channel::Sender<v1::Resource>>) -> Result<(), Box<dyn Error>> {
async fn subscribe(channel: tonic::transport::Channel, sender_map: Arc<Mutex<HashMap<String,crossbeam_channel::Sender<v1::Resource>>>>) -> Result<(), Box<dyn Error>> {
    println!("started subscriber_controller");
    let mut client = ConfigControllerClient::new(channel.clone());
    let request = tonic::Request::new(SubscriptionRequest {
        name: get_node(),
    });

    let mut stream = client
        .subscribe_list_watch(request)
        .await?
        .into_inner();

    while let Some(resource) = stream.message().await? {
        //println!("got resource");
        let sender_map = sender_map.clone();
        let sender_map = sender_map.lock().await;
        if let Some(sender) = sender_map.get(resource.kind.as_str()) {
            //println!("sending resource");
            sender.send(resource.clone()).unwrap();
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