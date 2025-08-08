use crate::cotwin::greynet_queen::GreynetQueen;
use greyjack::score_calculation::greynet::prelude::*;
use greyjack::score_calculation::greynet::stream_def::key;
use greyjack::score_calculation::score_calculators::GreynetScoreCalculator;
use greyjack::score_calculation::scores::SimpleScore;

pub struct NQueensGreynetScoreCalculator;

impl NQueensGreynetScoreCalculator {
    /// Creates a new Greynet score calculator for the N-Queens problem
    /// using the modern, fluent API.
    pub fn new() -> GreynetScoreCalculator<SimpleScore> {
        let mut builder = builder::<SimpleScore>();

        // --- Constraint 1: No two queens on the same row ---
        // A queen conflicts with another queen if they are in the same row.
        builder
            .add_constraint("Same Row", 1.0)
            .for_each::<GreynetQueen>()
            .join_on(
                builder.for_each::<GreynetQueen>(),
                // Left Key: The row_id of the first queen.
                key::first(|q: &GreynetQueen| q.row_id),
                // Right Key: The row_id of the second queen.
                key::first(|q: &GreynetQueen| q.row_id),
            )
            .filter(|tuple| {
                // Extract the two queens from the joined tuple.
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                // Filter to avoid duplicate pairs (q1,q2) and (q2,q1), and self-joins.
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        // --- Constraint 2: No two queens on the same ascending diagonal ---
        // A queen conflicts with another if they share an ascending diagonal.
        // The ascending diagonal is identified by (column - row).
        builder
            .add_constraint("Ascending Diagonal", 1.0)
            .for_each::<GreynetQueen>()
            .join_on(
                builder.for_each::<GreynetQueen>(),
                // Left Key: The ascending diagonal of the first queen.
                key::first(|q: &GreynetQueen| q.column_id - q.row_id),
                // Right Key: The ascending diagonal of the second queen.
                key::first(|q: &GreynetQueen| q.column_id - q.row_id),
            )
            .filter(|tuple| {
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        // --- Constraint 3: No two queens on the same descending diagonal ---
        // A queen conflicts with another if they share a descending diagonal.
        // The descending diagonal is identified by (column + row).
        builder
            .add_constraint("Descending Diagonal", 1.0)
            .for_each::<GreynetQueen>()
            .join_on(
                builder.for_each::<GreynetQueen>(),
                // Left Key: The descending diagonal of the first queen.
                key::first(|q: &GreynetQueen| q.column_id + q.row_id),
                // Right Key: The descending diagonal of the second queen.
                key::first(|q: &GreynetQueen| q.column_id + q.row_id),
            )
            .filter(|tuple| {
                let q1 = extract_fact::<GreynetQueen>(tuple, 0).unwrap();
                let q2 = extract_fact::<GreynetQueen>(tuple, 1).unwrap();
                q1.queen_id < q2.queen_id
            })
            .penalize(|_| SimpleScore::new(1.0));

        GreynetScoreCalculator::new(builder)
    }
}
