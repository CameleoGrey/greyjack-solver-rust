

use crate::domain::chess_board::ChessBoard;
use crate::domain::queen::Queen;
use crate::domain::position::Position;
use crate::domain::row::Row;
use crate::domain::column::Column;

use std::collections::HashMap;
use rand::*;
use rand::rngs::{StdRng};
use seq::SliceRandom;

pub struct DomainGenerator {

}

impl DomainGenerator {
    pub fn generate_domain(n_queens: u64, random_seed: u64) -> ChessBoard {

        let mut positions_map: HashMap<String, Position> = HashMap::new();
        for i in 0..n_queens {
            for j in 0..n_queens {
                let position_id = i.to_string() + "_" + &j.to_string();
                let position = Position::new(i, j);
                positions_map.insert(position_id, position);
            }
        }

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
            let chosen_position_id = current_row_id.to_string() + "_" + &current_column_id.to_string();
            let chosen_position = positions_map.get(&chosen_position_id).expect("Position on ({current_row_id}, {current_column_id}) doesn't exist");
            let current_queen = Queen::new(chosen_position.row, chosen_position.column);
            queens.push(current_queen);
        }

        let chess_board = ChessBoard::new(n_queens, positions_map, queens);
        return chess_board;

        
    }
}