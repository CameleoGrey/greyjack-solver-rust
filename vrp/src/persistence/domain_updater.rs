

use std::usize;

use crate::domain::VehicleRoutingPlan;
use greyjack::score_calculation::scores::HardSoftScore;
use polars::datatypes::AnyValue;
use serde_json::*;


pub struct DomainUpdater {
    
}

impl DomainUpdater {

    pub fn update_domain(domain: &mut VehicleRoutingPlan, solution: Value) {

        let solution: (Vec<(String, AnyValue)>, HardSoftScore)  = serde_json::from_value(solution).unwrap();
        let solution = solution.0;

        for i in (0..solution.len()).step_by(2) {
            
            // TODO: think about how to preserve order in cotwin entity representations
            let (encoded_vehicle_id, encoded_customer_id);
            if solution[i].0.contains("vehicle") {
                (encoded_vehicle_id, encoded_customer_id) = (i, i+1);
            } else {
                (encoded_vehicle_id, encoded_customer_id) = (i+1, i);
            }

            let vehicle_id: usize;
            match solution[encoded_vehicle_id].1 {
                AnyValue::Int64(v_i) => vehicle_id = v_i as usize,
                _ => panic!("Dragons")
            }

            let customer_id;
            match solution[encoded_customer_id].1 {
                AnyValue::Int64(c_i) => customer_id = c_i as usize,
                _ => panic!("Dragons")
            }

            let current_customer = domain.customers_vec[customer_id].clone();
            domain.vehicles[vehicle_id].customers.push( current_customer );
        }
    }
}