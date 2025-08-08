use crate::cotwin::{CotProcess, CotComputer, GreynetProcess, GreynetComputer};
use crate::domain::ScheduleCB;
use crate::score::{CloudBalancingPlainScoreCalculator, CloudBalancingIncrementalScoreCalculator, CloudBalancingGreynetScoreCalculator};
use greyjack::cotwin::{Cotwin, CotwinBuilderTrait, CotwinEntityTrait, CotwinValueTypes};
use greyjack::score_calculation::greynet::greynet_traits::{InitializableFact, ModifiableFact};
use greyjack::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use greyjack::score_calculation::scores::HardSoftScore;
use greyjack::variables::GJInteger;
use polars::datatypes::AnyValue;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone)]
pub enum EntityVariants<'a> {
    CotProcess(CotProcess<'a>),
    CotComputer(CotComputer<'a>),
}

impl<'a> CotwinEntityTrait for EntityVariants<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        match self {
            EntityVariants::CotProcess(x) => x.to_vec(),
            EntityVariants::CotComputer(x) => x.to_vec(),
        }
    }
}

impl<'a> InitializableFact for EntityVariants<'a> {
    fn to_initialized_fact(&self) -> Rc<dyn ModifiableFact + Send> {
        match self {
            EntityVariants::CotProcess(cp) => {
                let process_id = match cp.process_id {
                    CotwinValueTypes::PAV(AnyValue::UInt64(v)) => v as i64,
                    _ => panic!("Invalid process_id type"),
                };
                let cpu_power_req = match cp.cpu_power_req {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid cpu_power_req type"),
                };
                let memory_size_req = match cp.memory_size_req {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid memory_size_req type"),
                };
                let network_bandwidth_req = match cp.network_bandwidth_req {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid network_bandwidth_req type"),
                };
                let computer_id = match &cp.computer_id {
                    CotwinValueTypes::GJI(gji) => gji.initial_value.unwrap_or(0.0) as i64, 
                    _ => panic!("Invalid computer_id type"),
                };

                Rc::new(GreynetProcess {
                    process_id,
                    cpu_power_req,
                    memory_size_req,
                    network_bandwidth_req,
                    computer_id,
                })
            },
            EntityVariants::CotComputer(cc) => {
                let computer_id = match cc.computer_id {
                    CotwinValueTypes::PAV(AnyValue::UInt64(v)) => v as i64,
                    _ => panic!("Invalid computer_id type"),
                };
                let cpu_power = match cc.cpu_power {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid cpu_power type"),
                };
                let memory_size = match cc.memory_size {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid memory_size type"),
                };
                let network_bandwidth = match cc.network_bandwidth {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid network_bandwidth type"),
                };
                let cost = match cc.cost {
                    CotwinValueTypes::PAV(AnyValue::Int64(v)) => v,
                    _ => panic!("Invalid cost type"),
                };

                Rc::new(GreynetComputer {
                    computer_id,
                    cpu_power,
                    memory_size,
                    network_bandwidth,
                    cost,
                })
            }
        }
    }
}

#[derive(Clone)]
pub enum UtilityObjectVariants {
    ProcessesInfo(Vec<[i64; 3]>),
    ComputersInfo(Vec<[i64; 3]>),
    ComputersCosts(Vec<i64>),
}

#[derive(Clone, Copy)]
pub enum CalculatorType {
    Plain,
    Incremental,
    Greynet,
}

#[derive(Clone)]
pub struct CotwinBuilder {
    calculator_type: CalculatorType,
    use_greed_init: bool,
}

impl CotwinBuilder {
    pub fn new(calculator_type: CalculatorType, use_greed_init: bool) -> Self {
        Self {
            calculator_type,
            use_greed_init,
        }
    }

    fn build_planning_processes<'a>(&self, domain: &ScheduleCB) -> Vec<EntityVariants<'a>> {
        let n_computers = domain.computers.len();
        let n_processes = domain.processes.len();
        
        let initial_computer_ids = if self.use_greed_init {
            self.build_greed_initial_computer_ids(domain)
        } else {
            vec![None; n_processes]
        };

        let mut cot_processes = Vec::new();
        for (i, process) in domain.processes.iter().enumerate() {
            let cot_process = CotProcess {
                process_id: CotwinValueTypes::PAV(AnyValue::UInt64(process.process_id as u64)),
                cpu_power_req: CotwinValueTypes::PAV(AnyValue::Int64(process.cpu_power_req)),
                memory_size_req: CotwinValueTypes::PAV(AnyValue::Int64(process.memory_size_req)),
                network_bandwidth_req: CotwinValueTypes::PAV(AnyValue::Int64(process.network_bandwidth_req)),
                computer_id: CotwinValueTypes::GJI(GJInteger::new(
                    initial_computer_ids[i],
                    0,
                    (n_computers - 1) as i64,
                    false,
                    Some(vec!["common".to_string()]),
                )),
            };
            cot_processes.push(EntityVariants::CotProcess(cot_process));
        }

        cot_processes
    }

