#[derive(Debug, Clone)]
pub struct Computer {
    pub computer_id: usize,
    pub cpu_power: i64,
    pub memory_size: i64,
    pub network_bandwidth: i64,
    pub cost: i64,
}

impl Computer {
    pub fn new(
        computer_id: usize,
        cpu_power: i64,
        memory_size: i64,
        network_bandwidth: i64,
        cost: i64,
    ) -> Self {
        Self {
            computer_id,
            cpu_power,
            memory_size,
            network_bandwidth,
            cost,
        }
    }
}