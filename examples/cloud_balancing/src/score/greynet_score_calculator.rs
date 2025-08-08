// greynet_score_calculator.rs

use crate::cotwin::{GreynetProcess, GreynetComputer};
use greyjack::score_calculation::greynet::{ConstraintBuilder, JoinerType, Collectors};
use greyjack::score_calculation::greynet::stream_def::{extract_fact, key};
use greyjack::score_calculation::score_calculators::GreynetScoreCalculator;
use greyjack::score_calculation::scores::HardSoftScore;
use greyjack::key;

pub struct CloudBalancingGreynetScoreCalculator;

impl CloudBalancingGreynetScoreCalculator {
    pub fn new() -> GreynetScoreCalculator<HardSoftScore> {
        let mut builder = ConstraintBuilder::<HardSoftScore>::new();

        // CPU capacity constraint
        builder
            .add_constraint("required_cpu", 1.0)
            .for_each::<GreynetProcess>()
            .group_by(
                key!(GreynetProcess, |p| p.computer_id),
                Collectors::sum(|tuple| {
                    extract_fact::<GreynetProcess>(tuple, 0)
                        .map_or(0.0, |p| p.cpu_power_req as f64)
                })
            )
            .join_on_group_key( // FIX: Use specialized join for grouped streams
                builder.for_each::<GreynetComputer>(),
                key!(GreynetComputer, |c| c.computer_id)
            )
            .filter(|tuple| {
                let total_cpu = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                total_cpu > computer.cpu_power as f64
            })
            .penalize(|tuple| {
                let total_cpu = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                HardSoftScore::hard(total_cpu - computer.cpu_power as f64)
            });

        // Memory capacity constraint
        builder
            .add_constraint("required_memory", 1.0)
            .for_each::<GreynetProcess>()
            .group_by(
                key!(GreynetProcess, |p| p.computer_id),
                Collectors::sum(|tuple| {
                    extract_fact::<GreynetProcess>(tuple, 0)
                        .map_or(0.0, |p| p.memory_size_req as f64)
                })
            )
            .join_on_group_key( // FIX: Use specialized join for grouped streams
                builder.for_each::<GreynetComputer>(),
                key!(GreynetComputer, |c| c.computer_id)
            )
            .filter(|tuple| {
                let total_mem = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                total_mem > computer.memory_size as f64
            })
            .penalize(|tuple| {
                let total_mem = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                HardSoftScore::hard(total_mem - computer.memory_size as f64)
            });

        // Network capacity constraint
        builder
            .add_constraint("required_network", 1.0)
            .for_each::<GreynetProcess>()
            .group_by(
                key!(GreynetProcess, |p| p.computer_id),
                Collectors::sum(|tuple| {
                    extract_fact::<GreynetProcess>(tuple, 0)
                        .map_or(0.0, |p| p.network_bandwidth_req as f64)
                })
            )
            .join_on_group_key( // FIX: Use specialized join for grouped streams
                builder.for_each::<GreynetComputer>(),
                key!(GreynetComputer, |c| c.computer_id)
            )
            .filter(|tuple| {
                let total_net = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                total_net > computer.network_bandwidth as f64
            })
            .penalize(|tuple| {
                let total_net = extract_fact::<f64>(tuple, 1).copied().unwrap_or(0.0);
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                HardSoftScore::hard(total_net - computer.network_bandwidth as f64)
            });

        // Cost minimization
        builder
            .add_constraint("computer_cost", 1.0)
            .for_each::<GreynetProcess>()
            .group_by(
                key!(GreynetProcess, |p| p.computer_id),
                Collectors::to_list()
            )
            .join_on_group_key( // FIX: Use specialized join for grouped streams
                builder.for_each::<GreynetComputer>(),
                key!(GreynetComputer, |c| c.computer_id)
            )
            .penalize(|tuple| {
                let computer = extract_fact::<GreynetComputer>(tuple, 2).unwrap();
                HardSoftScore::soft(computer.cost as f64)
            });

        GreynetScoreCalculator::new(builder)
    }
}
