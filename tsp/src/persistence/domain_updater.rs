

use crate::domain::TravelSchedule;
use greyjack::score_calculation::scores::HardSoftScore;
use polars::datatypes::AnyValue;
use serde_json::*;


pub struct DomainUpdater {
    
}

impl DomainUpdater {

    pub fn update_domain(domain: &mut TravelSchedule, solution: Value) {

        let solution: (Vec<(String, AnyValue)>, HardSoftScore)  = serde_json::from_value(solution).unwrap();
        let score = solution.1;
        let solution = solution.0;

        domain.vehicle.trip_path = Vec::new();
        (0..solution.len()).for_each(|i| {
            let location_vec_id;
            let parsed_id = &solution[i].1;
            match parsed_id {
                AnyValue::Int64(x) => location_vec_id = *x as usize,
                _ => panic!("Invalid datatype. Expecting AnyValue::Int64")
            }

            domain.vehicle.trip_path.push(domain.locations_vec[location_vec_id].clone());
        });
    }
}