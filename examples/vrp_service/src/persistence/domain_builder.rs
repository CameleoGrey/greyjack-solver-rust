

use crate::domain::{VehicleRoutingPlan, Customer, Vehicle};
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};
use regex::Regex;
use greyjack::domain::DomainBuilderTrait;
use greyjack::utils::math_utils::round;
use greyjack::score_calculation::scores::HardMediumSoftScore;
use serde_json::*;
use polars::datatypes::AnyValue;


#[derive(Clone)]
pub struct DomainBuilder {
    vrp_json: Value,
}

impl DomainBuilderTrait<VehicleRoutingPlan> for DomainBuilder {
    fn build_domain_from_scratch(&self) -> VehicleRoutingPlan {

        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut customers_vec: Vec<Customer> = Vec::new();
        let mut readed_distance_matrix: Option<Vec<Vec<f64>>> = None;
        let mut depot_info: Vec<u64> = Vec::new();

    
        metadata.insert("dataset_name".to_string(), self.vrp_json["metadata"]["dataset_name"].to_string());
        metadata.insert("distance_type".to_string(), self.vrp_json["metadata"]["distance_type"].to_string());
        metadata.insert("task_type".to_string(), self.vrp_json["metadata"]["task_type"].to_string());
        metadata.insert("time_window_task_type".to_string(), self.vrp_json["metadata"]["time_window_task_type"].to_string());
        metadata.insert("vehicles_capacity".to_string(), self.vrp_json["metadata"]["vehicles_capacity"].to_string());
        metadata.insert("vehicles_count".to_string(), self.vrp_json["metadata"]["vehicles_count"].to_string());
        metadata.iter_mut().for_each(|(key, value)| *value = value.replace("\"", ""));
        
        let time_window_task = metadata["time_window_task_type"] == "true";
        let n_customers = self.vrp_json["customers_dict"]["n_customers"].as_u64().unwrap();
        for i in (0..n_customers) {
            let customer_i_json = &self.vrp_json["customers_dict"][i.to_string().as_str()];
            let customer_i = Customer{
                id: customer_i_json["id"].as_u64().unwrap() as usize,
                vec_id: i as usize,
                name: customer_i_json["name"].to_string(),
                latitude: customer_i_json["latitude"].as_f64().unwrap(),
                longitude: customer_i_json["longitude"].as_f64().unwrap(),
                demand: customer_i_json["demand"].as_f64().unwrap() as u64,
                time_window_start: if time_window_task {customer_i_json["time_window_start"].as_u64().unwrap()} else {0},
                time_window_end: if time_window_task {customer_i_json["time_window_end"].as_u64().unwrap()} else {0},
                service_time: if time_window_task {customer_i_json["service_time"].as_u64().unwrap()} else {0},
                distances_to_other_customers_map: None,
                frozen: false,
            };
            customers_vec.push(customer_i);
        }

        let n_depots = self.vrp_json["depot_dict"]["n_depots"].as_u64().unwrap();
        for i in (0..n_depots) {
            depot_info.push( self.vrp_json["depot_dict"][i.to_string().as_str()].as_u64().unwrap() );
        }
        
        let mut distance_matrix= Vec::new();
        match readed_distance_matrix {
            None => distance_matrix = Self::build_distance_matrix(&customers_vec),
            Some(dm) => {
                for i in 0..customers_vec.len() {
                    let mut distances_to_other_customers: HashMap<String, f64> = HashMap::new();
                    for j in 0..customers_vec.len() {
                        let current_distance = round(dm[i][j], 3);
                        let to_customer_name = customers_vec[j].name.clone();
                        distances_to_other_customers.insert(to_customer_name, current_distance);
                    }
                    customers_vec[i].distances_to_other_customers_map = Some(distances_to_other_customers);
                    distance_matrix = dm.clone();
                }
            }
        }
        for i in 0..customers_vec.len() {
            for j in 0..customers_vec.len() {
                distance_matrix[i][j] = round(distance_matrix[i][j], 3);
            }
        }

        let k_vehicles = metadata["vehicles_count"].parse().unwrap();
        let n_depots = depot_info.len();
        let max_stops = customers_vec.len() - n_depots;
        let capacity = metadata["vehicles_capacity"].parse().unwrap();

        let mut vehicles: Vec<Vehicle> = Vec::new();
        for i in 0..k_vehicles {
            let depot_vec_id = i % n_depots;
            let depot = customers_vec[depot_vec_id].clone();
            let work_day_start = depot.time_window_start;
            let work_day_end = depot.time_window_end;
            let customers: Vec<Customer> = Vec::new();
            let vehicle = Vehicle::new(depot, customers, depot_vec_id, work_day_start, work_day_end, capacity, max_stops);
            vehicles.push( vehicle );
        }

        let mut depot_vec: Vec<Customer> = Vec::new();
        for i in 0..n_depots {
            depot_vec.push(customers_vec[i].clone());
        }

        let domain_model = VehicleRoutingPlan::new(
            metadata["dataset_name"].clone(), vehicles, customers_vec, 
            distance_matrix, depot_vec, time_window_task
        );

        return domain_model;
    }

    fn build_from_solution(&self, solution: &Value, initial_domain: Option<VehicleRoutingPlan>) -> VehicleRoutingPlan {

        let mut domain = self.build_domain_from_scratch();
        let solution: (Vec<(String, AnyValue)>, HardMediumSoftScore) = from_value(solution.clone()).unwrap();
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

        return domain;
    }
}

impl DomainBuilder {

    pub fn new(vrp_json: &Value) -> Self {
        Self {
            vrp_json: vrp_json.clone(),
        }
    }

    fn build_distance_matrix(customers_vec: &Vec<Customer>) -> Vec<Vec<f64>> {

        let mut distance_matrix: Vec<Vec<f64>> = Vec::new();
        for i in 0..customers_vec.len() {
            let mut distance_row: Vec<f64> = Vec::new();
            for j in 0..customers_vec.len() {
                let current_distance = customers_vec[i].get_distance_to_other_customer(&customers_vec[j]);
                distance_row.push(current_distance);
            }
            distance_matrix.push(distance_row);
        }

        return distance_matrix;

    }

}