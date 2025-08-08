use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

#[derive(Clone)]
pub struct CotComputer<'a> {
    pub computer_id: CotwinValueTypes<'a>,
    pub cpu_power: CotwinValueTypes<'a>,
    pub memory_size: CotwinValueTypes<'a>,
    pub network_bandwidth: CotwinValueTypes<'a>,
    pub cost: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotComputer<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        let mut computer_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        computer_vec.push(("computer_id".to_string(), self.computer_id.clone()));
        computer_vec.push(("cpu_power".to_string(), self.cpu_power.clone()));
        computer_vec.push(("memory_size".to_string(), self.memory_size.clone()));
        computer_vec.push(("network_bandwidth".to_string(), self.network_bandwidth.clone()));
        computer_vec.push(("cost".to_string(), self.cost.clone()));
        computer_vec
    }
}