

use std::collections::HashMap;
use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotCustomer<'a> {

    customer_vec_id: CotwinValueTypes<'a>,
    demand: CotwinValueTypes<'a>,
    time_window_start: CotwinValueTypes<'a>,
    time_window_end: CotwinValueTypes<'a>,
    service_time: CotwinValueTypes<'a>,

}

impl<'a> CotCustomer<'a> {
    pub fn new(
        customer_vec_id: CotwinValueTypes<'a>,
        demand: CotwinValueTypes<'a>,
        time_window_start: CotwinValueTypes<'a>,
        time_window_end: CotwinValueTypes<'a>,
        service_time: CotwinValueTypes<'a>,
    ) -> Self {
        Self {
            customer_vec_id,
            demand: demand,
            time_window_start: time_window_start,
            time_window_end: time_window_end,
            service_time: service_time,
        }
    }
}

impl<'a> CotwinEntityTrait for CotCustomer<'a> {
    
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        
        let mut hash_map_representation: HashMap<String, CotwinValueTypes> = HashMap::new();
        hash_map_representation.insert("customer_id".to_string(), self.customer_vec_id.clone());
        hash_map_representation.insert("demand".to_string(), self.demand.clone());
        hash_map_representation.insert("time_window_start".to_string(), self.time_window_start.clone());
        hash_map_representation.insert("time_window_end".to_string(), self.time_window_end.clone());
        hash_map_representation.insert("service_time".to_string(), self.service_time.clone());

        return hash_map_representation;
    }

}