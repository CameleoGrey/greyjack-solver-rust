

use std::collections::HashMap;
use greysplanner::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotStop<'a> {
    pub vehicle_id: CotwinValueTypes<'a>,
    pub customer_id: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotStop<'a> {
    
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        
        let mut cot_stop_hash_map: HashMap<String, CotwinValueTypes> = HashMap::new();
        cot_stop_hash_map.insert("vehicle_id".to_string(), self.vehicle_id.clone());
        cot_stop_hash_map.insert("customer_id".to_string(), self.customer_id.clone());

        return cot_stop_hash_map;
    }

}