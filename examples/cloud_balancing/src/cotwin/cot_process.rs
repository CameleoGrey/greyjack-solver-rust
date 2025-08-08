use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

#[derive(Clone)]
pub struct CotProcess<'a> {
    pub process_id: CotwinValueTypes<'a>,
    pub cpu_power_req: CotwinValueTypes<'a>,
    pub memory_size_req: CotwinValueTypes<'a>,
    pub network_bandwidth_req: CotwinValueTypes<'a>,
    pub computer_id: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotProcess<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        let mut process_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        process_vec.push(("process_id".to_string(), self.process_id.clone()));
        process_vec.push(("cpu_power_req".to_string(), self.cpu_power_req.clone()));
        process_vec.push(("memory_size_req".to_string(), self.memory_size_req.clone()));
        process_vec.push(("network_bandwidth_req".to_string(), self.network_bandwidth_req.clone()));
        process_vec.push(("computer_id".to_string(), self.computer_id.clone()));
        process_vec
    }
}