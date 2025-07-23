

use greyjack::score_calculation::score_calculators::PlainScoreCalculator;
use greyjack::score_calculation::scores::HardMediumSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::{collections::HashMap, ops::RangeTo};
use polars::prelude::*;


pub struct VRPPlainScoreCalculator {

}

impl VRPPlainScoreCalculator {

    pub fn new() -> PlainScoreCalculator<UtilityObjectVariants, HardMediumSoftScore> {

        let mut score_calculator: PlainScoreCalculator<UtilityObjectVariants, HardMediumSoftScore> = PlainScoreCalculator::new();

        score_calculator.add_prescoring_function("build_common_df".to_string(), Box::new(Self::build_common_df));

        score_calculator.add_constraint("no_duplicating_stops_constraint".to_string(), Box::new(Self::no_duplicating_stops_constraint));
        score_calculator.add_constraint("capacity_constraint".to_string(), Box::new(Self::capacity_constraint));
        score_calculator.add_constraint("minimize_distance".to_string(), Box::new(Self::minimize_distance));
        score_calculator.add_constraint("late_arrival_penalty".to_string(), Box::new(Self::late_arrival_penalty));

        return score_calculator;
    }

    fn build_common_df(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) {
        let customers_df = problem_fact_dfs["customers"].clone().lazy();
        let vehicle_df = problem_fact_dfs["vehicles"].clone().lazy();
        let planning_stops_df = planning_entity_dfs["planning_stops"].clone().lazy();

        let common_df = planning_stops_df
            .with_row_index("index", None)
            .join(vehicle_df, [col("vehicle_id")], [col("vehicle_id")],  JoinArgs::new(JoinType::Inner))
            .join(customers_df, [col("customer_id")], [col("customer_id")],  JoinArgs::new(JoinType::Inner))
            .sort(["sample_id", "vehicle_id", "index"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        utility_objects.insert("common_df".to_string(), UtilityObjectVariants::DataFrame(common_df));

    }

    fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let planning_stops_df = planning_entity_dfs["planning_stops"].clone();

        let duplicate_counts =
            planning_stops_df
            .lazy()
            .group_by(["sample_id"])
            .agg([(col("customer_id").count() - col("customer_id").n_unique()).alias("duplicates_count")])
            .group_by(["sample_id"])
            .agg([col("duplicates_count").sum().cast(DataType::Float64)])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let scores: Vec<HardMediumSoftScore> = duplicate_counts["duplicates_count"]
                                        .f64()
                                        .unwrap()
                                        .into_iter()
                                        //multiply by 1000.0 to competing with capacity constraint
                                        .map(|x| HardMediumSoftScore::new(1000.0 * x.unwrap(), 0.0, 0.0))
                                        .collect();

        return scores;

    }

    fn capacity_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let vehicle_df = problem_fact_dfs["vehicles"].clone().lazy();
        let common_df: DataFrame;
        match &utility_objects["common_df"] {
            UtilityObjectVariants::DataFrame(df) => common_df = df.clone(),
            _ => panic!("There are the dragons")
        }

        let capacity_penalties = 
            common_df
            .clone()
            .lazy()
            .group_by(["sample_id", "vehicle_id"])
            .agg([col("demand").sum().alias("sum_trip_demand")])
            .join(vehicle_df, [col("vehicle_id")], [col("vehicle_id")], JoinArgs::new(JoinType::Inner))
            .with_column((col("capacity").cast(DataType::Int64) - col("sum_trip_demand").cast(DataType::Int64)).alias("capacity_difference"))
            .filter(col("capacity_difference").lt(lit(0)))
            .group_by(["sample_id"])
            .agg([col("capacity_difference").abs().sum().alias("capacity_constraint_penalty").cast(DataType::Float64)])
            .collect()
            .unwrap();

        let bad_sample_ids: Vec<usize> = capacity_penalties["sample_id"].as_materialized_series().u64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();
        let capacity_penalties: Vec<f64> = capacity_penalties["capacity_constraint_penalty"].as_materialized_series().f64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();
        let samples_count = common_df["sample_id"].n_unique().unwrap();

        let mut scores = vec![HardMediumSoftScore::new(0.0, 0.0, 0.0); samples_count.clone()];

        for (i, bad_sample_id) in bad_sample_ids.iter().enumerate() {
            scores[*bad_sample_id].hard_score = capacity_penalties[i];
        }

        return scores;

    }


    fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let common_df: DataFrame;
        match &utility_objects["common_df"] {
            UtilityObjectVariants::DataFrame(df) => common_df = df.clone(),
            _ => panic!("There are the dragons")
        }

        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("There are the dragons"),
        }

        let scores: Vec<HardMediumSoftScore> = 
            common_df
            .partition_by(["sample_id"], false).unwrap()
            .iter().map(|sample_df| {

                let vehicle_distances: Vec<HardMediumSoftScore> = sample_df
                .partition_by(["vehicle_id"], false).unwrap()
                .iter().map( |vehicle_df| {

                    let planning_stop_ids: Vec<usize> = vehicle_df["customer_id"].as_materialized_series().i64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();
                    let depot_vec_id: usize = vehicle_df["depot_vec_id"].get(0).unwrap().try_extract().unwrap();
                    let mut current_distance = 0.0;
                    let n_stops = planning_stop_ids.len() - 1; 
                    current_distance += distance_matrix[depot_vec_id][planning_stop_ids[0]];
                    current_distance += distance_matrix[planning_stop_ids[n_stops]][depot_vec_id];
                    current_distance += (1..planning_stop_ids.len()).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[planning_stop_ids[i-1]][planning_stop_ids[i]]);

                    HardMediumSoftScore::new(0.0, 0.0, current_distance)
                }).collect();

                let sum_vehicles_distance: HardMediumSoftScore = vehicle_distances.iter().fold(
                    HardMediumSoftScore::new(0.0, 0.0, 0.0), 
                    |total_distance, current_distance| total_distance + current_distance.clone()
                );
                return sum_vehicles_distance;
            }).collect();

        return scores;
    }


    fn late_arrival_penalty(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardMediumSoftScore> {

        let common_df: DataFrame;
        match &utility_objects["common_df"] {
            UtilityObjectVariants::DataFrame(df) => common_df = df.clone(),
            _ => panic!("There are the dragons")
        }

        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("There are the dragons"),
        }

        let scores: Vec<HardMediumSoftScore> = 
            common_df
            .partition_by(["sample_id"], false).unwrap()
            .iter().map(|sample_df| {

                let vehicle_time_penalties: Vec<HardMediumSoftScore> = sample_df
                .partition_by(["vehicle_id"], false).unwrap()
                .iter().map( |vehicle_df| {
                    
                    let work_day_start: u64 = vehicle_df["work_day_start"].get(0).unwrap().try_extract().unwrap();
                    let work_day_end: u64 = vehicle_df["work_day_end"].get(0).unwrap().try_extract().unwrap();
                    let customer_start_windows: Vec<u64> = vehicle_df["time_window_start"].as_materialized_series().u64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();
                    let customer_end_windows: Vec<u64> = vehicle_df["time_window_end"].as_materialized_series().u64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();
                    let customer_service_times: Vec<u64> = vehicle_df["service_time"].as_materialized_series().u64().unwrap().to_vec().iter().map(|x| x.unwrap()).collect();

                    let mut time_penalty = 0.0;
                    let n_stops = customer_start_windows.len() - 1;
                    let mut current_arrival_time = work_day_start;
                    for i in 0..n_stops {
                        current_arrival_time = std::cmp::max(current_arrival_time, customer_start_windows[i]);
                        if current_arrival_time > customer_end_windows[i] + customer_service_times[i] {
                            time_penalty += (current_arrival_time - (customer_end_windows[i] + customer_service_times[i])) as f64;
                        }

                        current_arrival_time += customer_service_times[i];
                    }

                    if current_arrival_time > work_day_end {
                        time_penalty += (current_arrival_time - work_day_end) as f64;
                    }

                    HardMediumSoftScore::new(0.0, time_penalty, 0.0)
                }).collect();

                let sum_vehicle_time_penalty: HardMediumSoftScore = vehicle_time_penalties.iter().fold(
                    HardMediumSoftScore::new(0.0, 0.0, 0.0), 
                    |total_time_penalty, current_time_penalty| total_time_penalty + current_time_penalty.clone()
                );
                return sum_vehicle_time_penalty;
            }).collect();

        return scores;
    }
}