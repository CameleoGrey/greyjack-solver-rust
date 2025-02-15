

use crate::domain::Queen;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::vec::Vec;

use super::Position;

#[derive(Clone)]
pub struct ChessBoard {
    pub n: u64,
    pub queens: Vec<Queen>
}

impl ChessBoard {
    pub fn new(n: u64, queens: Vec<Queen>) -> Self {
        ChessBoard {
            n: n,
            queens: queens
        }
    }
}

impl Display for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let mut queens_keys: HashSet<String> = HashSet::new();
        for queen in &self.queens {
            let row_id = queen.row.row_id;
            let column_id = queen.column.column_id;
            let queen_key = row_id.to_string() + "_" + &column_id.to_string();
            queens_keys.insert(queen_key);
        }

        let mut board_string: Vec<String> = Vec::new();

        for i in 0..self.n {
          let mut row_string: Vec<String> = Vec::new();
          for j in 0..self.n {
            let position_key = i.to_string() + "_" + &j.to_string();
            if queens_keys.contains(&position_key) {
                row_string.push("+".to_owned());
            } else {
                row_string.push("-".to_owned());   
            }
          }
          row_string.push("\n".to_owned());
          board_string.append(&mut row_string);
        }

        let board_string = board_string.join(" ");

        writeln!(f, "{}", board_string)
    }
}

unsafe impl Send for ChessBoard {}