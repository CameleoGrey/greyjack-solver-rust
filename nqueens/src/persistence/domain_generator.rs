

use crate::domain::ChessBoard;
use crate::domain::Queen;
use crate::domain::Position;

use std::collections::HashMap;
use rand::*;
use rand::rngs::{StdRng};
use seq::SliceRandom;

pub struct DomainGenerator {

}

impl DomainGenerator {
    pub fn generate_domain(n_queens: u64, random_seed: u64) -> ChessBoard {

        let mut random_row_ids: Vec<u64> = Vec::new();
        let mut column_ids: Vec<u64> = Vec::new();
        for i in 0..n_queens {
            random_row_ids.push(i);
            column_ids.push(i);
        }
        let mut random_generator = StdRng::seed_from_u64(random_seed);
        random_row_ids.shuffle(&mut random_generator);
        

        let mut queens: Vec<Queen> = Vec::new();
        for i in 0..n_queens {
            let current_row_id = random_row_ids[i as usize];
            let current_column_id = column_ids[i as usize];
            let position = Position::new(current_row_id, current_column_id);
            let current_queen = Queen::new(position.row, position.column);
            queens.push(current_queen);
        }

        let chess_board = ChessBoard::new(n_queens, queens);
        return chess_board;

        
    }
}

unsafe impl Send for DomainGenerator {}