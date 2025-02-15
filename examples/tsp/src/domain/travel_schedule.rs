

use std::collections::HashSet;

use super::{Location, Vehicle};

#[derive(Clone, Debug)]
pub struct TravelSchedule {
    pub name: String, 
    pub vehicle: Vehicle, 
    pub locations_vec: Vec<Location>, 
    pub distance_matrix: Vec<Vec<f64>>,
}

impl TravelSchedule {

    pub fn new(
        name: String, 
        vehicle: Vehicle, 
        locations_vec: Vec<Location>, 
        distance_matrix: Vec<Vec<f64>>,
    ) -> Self {

        Self {
            name: name,
            vehicle: vehicle,
            locations_vec: locations_vec,
            distance_matrix: distance_matrix,

        }
    }

    pub fn get_unique_stops(&self) -> HashSet<Location> {
        self.vehicle.trip_path.iter().map(|location_id| location_id.clone()).collect::<HashSet<Location>>()
    }

    pub fn get_travel_distance(&self, ) -> f64 {
        
        let depot = &self.vehicle.depot;
        let trip_path = self.vehicle.trip_path.clone();
        assert_ne!(trip_path.len(), 0, "Vehicle trip_path is not initialized. Probably, a TSP task isn't solved yet or domain model isn't updated.");

        let depot_to_first_stop_distance = depot.get_distance_to_other_location( &trip_path[0] );
        let last_stop_to_depot_distance = trip_path[trip_path.len() - 1].get_distance_to_other_location( &depot );
        let mut interim_stops_distance = 0.0;
        for i in 1..trip_path.len() {
            let stop_from = &trip_path[i-1];
            let stop_to = &trip_path[i];
            interim_stops_distance += stop_from.get_distance_to_other_location(&stop_to);
        }
        let travel_distance = depot_to_first_stop_distance + interim_stops_distance + last_stop_to_depot_distance;

        return travel_distance;

    }

    pub fn print_metrics(&self) {
        println!("Solution distance: {}", self.get_travel_distance());
        println!("Unique stops (excluding depot): {}", self.get_unique_stops().len());
    }

    pub fn print_path(&self) {
        println!( "{}", self.build_string_of_path_names() );
        println!( "{}", self.build_string_of_path_ids() );
    }

    pub fn build_string_of_path_names(&self) -> String {
        let mut path_names_string: Vec<String> = Vec::new();
        path_names_string.push(self.vehicle.depot.name.clone());
        for stop in &self.vehicle.trip_path {
            path_names_string.push( stop.name.clone() );
        }
        path_names_string.push(self.vehicle.depot.name.clone());
        let path_names_string = path_names_string.join(" --> ");
        return path_names_string;
    }

    pub fn build_string_of_path_ids(&self) -> String {
        let mut path_ids_string: Vec<usize> = Vec::new();
        path_ids_string.push(self.vehicle.depot.id.clone());
        for stop in &self.vehicle.trip_path {
            path_ids_string.push( stop.id.clone() );
        }
        path_ids_string.push(self.vehicle.depot.id.clone());
        let path_ids_string: Vec<String> = path_ids_string.iter().map(|id| id.to_string()).collect();
        let path_ids_string = path_ids_string.join(" --> ");
        return path_ids_string;
    }

}
