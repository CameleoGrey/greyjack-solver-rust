use greyjack::score_calculation::score_calculators::IncrementalScoreCalculator;
use greyjack::score_calculation::scores::HardSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::{HashMap, HashSet};
use polars::prelude::*;

pub struct CloudBalancingIncrementalScoreCalculator;

impl CloudBalancingIncrementalScoreCalculator {
    pub fn new() -> IncrementalScoreCalculator<UtilityObjectVariants, HardSoftScore> {
        let mut score_calculator = IncrementalScoreCalculator::new();
        score_calculator.add_constraint("all_in_one_constraint".to_string(), Box::new(Self::all_in_one_constraint));
        score_calculator
    }

    fn all_in_one_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>,
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {
        let processes_df = planning_entity_dfs["processes"].clone();
        let processes_deltas_df = delta_dfs["processes"].clone();

        let processes_info = match &utility_objects["processes_info"] {
            UtilityObjectVariants::ProcessesInfo(info) => info,
            _ => panic!("Expected processes_info in utility objects"),
        };

        let computers_info = match &utility_objects["computers_info"] {
            UtilityObjectVariants::ComputersInfo(info) => info,
            _ => panic!("Expected computers_info in utility objects"),
        };

        let computers_costs = match &utility_objects["computers_costs"] {
            UtilityObjectVariants::ComputersCosts(costs) => costs,
            _ => panic!("Expected computers_costs in utility objects"),
        };

        let native_computer_ids: Vec<usize> = processes_df["computer_id"]
            .i64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|id| id.unwrap() as usize)
            .collect();

        let scores: Vec<HardSoftScore> = processes_deltas_df
            .partition_by(["sample_id"], false)
            .unwrap()
            .iter()
            .map(|sample_df| {
                let delta_row_ids: Vec<usize> = sample_df["candidate_df_row_id"]
                    .u64()
                    .unwrap()
                    .to_vec()
                    .iter()
                    .map(|id| id.unwrap() as usize)
                    .collect();

                let computer_delta_ids: Vec<usize> = sample_df["computer_id"]
                    .i64()
                    .unwrap()
                    .to_vec()
                    .iter()
                    .map(|id| id.unwrap() as usize)
                    .collect();

                let (resources_unacceptance, candidate_total_cost) = compute_penalties(
                    &native_computer_ids,
                    &delta_row_ids,
                    &computer_delta_ids,
                    computers_costs,
                    computers_info,
                    processes_info,
                );

                HardSoftScore::new(resources_unacceptance, candidate_total_cost)
            })
            .collect();

        scores
    }
}

fn compute_penalties(
    native_computer_ids: &[usize],
    delta_row_ids: &[usize],
    computer_delta_ids: &[usize],
    computers_costs: &[i64],
    computers_info: &[[i64; 3]],
    processes_info: &[[i64; 3]],
) -> (f64, f64) {
    let mut sample_computer_ids = native_computer_ids.to_vec();
    for (row_id, computer_id) in delta_row_ids.iter().zip(computer_delta_ids.iter()) {
        sample_computer_ids[*row_id] = *computer_id;
    }

    // Calculate soft score (total cost of used computers)
    let mut candidate_total_cost = 0.0;
    let unique_computer_ids: HashSet<usize> = sample_computer_ids.iter().cloned().collect();
    for computer_id in unique_computer_ids {
        candidate_total_cost += computers_costs[computer_id] as f64;
    }

    // Calculate hard score (resource violations)
    let mut consumed_resources = vec![[0i64; 3]; computers_info.len()];
    for (process_id, computer_id) in sample_computer_ids.iter().enumerate() {
        for resource_type in 0..3 {
            consumed_resources[*computer_id][resource_type] += processes_info[process_id][resource_type];
        }
    }

    let mut resources_unacceptance = 0.0;
    for (computer_id, computer_resources) in computers_info.iter().enumerate() {
        for resource_type in 0..3 {
            let excess = consumed_resources[computer_id][resource_type] - computer_resources[resource_type];
            if excess > 0 {
                resources_unacceptance += excess as f64;
            }
        }
    }

    (resources_unacceptance, candidate_total_cost)
}