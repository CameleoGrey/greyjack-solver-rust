

use std::collections::HashMap;

use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotStop<'a> {
    pub stop_id: CotwinValueTypes<'a>,
    pub locations_vec_id: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotStop<'a> {
    
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        
        let mut cot_stop_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        cot_stop_vec.push(("stop_id".to_string(), self.stop_id.clone()));
        cot_stop_vec.push(("location_vec_id".to_string(), self.locations_vec_id.clone()));

        return cot_stop_vec;
    }

}