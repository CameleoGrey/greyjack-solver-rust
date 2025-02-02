

use crate::domain::{queen, ChessBoard};
use greyjack::score_calculation::scores::SimpleScore;
use polars::prelude::AnyValue;
use serde_json::Value;
use serde_json;


pub struct DomainUpdater {

}

impl DomainUpdater {

    pub fn update_domain(domain: &mut ChessBoard, solution: Value) {

        let solution: (Vec<(String, AnyValue)>, SimpleScore)  = serde_json::from_value(solution).unwrap();
        let solution = solution.0;

        (0..solution.len()).for_each(|i| {
            let queen_id = &solution[i].0;
            let queen_id: Vec<&str> = queen_id.split(" ").collect();
            let queen_id: Vec<&str> = queen_id[1].split("-->").collect();
            let queen_id: usize = queen_id[0].parse().unwrap();

            let mut parsed_row_id = &solution[i].1;
            let row_id;
            match parsed_row_id {
                AnyValue::Int64(x) => row_id = *x as u64,
                _ => panic!("Invalid datatype. Expecting AnyValue::Int64")
            }

            domain.queens[queen_id].row.row_id = row_id;
        });

    }

}