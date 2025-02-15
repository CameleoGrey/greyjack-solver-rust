

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
    vrp_file_path: String,
}

impl DomainBuilderTrait<VehicleRoutingPlan> for DomainBuilder {
    fn build_domain_from_scratch(&self) -> VehicleRoutingPlan {
        let (metadata, mut customers_vec, readed_distance_matrix, demand_info, depot_info) = Self::read_vrp_file(&self.vrp_file_path);

        assert_eq!(customers_vec.len(), demand_info.len(), "Customers or demands have been readed incorrect");

        let mut time_window_task = false;
        for i in 0..customers_vec.len() {

            assert_eq!(customers_vec[i].id, demand_info[i][0] as usize, "Invalid customer to demand mapping");
            //customers_vec[i].id = demand_info[i][0] as usize;
            customers_vec[i].demand = demand_info[i][1];

            if demand_info[i].len() == 5 {
                time_window_task = true;
                customers_vec[i].time_window_start = demand_info[i][2];
                customers_vec[i].time_window_end = demand_info[i][3];
                customers_vec[i].service_time = demand_info[i][4];
            }
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

    fn build_from_solution(&self, solution: &Value) -> VehicleRoutingPlan {

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

    pub fn new(vrp_file_path: &str) -> Self {
        Self {
            vrp_file_path: vrp_file_path.to_string(),
        }
    }

    pub fn read_vrp_file(vrp_file_path: &str) -> (HashMap<String, String>, Vec<Customer>, Option<Vec<Vec<f64>>>, Vec<Vec<u64>>, Vec<u64>) {

        let file_pointer = File::open(vrp_file_path).expect(format!("Failed to read {}", vrp_file_path).as_str());
        let file_reader = BufReader::new(file_pointer);
        let mut lines_iter = file_reader.lines().into_iter();

        let metadata: HashMap<String, String> = Self::read_metadata(&mut lines_iter);
        let customers_vec = Self::read_customers_vec(&mut lines_iter);
        
        let mut readed_distance_matrix: Option<Vec<Vec<f64>>> = None;
        if metadata["distance_type"].contains("EUC_2D") == false {
            readed_distance_matrix = Self::read_distance_matrix(&mut lines_iter);
        }

        let demand_info: Vec<Vec<u64>> = Self::read_customers_demand_info(&mut lines_iter);
        let depot_info: Vec<u64> = Self::read_depot_info(&mut lines_iter);

        return (metadata, customers_vec, readed_distance_matrix, demand_info, depot_info);

    }

    fn read_metadata(lines_iter: &mut Lines<BufReader<File>>) -> HashMap<String, String> {

        let mut metadata: HashMap<String, String> = HashMap::new();
        loop {
            let readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("NODE_COORD_SECTION") {
                break;
            }
            
            let readed_line = readed_line.replace("\n", "");
            if readed_line.contains("NAME") {
                let task_name: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let task_name = task_name[task_name.len() - 1].clone();
                let vehicles_count: Vec<String> = task_name.split("-").into_iter().map(|x| x.to_string()).collect();
                let vehicles_count = vehicles_count[vehicles_count.len() - 1].clone();
                let vehicles_count = vehicles_count.replace("k", "");
                metadata.insert("vehicles_count".to_string(), vehicles_count);
            }

            if readed_line.contains("TYPE") {
                let task_type: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let task_type = task_type[task_type.len() - 1].clone();
                metadata.insert("task_type".to_string(), task_type);
            }

            if readed_line.contains("NAME") {
                let dataset_name: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let dataset_name = dataset_name[dataset_name.len() - 1].clone();
                metadata.insert("dataset_name".to_string(), dataset_name);
            }

            if readed_line.contains("EDGE_WEIGHT_TYPE") {
                let distance_type: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let distance_type = distance_type[distance_type.len() - 1].clone();
                metadata.insert("distance_type".to_string(), distance_type);
            }

            if readed_line.contains("CAPACITY") {
                let vehicles_capacity: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let vehicles_capacity = vehicles_capacity[vehicles_capacity.len() - 1].clone();
                metadata.insert("vehicles_capacity".to_string(), vehicles_capacity);
            }
        }

        return metadata;
    }

    fn read_customers_vec(lines_iter: &mut Lines<BufReader<File>>) -> Vec<Customer> {

        let mut customers_counter = 0;
        let mut customers_vec: Vec<Customer> = Vec::new();
        loop {
            let mut readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("EOF") || readed_line.contains("DEMAND_SECTION") {
                break;
            }

            readed_line = readed_line.replace("\n", "");
            readed_line = readed_line.trim().to_string();
            readed_line = Regex::new(r"\s+").unwrap().replace_all(readed_line.clone().as_str(), " ").to_string();

            let customer_parts: Vec<&str> = readed_line.split(" ").collect();
            let id: usize = customer_parts[0].parse().unwrap();
            let latitude: f64 = customer_parts[1].parse().unwrap();
            let longitude: f64 = customer_parts[2].parse().unwrap();
            let name: String;
            if customer_parts.len() > 3 {
                name = customer_parts[3].replace("\n", "");
            } else {
                name = id.to_string();
            }

            let readed_customer = Customer{
                id: id,
                vec_id: customers_counter,
                latitude: latitude,
                longitude: longitude,
                name: name,
                distances_to_other_customers_map: None,
                time_window_start: 0,
                time_window_end: 0,
                service_time: 0,
                demand: 0,
            };
            customers_vec.push(readed_customer);
            customers_counter += 1;
        }

        return customers_vec;
    }

    fn read_distance_matrix(lines_iter: &mut Lines<BufReader<File>>) -> Option<Vec<Vec<f64>>>{

        let mut distance_matrix: Vec<Vec<f64>> = Vec::new();
        loop {
            let mut readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("EOF") {
                break;
            }

            let mut distances_parts: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
            distances_parts.pop();
            let distances_vec: Vec<f64> = distances_parts.iter().map(|x| x.parse().unwrap()).collect();
            distance_matrix.push(distances_vec);
        }

        if distance_matrix.len() == 0 {
            return None;
        }

        return Some(distance_matrix);
    }

    pub fn read_customers_demand_info(lines_iter: &mut Lines<BufReader<File>>) -> Vec<Vec<u64>> {

        let mut demand_info: Vec<Vec<u64>> = Vec::new();
        loop {
            let mut readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("EOF") || readed_line.contains("DEPOT_SECTION"){
                return demand_info;
            }

            readed_line = readed_line.trim().to_string();
            let readed_line: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
            let current_demand_info: Vec<u64> = readed_line.iter().map(|x| x.parse().unwrap()).collect();

            demand_info.push(current_demand_info);
        }
    }

    pub fn read_depot_info(lines_iter: &mut Lines<BufReader<File>>) -> Vec<u64> {

        let mut depot_info: Vec<u64> = Vec::new();
        loop {
            let mut readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("EOF") || readed_line.contains("-1"){
                return depot_info;
            }

            readed_line = readed_line.trim().to_string();
            let readed_depot_id = readed_line.parse().unwrap();
            depot_info.push(readed_depot_id);
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