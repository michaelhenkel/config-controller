use config_client::protos::github::com::michaelhenkel::config_controller::pkg::apis::v1;
use std::collections::VecDeque;

pub struct ResourceQueue {
    queue: VecDeque<v1::Resource>,
}

impl ResourceQueue {
    pub fn new() -> ResourceQueue {
        ResourceQueue{
            queue: VecDeque::new(),
        }
    }
    pub fn push(&mut self, resource: v1::Resource) -> bool {
        let mut pushed: bool = false;
        if !self.exists(&resource){
            self.queue.push_front(resource);
            pushed = true;
        }
        pushed
    }

    pub fn pop(&mut self) -> v1::Resource{
        let option_resource = self.queue.pop_front();
        let resource = option_resource.unwrap();
        resource
    }

    pub fn exists(&mut self, resource: &v1::Resource) -> bool {
        self.queue.contains(&resource)
    }

    pub fn empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}