

use greyjack::score_calculation::score_calculators::IncrementalScoreCalculator;
use greyjack::score_calculation::scores::SimpleScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::{HashMap, HashSet};
use polars::prelude::*;

pub struct NQueensIncrementalScoreCalculator {

}

impl NQueensIncrementalScoreCalculator {
    pub fn new() -> IncrementalScoreCalculator<UtilityObjectVariants, SimpleScore> {
        let mut score_calculator= IncrementalScoreCalculator::new();

        // 5x faster than plain
        score_calculator.add_constraint("all_different".to_string(), Box::new(Self::all_different));

        return score_calculator;
    }

    fn all_different(
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: &HashMap<String, DataFrame>,
        utility_objects: &mut HashMap<String, UtilityObjectVariants>,
    ) -> Vec<SimpleScore> {

        // clone() is cheap operation for Polars DataFrame (see docs and forums)
        let queens_df = planning_entity_dfs.get("queens").unwrap().clone();
        let delta_df: DataFrame = delta_dfs.get("queens").unwrap().clone();

        let native_column_ids: Vec<usize> = queens_df["column_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let native_row_ids: Vec<usize> = queens_df["row_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
        let n_planning_row_ids = native_row_ids.len() as i64;

        let scores: Vec<SimpleScore> = delta_df
        .partition_by(["sample_id"], false).unwrap()
        .iter().map(|sample_df| {

            let candidate_df_row_ids: Vec<usize> = sample_df["candidate_df_row_id"].u64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let delta_row_ids: Vec<usize> = sample_df["row_id"].i64().unwrap().to_vec().iter().map(|id| id.unwrap() as usize).collect();
            let mut changed_row_ids = native_row_ids.clone();
            candidate_df_row_ids.iter().zip(delta_row_ids.iter())
            .for_each(|(df_row_id, new_queen_row_id)| changed_row_ids[*df_row_id] = *new_queen_row_id);

            let unique_row_ids: HashSet<usize> = changed_row_ids.iter().map(|row_id| *row_id).collect();
            let unique_desc_ids: HashSet<usize> = native_column_ids.iter().zip(changed_row_ids.iter()).map(|(col_id, row_id)| *col_id + *row_id).collect();
            let unique_asc_ids: HashSet<i64> = native_column_ids.iter().zip(changed_row_ids.iter()).map(|(col_id, row_id)| *col_id as i64 - *row_id as i64).collect();

            let unique_rows_penalty = (n_planning_row_ids - unique_row_ids.len() as i64) as f64;
            let unique_desc_ids_penalty = (n_planning_row_ids - unique_desc_ids.len() as i64) as f64;
            let unique_asc_ids_penalty = (n_planning_row_ids - unique_asc_ids.len() as i64) as f64;

            SimpleScore::new(unique_rows_penalty + unique_desc_ids_penalty + unique_asc_ids_penalty)
        }).collect();

        return scores;
    }
}