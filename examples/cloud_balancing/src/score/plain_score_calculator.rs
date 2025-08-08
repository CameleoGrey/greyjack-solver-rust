use greyjack::score_calculation::score_calculators::PlainScoreCalculator;
use greyjack::score_calculation::scores::HardSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::HashMap;
use polars::prelude::*;

pub struct CloudBalancingPlainScoreCalculator;

impl CloudBalancingPlainScoreCalculator {
    pub fn new() -> PlainScoreCalculator<UtilityObjectVariants, HardSoftScore> {
        let mut score_calculator = PlainScoreCalculator::new();
        score_calculator.add_constraint("all_in_one_constraint".to_string(), Box::new(Self::all_in_one_constraint));
        score_calculator
    }

    fn all_in_one_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>,
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {
        let processes_df = planning_entity_dfs["processes"].clone();
        let computers_df = problem_fact_dfs["computers"].clone();

        let scores = processes_df
            .lazy()
            .with_columns([
                // Cast to f64 to avoid i64 overflow issues
                col("cpu_power_req").cast(DataType::Float64),
                col("memory_size_req").cast(DataType::Float64),
                col("network_bandwidth_req").cast(DataType::Float64),
                // Ensure computer_id is consistent type for join
                col("computer_id").cast(DataType::UInt64),
            ])
            .group_by(["sample_id", "computer_id"])
            .agg([
                col("cpu_power_req").sum().alias("total_cpu_consumption"),
                col("memory_size_req").sum().alias("total_memory_consumption"),
                col("network_bandwidth_req").sum().alias("total_network_consumption"),
            ])
            .join(
                computers_df.lazy().with_columns([
                    // Cast computer specs to f64 as well
                    col("cpu_power").cast(DataType::Float64),
                    col("memory_size").cast(DataType::Float64),
                    col("network_bandwidth").cast(DataType::Float64),
                    col("cost").cast(DataType::Float64),
                ]), 
                [col("computer_id")], 
                [col("computer_id")], 
                JoinArgs::new(JoinType::Inner)
            )
            .with_columns([
                (col("total_cpu_consumption") - col("cpu_power")).alias("cpu_penalty"),
                (col("total_memory_consumption") - col("memory_size")).alias("memory_penalty"),
                (col("total_network_consumption") - col("network_bandwidth")).alias("network_penalty"),
            ])
            .with_columns([
                when(col("cpu_penalty").gt(lit(0.0)))
                    .then(col("cpu_penalty"))
                    .otherwise(lit(0.0))
                    .alias("cpu_penalty"),
                when(col("memory_penalty").gt(lit(0.0)))
                    .then(col("memory_penalty"))
                    .otherwise(lit(0.0))
                    .alias("memory_penalty"),
                when(col("network_penalty").gt(lit(0.0)))
                    .then(col("network_penalty"))
                    .otherwise(lit(0.0))
                    .alias("network_penalty"),
            ])
            .group_by(["sample_id", "computer_id", "cost"])
            .agg([
                col("cpu_penalty").sum().alias("cpu_penalty"),
                col("memory_penalty").sum().alias("memory_penalty"),
                col("network_penalty").sum().alias("network_penalty"),
            ])
            .group_by(["sample_id"])
            .agg([
                (col("cpu_penalty").sum() + col("memory_penalty").sum() + col("network_penalty").sum())
                    .alias("hard_score"),
                col("cost").sum().alias("soft_score"),
            ])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let hard_scores: Vec<f64> = scores["hard_score"].f64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();
        let soft_scores: Vec<f64> = scores["soft_score"].f64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();

        hard_scores
            .iter()
            .zip(soft_scores.iter())
            .map(|(&hard, &soft)| HardSoftScore::new(hard, soft))
            .collect()
    }
}