    fn build_problem_fact_computers<'a>(&self, domain: &ScheduleCB) -> Vec<EntityVariants<'a>> {
        let mut cot_computers = Vec::new();
        for computer in &domain.computers {
            let cot_computer = CotComputer {
                computer_id: CotwinValueTypes::PAV(AnyValue::UInt64(computer.computer_id as u64)),
                cpu_power: CotwinValueTypes::PAV(AnyValue::Int64(computer.cpu_power)),
                memory_size: CotwinValueTypes::PAV(AnyValue::Int64(computer.memory_size)),
                network_bandwidth: CotwinValueTypes::PAV(AnyValue::Int64(computer.network_bandwidth)),
                cost: CotwinValueTypes::PAV(AnyValue::Int64(computer.cost)),
            };
            cot_computers.push(EntityVariants::CotComputer(cot_computer));
        }
        cot_computers
    }

    fn build_greed_initial_computer_ids(&self, domain: &ScheduleCB) -> Vec<Option<i64>> {
        let n_processes = domain.processes.len();
        let n_computers = domain.computers.len();
        
        let mut initial_computer_ids = vec![None; n_processes];
        let mut assigned_processes_counts = vec![0; n_computers];
        
        for process_id in 0..n_processes {
            let process = &domain.processes[process_id];
            
            for computer_id in 0..n_computers {
                let computer = &domain.computers[computer_id];
                
                // Simple greedy: assign to first computer that has capacity
                let mut current_consumed = [0i64; 3];
                
                // Calculate current consumption for this computer
                for (i, &count) in assigned_processes_counts.iter().enumerate() {
                    if i == computer_id {
                        for j in 0..process_id {
                            if initial_computer_ids[j] == Some(computer_id as i64) {
                                let other_process = &domain.processes[j];
                                current_consumed[0] += other_process.cpu_power_req;
                                current_consumed[1] += other_process.memory_size_req;
                                current_consumed[2] += other_process.network_bandwidth_req;
                            }
                        }
                        break;
                    }
                }
                
                // Add current process requirements
                current_consumed[0] += process.cpu_power_req;
                current_consumed[1] += process.memory_size_req;
                current_consumed[2] += process.network_bandwidth_req;
                
                // Check if computer has enough capacity
                let computer_resources = [computer.cpu_power, computer.memory_size, computer.network_bandwidth];
                let mut fits = true;
                for resource_type in 0..3 {
                    if current_consumed[resource_type] > computer_resources[resource_type] {
                        fits = false;
                        break;
                    }
                }
                
                if fits {
                    initial_computer_ids[process_id] = Some(computer_id as i64);
                    assigned_processes_counts[computer_id] += 1;
                    break;
                }
            }
        }
        
        initial_computer_ids
    }

    fn build_processes_common_info(&self, domain: &ScheduleCB) -> Vec<[i64; 3]> {
        domain.processes.iter().map(|process| [
            process.cpu_power_req,
            process.memory_size_req,
            process.network_bandwidth_req,
        ]).collect()
    }

    fn build_computers_common_info(&self, domain: &ScheduleCB) -> Vec<[i64; 3]> {
        domain.computers.iter().map(|computer| [
            computer.cpu_power,
            computer.memory_size,
            computer.network_bandwidth,
        ]).collect()
    }

    fn build_computers_costs(&self, domain: &ScheduleCB) -> Vec<i64> {
        domain.computers.iter().map(|computer| computer.cost).collect()
    }
}

impl<'a> CotwinBuilderTrait<ScheduleCB, EntityVariants<'a>, UtilityObjectVariants, HardSoftScore>
    for CotwinBuilder
{
    fn build_cotwin(
        &self,
        domain_model: ScheduleCB,
        _is_already_initialized: bool,
    ) -> Cotwin<EntityVariants<'a>, UtilityObjectVariants, HardSoftScore> {
        let mut cotwin = Cotwin::new();

        cotwin.add_planning_entities("processes".to_string(), self.build_planning_processes(&domain_model));
        cotwin.add_problem_facts("computers".to_string(), self.build_problem_fact_computers(&domain_model));

        let calculator = match self.calculator_type {
            CalculatorType::Plain => {
                ScoreCalculatorVariants::PSC(CloudBalancingPlainScoreCalculator::new())
            },
            CalculatorType::Incremental => {
                let mut calculator = CloudBalancingIncrementalScoreCalculator::new();
                calculator.add_utility_object(
                    "processes_info".to_string(),
                    UtilityObjectVariants::ProcessesInfo(self.build_processes_common_info(&domain_model))
                );
                calculator.add_utility_object(
                    "computers_info".to_string(),
                    UtilityObjectVariants::ComputersInfo(self.build_computers_common_info(&domain_model))
                );
                calculator.add_utility_object(
                    "computers_costs".to_string(),
                    UtilityObjectVariants::ComputersCosts(self.build_computers_costs(&domain_model))
                );
                ScoreCalculatorVariants::ISC(calculator)
            },
            CalculatorType::Greynet => {
                ScoreCalculatorVariants::Greynet(CloudBalancingGreynetScoreCalculator::new())
            }
        };

        cotwin.add_score_calculator(calculator);
        cotwin
    }
}

unsafe impl Send for CotwinBuilder {}