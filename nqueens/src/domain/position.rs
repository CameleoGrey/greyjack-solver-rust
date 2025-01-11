
use crate::domain::column::Column;
use crate::domain::row::Row;

#[derive(Clone, Copy)]
pub struct Position {
    pub row: Row,
    pub column: Column
}

impl Position {
    pub fn new(i: u64, j: u64) -> Self{
        Position {
            row: Row::new(i),
            column: Column::new(j)
        }
    }
}