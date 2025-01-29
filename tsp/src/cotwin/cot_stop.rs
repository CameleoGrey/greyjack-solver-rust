

use std::collections::HashMap;

use greysplanner::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotStop<'a> {
    pub stop_id: CotwinValueTypes<'a>,
    pub locations_vec_id: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotStop<'a> {
    
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        
        let mut cot_stop_hash_map: HashMap<String, CotwinValueTypes> = HashMap::new();
        cot_stop_hash_map.insert("stop_id".to_string(), self.stop_id.clone());
        cot_stop_hash_map.insert("location_vec_id".to_string(), self.locations_vec_id.clone());

        return cot_stop_hash_map;
    }

}