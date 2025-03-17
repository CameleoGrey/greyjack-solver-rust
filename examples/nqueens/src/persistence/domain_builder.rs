
use greyjack::domain::DomainBuilderTrait;
use greyjack::score_calculation::scores::SimpleScore;
use crate::domain::ChessBoard;
use crate::domain::Queen;
use crate::domain::Position;

use std::collections::HashMap;
use rand::*;
use rand::rngs::{StdRng};
use seq::SliceRandom;
use polars::datatypes::AnyValue;
use serde_json::*;

#[derive(Clone)]
pub struct DomainBuilder {
    n_queens: u64, 
    random_seed: u64
}

impl DomainBuilderTrait<ChessBoard> for DomainBuilder {
    fn build_domain_from_scratch(&self) -> ChessBoard {

        let mut random_row_ids: Vec<u64> = Vec::new();
        let mut column_ids: Vec<u64> = Vec::new();
        for i in 0..self.n_queens {
            random_row_ids.push(i);
            column_ids.push(i);
        }
        let mut random_generator = StdRng::seed_from_u64(self.random_seed);
        random_row_ids.shuffle(&mut random_generator);
        

        let mut queens: Vec<Queen> = Vec::new();
        for i in 0..self.n_queens {
            let current_row_id = random_row_ids[i as usize];
            let current_column_id = column_ids[i as usize];
            let position = Position::new(current_row_id, current_column_id);
            let current_queen = Queen::new(position.row, position.column);
            queens.push(current_queen);
        }

        let chess_board = ChessBoard::new(self.n_queens, queens);
        return chess_board;
        
    }

    fn build_from_solution(&self, solution: &Value, initial_domain: Option<ChessBoard>) -> ChessBoard {
        let mut domain = self.build_domain_from_scratch();
        let solution: (Vec<(String, AnyValue)>, SimpleScore)  = from_value(solution.clone()).unwrap();
        let solution = solution.0;

        (0..solution.len()).for_each(|i| {
            let queen_id = &solution[i].0;
            let queen_id: Vec<&str> = queen_id.split(" ").collect();
            let queen_id: Vec<&str> = queen_id[1].split("-->").collect();
            let queen_id: usize = queen_id[0].parse().unwrap();

            let parsed_row_id = &solution[i].1;
            let row_id;
            match parsed_row_id {
                AnyValue::Int64(x) => row_id = *x as u64,
                _ => panic!("Invalid datatype. Expecting AnyValue::Int64")
            }

            domain.queens[queen_id].row.row_id = row_id;
        });

        return domain;
    }
}

impl DomainBuilder {
    pub fn new(n_queens: u64, random_seed: u64) -> Self {
        Self {
            n_queens: n_queens,
            random_seed: random_seed,
        }
    }
}

unsafe impl Send for DomainBuilder {}