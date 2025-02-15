

#[derive(Clone, Copy)]
pub struct Column {
    pub column_id: u64  
}

impl Column {
    pub fn new(j: u64) -> Self {
        Column {
            column_id: j
        }
    }
}