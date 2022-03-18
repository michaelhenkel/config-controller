use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crossbeam_channel::TryRecvError;
use std::vec::Vec;
use async_trait::async_trait;
use std::error::Error;
use crate::resources;


pub struct ResourceController {

}

impl ResourceController {
    pub fn new() -> Self {
        Self{}
    }
    pub async fn run(self, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>, resource_interface: Box<dyn ResourceInterface + Send>) -> Result<(), Box<dyn Error + Send >>{
        println!("starting ResoureController");
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
                        let mut resource_queue_lock = resource_queue_mutex.lock().await;
                        if !resource_queue_lock.contains(&resource){
                            resource_queue_lock.push_back(resource);
                        }
                    },
                    Err(TryRecvError::Empty) => {
                        let mut resource_queue_lock = resource_queue_mutex.lock().await;
                        let worker_queue_mutex_clone = worker_queue_mutex.clone();
                        if !resource_queue_lock.is_empty(){
                            let resource = resource_queue_lock.pop_front().unwrap();
                            let mut worker_queue_lock = worker_queue_mutex.lock().await;
                            if !worker_queue_lock.contains(&resource){
                                worker_queue_lock.push(resource.clone());
                                let mut client = client.clone();
                                resource_interface.process(&mut client, resource.clone(), worker_queue_mutex_clone);
                            } else {
                                if !resource_queue_lock.contains(&resource){
                                    resource_queue_lock.push_back(resource.clone());
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    },
                    _ => { continue; },
                }
            }            
        });
        Ok(())
    }
}

pub trait ResourceInterface: Send {
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);

}


pub fn res_list() -> Vec<String> {
    vec!["VirtualNetwork".to_string()]
}

/*
async fn foo() {
    let mut join_handles = Vec::new();
    for r in res_list(){
        let rc = ResourceController::new();
        let res = get_res(r);
        let run_res = rc.run(res);
        let join_handle = tokio::task::spawn(run_res);
        join_handles.push(join_handle);
    }
    futures::future::join_all(join_handles).await;
}
*/

pub fn get_res(name: String) -> Box<dyn ResourceInterface + Send> {
    match name.as_str() {
        "VirtualNetwork" => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
        _ => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
    }
}

/*
#[async_trait]
pub trait ResourceController {
    async fn process_resource(&mut self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);
    //fn process_resource2(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>);
    async fn process_resource2(&mut self);
    async fn run(&self, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>) -> Result<(), Box<dyn Error>>;
    fn to_trait(self) -> Box<dyn ResourceController + Send + Sync>;
    //fn bla(self, da: String) -> String;

    fn test(self: Arc<Self>, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>){
        println!("starting queue_watcher");
        let receiver_clone = receiver.clone();
        let client = ConfigControllerClient::new(channel.clone());
        let resource_queue: VecDeque<v1::Resource> = VecDeque::new();
        let resource_queue_mutex: Arc<Mutex<VecDeque<v1::Resource>>> = Arc::new(Mutex::new(resource_queue));
        //let mut bla = Arc::new(self.to_trait());
        //let mut x = self.to_trait();
        //let me = Arc::clone(&self);
        //let mut t = me.to_trait();
        tokio::spawn(async move {
            let worker_queue: Vec<v1::Resource> = Vec::new();
            let worker_queue_mutex = Arc::new(Mutex::new(worker_queue));
            //me.to_trait();
            loop{
                match receiver_clone.try_recv() {
                    Ok(resource) => { 
                        let mut resource_queue_lock = resource_queue_mutex.lock().await;
                        if !resource_queue_lock.contains(&resource){
                            resource_queue_lock.push_back(resource);
                        }
                    },
                    Err(TryRecvError::Empty) => {
                        let mut resource_queue_lock = resource_queue_mutex.lock().await;
                        let worker_queue_mutex_clone = worker_queue_mutex.clone();
                        if !resource_queue_lock.is_empty(){
                            let resource = resource_queue_lock.pop_front().unwrap();
                            let mut worker_queue_lock = worker_queue_mutex.lock().await;
                            if !worker_queue_lock.contains(&resource){
                                worker_queue_lock.push(resource.clone());
                                let mut client = client.clone();
                                //x.process_resource2();
                                //t.process_resource2();
                                //let me = Arc::clone(&self);
                                //bla.process_resource2();
                                //let foo = Box::new(ResourceController{}) as Box<dyn ResourceController + Send>;
                                //self.bla("bla".to_string());
                                //let rc = Box::new(&mut self);
                                //let self_mutex = Arc::new(Mutex::new(rc));
                                //let join_handle = tokio::spawn(async move {
                                //self.process_resource2(&mut client, resource.clone(), worker_queue_mutex_clone).await;
                                //});
                                //let mut resource_controller_lock = self_mutex.lock().await;
                                //resource_controller_lock.process_resource2(&mut client, resource.clone(), worker_queue_mutex_clone).await;
                            } else {
                                if !resource_queue_lock.contains(&resource){
                                    resource_queue_lock.push_back(resource.clone());
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                            }
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    },
                    _ => { continue; },
                }
            }
        });
    }
}

pub async fn process_resource(resource_controller: Arc<Mutex<Box<dyn ResourceController + Send + Sync>>>){

}

pub fn get_controller(kind: String) -> Arc<Mutex<Box<dyn ResourceController + Send + Sync>>>{
    match kind.as_str() {
        "VirtualNetwork" => Arc::new(Mutex::new(Box::new(resources::virtualnetwork::VirtualNetworkController::new()))),
        "VirtualMachineInterface" => Arc::new(Mutex::new(Box::new(resources::virtualnetwork::VirtualNetworkController::new()))),
        _ => Arc::new(Mutex::new(Box::new(resources::virtualnetwork::VirtualNetworkController::new()))),
    }
}

pub fn get_controller2(kind: String) -> Box<dyn ResourceController + Send + Sync>{
    match kind.as_str() {
        "VirtualNetwork" => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
        "VirtualMachineInterface" => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
        _ => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
    }
}

pub async fn queue_watcher(channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>, resource_controller: Arc<Mutex<Box<dyn ResourceController + Send + Sync>>>) {
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
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    if !resource_queue_lock.contains(&resource){
                        resource_queue_lock.push_back(resource);
                    }
                },
                Err(TryRecvError::Empty) => {
                    let mut resource_queue_lock = resource_queue_mutex.lock().await;
                    let worker_queue_mutex_clone = worker_queue_mutex.clone();
                    if !resource_queue_lock.is_empty(){
                        let resource = resource_queue_lock.pop_front().unwrap();
                        let mut worker_queue_lock = worker_queue_mutex.lock().await;
                        if !worker_queue_lock.contains(&resource){
                            worker_queue_lock.push(resource.clone());
                            let mut client = client.clone();
                            let mut resource_controller_lock = resource_controller.lock().await;
                            resource_controller_lock.process_resource(&mut client, resource.clone(), worker_queue_mutex_clone).await;
                        } else {
                            if !resource_queue_lock.contains(&resource){
                                resource_queue_lock.push_back(resource.clone());
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                },
                _ => { continue; },
            }
        }
    });
}
*/