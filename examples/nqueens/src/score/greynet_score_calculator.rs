use std::collections::HashMap;

use crate::cotwin::greynet_queen::GreynetQueen;
use greyjack::score_calculation::greynet::stream_def::extract_fact;
use greyjack::score_calculation::greynet::{ConstraintBuilder, JoinerType};
use greyjack::score_calculation::score_calculators::GreynetScoreCalculator;
use greyjack::score_calculation::scores::SimpleScore;

pub struct NQueensGreynetScoreCalculator;

impl NQueensGreynetScoreCalculator {
    pub fn new() -> GreynetScoreCalculator<SimpleScore> {
        let mut builder = ConstraintBuilder::<SimpleScore>::new();

        // Constraint 1: No two queens on the same row
        builder
            .add_constraint("Same Row", 1.0)
            .for_each::<GreynetQueen>()
            .join_on_with_comparator(
                builder.for_each::<GreynetQueen>(),
                JoinerType::Equal,
                |q: &GreynetQueen| q.row_id,
                |q: &GreynetQueen| q.row_id,
            )
            .filter_tuple(|tuple| {
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        // Constraint 2: No two queens on the same ascending diagonal
        builder
            .add_constraint("Ascending Diagonal", 1.0)
            .for_each::<GreynetQueen>()
            .join_on_with_comparator(
                builder.for_each::<GreynetQueen>(),
                JoinerType::Equal,
                |q: &GreynetQueen| q.column_id - q.row_id,
                |q: &GreynetQueen| q.column_id - q.row_id,
            )
            .filter_tuple(|tuple| {
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        // Constraint 3: No two queens on the same descending diagonal
        builder
            .add_constraint("Descending Diagonal", 1.0)
            .for_each::<GreynetQueen>()
            .join_on_with_comparator(
                builder.for_each::<GreynetQueen>(),
                JoinerType::Equal,
                |q: &GreynetQueen| q.column_id + q.row_id,
                |q: &GreynetQueen| q.column_id + q.row_id,
            )
            .filter_tuple(|tuple| {
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        GreynetScoreCalculator::new(builder)
    }
}