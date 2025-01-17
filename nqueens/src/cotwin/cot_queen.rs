

use greysplanner::cotwin::CotwinEntityTrait;
use greysplanner::cotwin::CotwinValueTypes;
use std::collections::HashMap;

pub struct CotQueen<'a> {
    pub queen_id: CotwinValueTypes<'a>,
    pub row_id: CotwinValueTypes<'a>,
    pub column_id: CotwinValueTypes<'a>
}

impl<'a> CotwinEntityTrait for CotQueen<'a> {
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        
        let mut queen_hashmap: HashMap<String, CotwinValueTypes> = HashMap::new();
        queen_hashmap.insert(String::from("queen_id"), self.queen_id.clone());
        queen_hashmap.insert(String::from("row_id"), self.row_id.clone());
        queen_hashmap.insert(String::from("column_id"), self.column_id.clone());

        return queen_hashmap;

    }
}

