

use std::collections::HashMap;
use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotStop<'a> {
    pub vehicle_id: CotwinValueTypes<'a>,
    pub customer_id: CotwinValueTypes<'a>,
}

impl<'a> CotwinEntityTrait for CotStop<'a> {
    
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        
        let mut object_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        object_vec.push(("vehicle_id".to_string(), self.vehicle_id.clone()));
        object_vec.push(("customer_id".to_string(), self.customer_id.clone()));

        return object_vec;
    }

}