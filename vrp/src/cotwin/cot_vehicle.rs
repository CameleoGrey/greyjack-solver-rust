

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
    
    fn to_hash_map(&self) -> HashMap<String, CotwinValueTypes> {
        
        let mut hash_map_representation: HashMap<String, CotwinValueTypes> = HashMap::new();
        hash_map_representation.insert("vehicle_id".to_string(), self.vehicle_id.clone());
        hash_map_representation.insert("capacity".to_string(), self.capacity.clone());
        hash_map_representation.insert("depot_vec_id".to_string(), self.depot_vec_id.clone());
        hash_map_representation.insert("work_day_start".to_string(), self.work_day_start.clone());
        hash_map_representation.insert("work_day_end".to_string(), self.work_day_end.clone());

        return hash_map_representation;
    }

}