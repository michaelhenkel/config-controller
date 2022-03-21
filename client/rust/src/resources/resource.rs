use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1::config_controller_client::ConfigControllerClient;
use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crossbeam_channel::TryRecvError;
use std::vec::Vec;
use std::error::Error;
use crate::resources;

const INTERVAL: u64 = 1;

pub struct ResourceController2 {

}

impl ResourceController2 {
    pub fn new() -> Self {
        Self{}
    }
    pub async fn run(self, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>, sender: crossbeam_channel::Sender<v1::Resource>, resource_interface: Box<dyn ResourceInterface2 + Send>, name: String) -> Result<(), Box<dyn Error + Send >>{
        let mut w_q = Vec::new();
        let mut r_q = Vec::new();
        let mut client = ConfigControllerClient::new(channel.clone());
        loop{
            let resource = receiver.clone().recv().unwrap();
            match v1::resource::Action::from_i32(resource.action){
            //match resource.action.from_i32(action_type) {
                Some(v1::resource::Action::Add) => {
                    if w_q.contains(&resource){
                        if !r_q.contains(&resource){
                            r_q.push(resource.clone());
                        }
                    } else {
                        w_q.push(resource.clone());
                        resource_interface.process(&mut client, sender.clone(), resource.clone());
                    }
                },
                Some(v1::resource::Action::Del) => {
                    w_q.retain(|x| *x != resource);
                    if r_q.contains(&resource){
                        r_q.retain(|x| *x != resource);
                        let resource = v1::Resource{
                            name: resource.name,
                            namespace: resource.namespace,
                            kind: resource.kind,
                            action: i32::from(v1::resource::Action::Add),
                        };
                        sender.send(resource).unwrap();
                    }

                },
                Some(v1::resource::Action::Retry) => {
                    if r_q.contains(&resource){
                        r_q.retain(|x| *x != resource);
                    }
                    resource_interface.process(&mut client, sender.clone(), resource.clone());
                },
                _ => {},
            }
        }
        Ok(())
    }
}

pub trait ResourceInterface2: Send {
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, sender: crossbeam_channel::Sender<v1::Resource>, resource: v1::Resource);

}

pub struct ResourceController {

}

impl ResourceController {
    pub fn new() -> Self {
        Self{}
    }
    pub async fn run(self, channel: tonic::transport::Channel, receiver: crossbeam_channel::Receiver<v1::Resource>, resource_interface: Box<dyn ResourceInterface + Send>, name: String) -> Result<(), Box<dyn Error + Send >>{
        let duration = tokio::time::Duration::from_millis(INTERVAL);
        println!("starting ResoureController for {}", name);
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
                        //println!("ResoureController {} got resource", name);
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
                                //println!("${:?} is sleeping for 2 secs", name);
                                tokio::time::sleep(duration).await;
                            }
                        } else {
                            //println!("${:?} is sleeping for 2 secs", name);
                            tokio::time::sleep(duration).await;
                        }
                    },
                    _ => {
                        println!("error");
                        continue; 
                    },
                }
            }            
        });
        Ok(())
    }
}

pub trait ResourceInterface: Send {
    fn process(&self, client: &mut ConfigControllerClient<tonic::transport::Channel>, resource: v1::Resource, worker_queue_mutex: Arc<Mutex<Vec<v1::Resource>>>) -> Result<(), Box<dyn Error + Send >>;

}

pub fn res_list() -> Vec<String> {
    vec![
        "VirtualNetwork".to_string(),
        "VirtualMachineInterface".to_string(),
        "VirtualMachine".to_string(),
    ]
}

pub fn get_res(name: String) -> Box<dyn ResourceInterface + Send> {
    match name.as_str() {
        "VirtualNetwork" => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
        "VirtualMachineInterface" => Box::new(resources::virtualmachineinterface::VirtualMachineInterfaceController::new()),
        "VirtualMachine" => Box::new(resources::virtualmachine::VirtualMachineController::new()),
        _ => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
    }
}

pub fn get_res2(name: String) -> Box<dyn ResourceInterface2 + Send> {
    match name.as_str() {
        "VirtualNetwork" => Box::new(resources::virtualnetwork::VirtualNetworkController::new()),
        "VirtualMachineInterface" => Box::new(resources::virtualmachineinterface::VirtualMachineInterfaceController::new()),
        "VirtualMachine" => Box::new(resources::virtualmachine::VirtualMachineController::new()),
        _ => Box::new(resources::virtualmachineinterface::VirtualMachineInterfaceController::new()),
    }
}