

use greyjack::score_calculation::score_calculators::PlainScoreCalculator;
use greyjack::score_calculation::scores::HardSoftScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::HashMap;
use polars::prelude::*;
use ndarray::{Array, Array2};


pub struct TSPPlainScoreCalculator {

}

impl TSPPlainScoreCalculator {

    pub fn new() -> PlainScoreCalculator<UtilityObjectVariants, HardSoftScore> {

        let mut score_calculator: PlainScoreCalculator<UtilityObjectVariants, HardSoftScore> = PlainScoreCalculator::new();

        score_calculator.add_constraint("no_duplicating_stops_constraint".to_string(), Box::new(Self::no_duplicating_stops_constraint));
        score_calculator.add_constraint("minimize_distance".to_string(), Box::new(Self::minimize_distance));

        return score_calculator;
    }

    fn no_duplicating_stops_constraint(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        let path_stops_df = planning_entity_dfs["path_stops"].clone();

        let duplicate_counts =
            path_stops_df
            .lazy()
            .group_by(["sample_id"])
            .agg([(col("location_vec_id").count() - col("location_vec_id").n_unique()).alias("duplicates_count")])
            .group_by(["sample_id"])
            .agg([col("duplicates_count").sum().cast(DataType::Float64)])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let scores: Vec<HardSoftScore> = duplicate_counts["duplicates_count"]
                                        .f64()
                                        .unwrap()
                                        .into_iter()
                                        .map(|x| HardSoftScore::new(x.unwrap(), 0.0))
                                        .collect();

        return scores;

    }


    fn minimize_distance(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<HardSoftScore> {

        let path_stops_df = planning_entity_dfs["path_stops"].clone();
        let distance_matrix: &Vec<Vec<f64>>;
        match &utility_objects["distance_matrix"] {
            UtilityObjectVariants::DistanceMatrix(dm) => distance_matrix = &dm,
            _ => panic!("dragons")
        }

        let scores: Vec<HardSoftScore> = 
            path_stops_df
            .sort(["sample_id"], SortMultipleOptions::default()).unwrap()
            .partition_by(["sample_id"], false).unwrap()
            .iter().map(|sample_df| {
                
                let planning_stop_ids: Vec<usize> = sample_df["location_vec_id"].as_materialized_series().i64().unwrap().to_vec().iter().map(|x| x.unwrap() as usize).collect();
                let mut current_distance = 0.0;
                let n_stops = planning_stop_ids.len() - 1; 
                current_distance += distance_matrix[0][planning_stop_ids[0]];
                current_distance += distance_matrix[planning_stop_ids[n_stops]][0];
                current_distance += (1..planning_stop_ids.len()).into_iter().fold(0.0, |interim_distance, i| interim_distance + distance_matrix[planning_stop_ids[i-1]][planning_stop_ids[i]]);

                HardSoftScore::new(0.0, current_distance)
            }).collect();

        return scores;
    }
}