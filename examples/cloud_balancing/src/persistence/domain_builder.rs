use crate::domain::{Computer, Process, ScheduleCB};
use greyjack::domain::DomainBuilderTrait;
use greyjack::score_calculation::scores::HardSoftScore;
use serde_json::*;
use polars::datatypes::AnyValue;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone)]
pub struct DomainBuilder {
    file_path: String,
}

impl DomainBuilderTrait<ScheduleCB> for DomainBuilder {
    fn build_domain_from_scratch(&self) -> ScheduleCB {
        let file = File::open(&self.file_path)
            .expect(&format!("Failed to open file: {}", self.file_path));
        let reader = BufReader::new(file);
        let data: Value = serde_json::from_reader(reader)
            .expect("Failed to parse JSON");

        let mut computers = Vec::new();
        if let Some(computer_list) = data["computerList"].as_array() {
            for (i, computer_dict) in computer_list.iter().enumerate() {
                let computer = Computer::new(
                    i,
                    computer_dict["cpuPower"].as_i64().unwrap(),
                    computer_dict["memory"].as_i64().unwrap(),
                    computer_dict["networkBandwidth"].as_i64().unwrap(),
                    computer_dict["cost"].as_i64().unwrap(),
                );
                computers.push(computer);
            }
        }

        let mut processes = Vec::new();
        if let Some(process_list) = data["processList"].as_array() {
            for (i, process_dict) in process_list.iter().enumerate() {
                let computer_id = process_dict["computer"].as_i64()
                    .map(|id| id as usize);
                let process = Process::new(
                    i,
                    process_dict["requiredCpuPower"].as_i64().unwrap(),
                    process_dict["requiredMemory"].as_i64().unwrap(),
                    process_dict["requiredNetworkBandwidth"].as_i64().unwrap(),
                    computer_id,
                );
                processes.push(process);
            }
        }

        ScheduleCB::new(computers, processes)
    }

    fn build_from_solution(&self, solution: &Value, initial_domain: Option<ScheduleCB>) -> ScheduleCB {
        let mut domain = match initial_domain {
            Some(d) => d,
            None => self.build_domain_from_scratch(),
        };

        let solution: (Vec<(String, AnyValue)>, HardSoftScore) = from_value(solution.clone()).unwrap();
        let variable_values = solution.0;

        let n_processes = variable_values.len();
        let mut process_assigned_computer_id_map = vec![-1i64; n_processes];

        for (assign_name, computer_id_value) in variable_values {
            let computer_id = match computer_id_value {
                AnyValue::Int64(id) => id,
                _ => panic!("Expected Int64 for computer_id"),
            };
            
            // Parse process ID from assignment name (format: "process X-->computer_assignment")
            let parts: Vec<&str> = assign_name.split("-->").collect();
            if parts.len() >= 1 {
                let process_part: Vec<&str> = parts[0].split(" ").collect();
                if process_part.len() >= 2 {
                    let process_id: usize = process_part[1].parse().unwrap();
                    process_assigned_computer_id_map[process_id] = computer_id;
                }
            }
        }

        // Update process assignments
        for (i, &computer_id) in process_assigned_computer_id_map.iter().enumerate() {
            if computer_id >= 0 {
                domain.processes[i].computer_id = Some(computer_id as usize);
            } else {
                domain.processes[i].computer_id = None;
            }
        }

        domain
    }
}

impl DomainBuilder {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

unsafe impl Send for DomainBuilder {}