

use greyjack::cotwin::CotwinEntityTrait;
use greyjack::cotwin::CotwinValueTypes;
use std::collections::HashMap;

pub struct CotQueen<'a> {
    pub queen_id: CotwinValueTypes<'a>,
    pub row_id: CotwinValueTypes<'a>,
    pub column_id: CotwinValueTypes<'a>
}

impl<'a> CotwinEntityTrait for CotQueen<'a> {
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        
        let mut queen_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        queen_vec.push((String::from("queen_id"), self.queen_id.clone()));
        queen_vec.push((String::from("row_id"), self.row_id.clone()));
        queen_vec.push((String::from("column_id"), self.column_id.clone()));

        return queen_vec;

    }
}

