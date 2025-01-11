

use greysplanner::core::score_calculation::score_calculators::oop_score_calculator::OOPScoreCalculator;
use greysplanner::core::score_calculation::scores::simple_score::SimpleScore;
use crate::persistence::cotwin_builder::UtilityObjectVariants;
use std::collections::HashMap;
use polars::prelude::*;

pub struct NQueensScoreCalculator {

}

impl NQueensScoreCalculator {
    pub fn new() -> OOPScoreCalculator<UtilityObjectVariants, SimpleScore> {
        let mut score_calculator= OOPScoreCalculator::new();

        score_calculator.add_constraint("different_rows".to_string(), Box::new(Self::different_rows));
        score_calculator.add_constraint("different_descending_diagonals".to_string(), Box::new(Self::different_descending_diagonals));
        score_calculator.add_constraint("different_ascending_diagonals".to_string(), Box::new(Self::different_ascending_diagonals));

        return score_calculator;
    }

    fn different_rows(planning_entity_dfs: &mut HashMap<String, DataFrame>, problem_fact_dfs: &HashMap<String, DataFrame>) -> Vec<SimpleScore> {

        // clone() is cheap operation for Polars DataFrame (see docs and forums)
        let queens_df = planning_entity_dfs.get("queens").unwrap().clone();
        let same_row_id_counts =
            queens_df
            .lazy()
            .group_by([col("sample_id"), col("row_id")])
            .agg([(col("queen_id").count() - lit(1)).alias("conflicts_count")])
            .group_by([col("sample_id")])
            .agg([col("conflicts_count").sum().cast(DataType::Float64)])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let scores: Vec<SimpleScore> = same_row_id_counts["conflicts_count"]
                                    .as_materialized_series()
                                    .f64()
                                    .unwrap()
                                    .into_iter()
                                    .map(|x| SimpleScore::new(x.unwrap()))
                                    .collect();

        return scores;
    }

    fn different_descending_diagonals(planning_entity_dfs: &mut HashMap<String, DataFrame>, problem_fact_dfs: &HashMap<String, DataFrame>) -> Vec<SimpleScore> {

        // clone() is cheap operation for Polars DataFrame (see docs and forums)
        let queens_df = planning_entity_dfs.get("queens").unwrap().clone();
        let same_row_id_counts =
            queens_df
            .lazy()
            .with_columns([(col("column_id") + col("row_id")).alias("desc_id")])
            .group_by(["sample_id", "desc_id"])
            .agg([(col("queen_id").count() - lit(1)).alias("conflicts_count")])
            .group_by(["sample_id"])
            .agg([col("conflicts_count").sum().cast(DataType::Float64)])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let scores: Vec<SimpleScore> = same_row_id_counts["conflicts_count"]
                                    .as_materialized_series()
                                    .f64()
                                    .unwrap()
                                    .into_iter()
                                    .map(|x| SimpleScore::new(x.unwrap()))
                                    .collect();

        return scores;
    }

    fn different_ascending_diagonals(planning_entity_dfs: &mut HashMap<String, DataFrame>, problem_fact_dfs: &HashMap<String, DataFrame>) -> Vec<SimpleScore> {

        // clone() is cheap operation for Polars DataFrame (see docs and forums)
        let queens_df = planning_entity_dfs.get("queens").unwrap().clone();
        let same_row_id_counts =
            queens_df
            .lazy()
            .with_columns([(col("column_id") - col("row_id")).alias("asc_id")])
            .group_by(["sample_id", "asc_id"])
            .agg([(col("queen_id").count() - lit(1)).alias("conflicts_count")])
            .group_by(["sample_id"])
            .agg([col("conflicts_count").sum().cast(DataType::Float64)])
            .sort(["sample_id"], SortMultipleOptions::default())
            .collect()
            .unwrap();

        let scores: Vec<SimpleScore> = same_row_id_counts["conflicts_count"]
                                    .as_materialized_series()
                                    .f64()
                                    .unwrap()
                                    .into_iter()
                                    .map(|x| SimpleScore::new(x.unwrap()))
                                    .collect();

        return scores;
    }
}