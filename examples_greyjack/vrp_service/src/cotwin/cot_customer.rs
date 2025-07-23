

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
    
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        
        let mut object_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        object_vec.push(("customer_id".to_string(), self.customer_vec_id.clone()));
        object_vec.push(("demand".to_string(), self.demand.clone()));
        object_vec.push(("time_window_start".to_string(), self.time_window_start.clone()));
        object_vec.push(("time_window_end".to_string(), self.time_window_end.clone()));
        object_vec.push(("service_time".to_string(), self.service_time.clone()));

        return object_vec;
    }

}