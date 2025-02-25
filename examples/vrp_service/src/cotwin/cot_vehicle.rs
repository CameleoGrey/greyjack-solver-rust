

use std::collections::HashMap;
use greyjack::cotwin::{CotwinEntityTrait, CotwinValueTypes};

pub struct CotVehicle<'a> {

    vehicle_id: CotwinValueTypes<'a>,
    capacity: CotwinValueTypes<'a>,
    depot_vec_id: CotwinValueTypes<'a>,
    work_day_start: CotwinValueTypes<'a>,
    work_day_end: CotwinValueTypes<'a>,

}

impl<'a> CotVehicle<'a> {
    pub fn new(
        vehicle_id: CotwinValueTypes<'a>,
        capacity: CotwinValueTypes<'a>,
        depot_vec_id: CotwinValueTypes<'a>,
        work_day_start: CotwinValueTypes<'a>,
        work_day_end: CotwinValueTypes<'a>,
    ) -> Self {
        Self {
            vehicle_id: vehicle_id,
            capacity: capacity,
            depot_vec_id,
            work_day_start: work_day_start,
            work_day_end: work_day_end,
        }
    }
}

impl<'a> CotwinEntityTrait for CotVehicle<'a> {
    
    fn to_vec(&self) -> Vec<(String, CotwinValueTypes)> {
        
        let mut object_vec: Vec<(String, CotwinValueTypes)> = Vec::new();
        object_vec.push(("vehicle_id".to_string(), self.vehicle_id.clone()));
        object_vec.push(("capacity".to_string(), self.capacity.clone()));
        object_vec.push(("depot_vec_id".to_string(), self.depot_vec_id.clone()));
        object_vec.push(("work_day_start".to_string(), self.work_day_start.clone()));
        object_vec.push(("work_day_end".to_string(), self.work_day_end.clone()));

        return object_vec;
    }

}