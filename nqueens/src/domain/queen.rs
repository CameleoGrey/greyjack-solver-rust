
use crate::domain::Column;
use crate::domain::Row;

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