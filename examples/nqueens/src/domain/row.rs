

#[derive(Clone, Copy)]
pub struct Row {
    pub row_id: u64
}

impl Row {
    pub fn new(i: u64) -> Self{
        Row {
            row_id: i
        }
    }
}