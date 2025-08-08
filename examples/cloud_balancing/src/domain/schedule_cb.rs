use super::{Computer, Process};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ScheduleCB {
    pub computers: Vec<Computer>,
    pub processes: Vec<Process>,
}

impl ScheduleCB {
    pub fn new(computers: Vec<Computer>, processes: Vec<Process>) -> Self {
        Self { computers, processes }
    }

    pub fn print_metrics(&self) {
        let mut used_resources_map: HashMap<usize, ResourceUsage> = HashMap::new();

        for process in &self.processes {
            if let Some(computer_id) = process.computer_id {
                let usage = used_resources_map.entry(computer_id).or_insert(ResourceUsage {
                    processes: Vec::new(),
                    cpu: 0,
                    memory: 0,
                    network: 0,
                });

                usage.processes.push(process.process_id);
                usage.cpu += process.cpu_power_req;
                usage.memory += process.memory_size_req;
                usage.network += process.network_bandwidth_req;
            }
        }

        let mut total_violations = 0;
        let mut computer_ids: Vec<usize> = used_resources_map.keys().cloned().collect();
        computer_ids.sort();

        for computer_id in &computer_ids {
            let usage = &used_resources_map[computer_id];
            let computer = &self.computers[*computer_id];

            if usage.cpu > computer.cpu_power {
                total_violations += 1;
            }
            if usage.memory > computer.memory_size {
                total_violations += 1;
            }
            if usage.network > computer.network_bandwidth {
                total_violations += 1;
            }

            println!("Computer {} utilization:", computer_id);
            println!("    CPU: {} / {} | RAM: {} / {} | NTWRK: {} / {}",
                     usage.cpu, computer.cpu_power,
                     usage.memory, computer.memory_size,
                     usage.network, computer.network_bandwidth);
            println!("    PIDs: {:?}", usage.processes);
            println!();
        }

        println!("Total violations: {}", total_violations);
        println!("Computers used: {}", computer_ids.len());
    }
}

#[derive(Debug)]
struct ResourceUsage {
    processes: Vec<usize>,
    cpu: i64,
    memory: i64,
    network: i64,
}