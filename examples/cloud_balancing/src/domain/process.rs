#[derive(Debug, Clone)]
pub struct Process {
    pub process_id: usize,
    pub cpu_power_req: i64,
    pub memory_size_req: i64,
    pub network_bandwidth_req: i64,
    pub computer_id: Option<usize>,
}

impl Process {
    pub fn new(
        process_id: usize,
        cpu_power_req: i64,
        memory_size_req: i64,
        network_bandwidth_req: i64,
        computer_id: Option<usize>,
    ) -> Self {
        Self {
            process_id,
            cpu_power_req,
            memory_size_req,
            network_bandwidth_req,
            computer_id,
        }
    }
}