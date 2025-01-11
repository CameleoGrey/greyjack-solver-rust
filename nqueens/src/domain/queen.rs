
use crate::domain::column::Column;
use crate::domain::row::Row;

pub struct Queen {
    pub row: Row,
    pub column: Column
}

impl Queen {
    pub fn new(row: Row, column: Column) -> Self {
        Queen {
            row: row,
            column: column
        }
    }
}