

use crate::domain::{TravelSchedule, Location, Vehicle};
use greyjack::domain::DomainBuilderTrait;
use greyjack::score_calculation::scores::HardSoftScore;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, Lines};
use regex::Regex;
use greyjack::utils::math_utils::round;
use serde_json::*;
use polars::datatypes::AnyValue;


#[derive(Clone)]

pub struct DomainBuilder {
    tsp_file_path: String
}

impl DomainBuilderTrait<TravelSchedule> for DomainBuilder {
    fn build_domain_from_scratch(&self) -> TravelSchedule {

        let (metadata, mut locations_vec, readed_distance_matrix) = Self::read_tsp_file(&self.tsp_file_path);

        let mut distance_matrix= Vec::new();
        match readed_distance_matrix {
            None => distance_matrix = Self::build_distance_matrix(&locations_vec),
            Some(dm) => {
                for i in 0..locations_vec.len() {
                    let mut distances_to_other_locations: HashMap<String, f64> = HashMap::new();
                    for j in 0..locations_vec.len() {
                        let current_distance = round(dm[i][j], 3);
                        let to_location_name = locations_vec[j].name.clone();
                        distances_to_other_locations.insert(to_location_name, current_distance);
                    }
                    locations_vec[i].distances_to_other_locations_map = Some(distances_to_other_locations);
                    distance_matrix = dm.clone();
                }
            }
        }
        for i in 0..locations_vec.len() {
            for j in 0..locations_vec.len() {
                distance_matrix[i][j] = round(distance_matrix[i][j], 3);
            }
        }


        let depot = locations_vec[0].clone();
        let vehicle = Vehicle{depot: depot, trip_path: Vec::new()};
        let domain_model = TravelSchedule::new(metadata["dataset_name"].clone(), vehicle, locations_vec, distance_matrix);

        return domain_model;


    }

    fn build_from_solution(&self, solution: &Value, initial_domain: Option<TravelSchedule>) -> TravelSchedule {
        let mut domain = self.build_domain_from_scratch();
        let solution: (Vec<(String, AnyValue)>, HardSoftScore)  = from_value(solution.clone()).unwrap();
        let values = solution.0;
        let score = solution.1;

        domain.vehicle.trip_path = Vec::new();
        (0..values.len()).for_each(|i| {
            let location_vec_id;
            let parsed_id = &values[i].1;
            match parsed_id {
                AnyValue::Int64(x) => location_vec_id = *x as usize,
                _ => panic!("Invalid datatype. Expecting AnyValue::Int64")
            }

            domain.vehicle.trip_path.push(domain.locations_vec[location_vec_id].clone());
        });

        return domain;
    }
}

impl DomainBuilder {

    pub fn new(tsp_file_path: &str) -> Self {

        Self {
            tsp_file_path: tsp_file_path.to_string(),
        }

    }

    pub fn read_tsp_file(tsp_file_path: &str) -> (HashMap<String, String>, Vec<Location>, Option<Vec<Vec<f64>>>) {

        let file_pointer = File::open(tsp_file_path).expect(format!("Failed to read {}", tsp_file_path).as_str());
        let file_reader = BufReader::new(file_pointer);
        let mut lines_iter = file_reader.lines().into_iter();

        let metadata: HashMap<String, String> = Self::read_metadata(&mut lines_iter);
        let locations_vec = Self::read_locations_vec(&mut lines_iter);

        let mut readed_distance_matrix: Option<Vec<Vec<f64>>> = None;
        if metadata["distance_type"].contains("EUC_2D") == false {
            readed_distance_matrix = Self::read_distance_matrix(&mut lines_iter);
        }

        return (metadata, locations_vec, readed_distance_matrix);

    }

    fn read_metadata(lines_iter: &mut Lines<BufReader<File>>) -> HashMap<String, String> {

        let mut metadata: HashMap<String, String> = HashMap::new();
        loop {
            let readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("NODE_COORD_SECTION") {
                break;
            }

            if readed_line.contains("NAME") {
                let dataset_name: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let dataset_name = dataset_name[dataset_name.len() - 1].clone();
                let dataset_name = dataset_name.replace("\n", "");
                metadata.insert("dataset_name".to_string(), dataset_name);
            }

            if readed_line.contains("EDGE_WEIGHT_TYPE") {
                let distance_type: Vec<String> = readed_line.split(" ").into_iter().map(|x| x.to_string()).collect();
                let distance_type = distance_type[distance_type.len() - 1].clone();
                let distance_type = distance_type.replace("\n", "");
                metadata.insert("distance_type".to_string(), distance_type);
            }
        }

        return metadata;
    }

    fn read_locations_vec(lines_iter: &mut Lines<BufReader<File>>) -> Vec<Location> {

        let mut locations_vec: Vec<Location> = Vec::new();
        loop {
            let mut readed_line = lines_iter.next().expect("Failed to read next line 1").expect("Failed to read next line 2");
            
            if readed_line.contains("EOF") {
                break;
            }

            readed_line = readed_line.trim().to_string();
            readed_line = Regex::new(r"\s+").unwrap().replace_all(readed_line.clone().as_str(), " ").to_string();

            let location_parts: Vec<&str> = readed_line.split(" ").collect();
            let id: usize = location_parts[0].parse().unwrap();
            let latitude: f64 = location_parts[1].parse().unwrap();
            let longitude: f64 = location_parts[2].parse().unwrap();
            let name: String;
            if location_parts.len() > 3 {
                name = location_parts[3].replace("\n", "");
            } else {
                name = id.to_string();
            }

            let readed_location = Location{
                id: id,
                latitude: latitude,
                longitude: longitude,
                name: name,
                distances_to_other_locations_map: None,
            };
            locations_vec.push(readed_location);
        }

        return locations_vec;

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

    fn build_distance_matrix(locations_vec: &Vec<Location>) -> Vec<Vec<f64>> {

        let mut distance_matrix: Vec<Vec<f64>> = Vec::new();
        for i in 0..locations_vec.len() {
            let mut distance_row: Vec<f64> = Vec::new();
            for j in 0..locations_vec.len() {
                let current_distance = locations_vec[i].get_distance_to_other_location(&locations_vec[j]);
                distance_row.push(current_distance);
            }
            distance_matrix.push(distance_row);
        }

        return distance_matrix;

    }

}

unsafe impl Send for DomainBuilder {
    
}