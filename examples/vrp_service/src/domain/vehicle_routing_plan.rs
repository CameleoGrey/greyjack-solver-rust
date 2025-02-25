

use std::collections::HashSet;

use super::{Customer, Vehicle};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VehicleRoutingPlan {
    pub name: String, 
    pub vehicles: Vec<Vehicle>, 
    pub customers_vec: Vec<Customer>, 
    pub depot_vec: Vec<Customer>,
    pub distance_matrix: Vec<Vec<f64>>,
    pub time_windowed: bool,
}

impl VehicleRoutingPlan {

    pub fn new(
        name: String, 
        vehicles: Vec<Vehicle>, 
        customers_vec: Vec<Customer>, 
        distance_matrix: Vec<Vec<f64>>,
        depot_vec: Vec<Customer>,
        time_windowed: bool,
    ) -> Self {

        Self {
            name: name,
            vehicles: vehicles,
            customers_vec: customers_vec,
            distance_matrix: distance_matrix,
            depot_vec: depot_vec,
            time_windowed: time_windowed,
        }
    }

    pub fn get_unique_stops(&self) -> HashSet<Customer> {

        let mut unique_stops = HashSet::new();
        for vehicle in &self.vehicles {
            for customer in &vehicle.customers {
                unique_stops.insert(customer.clone());
            }
        }

        return unique_stops;
    }

    pub fn get_sum_travel_distance(&self) -> f64 {
        let mut sum_distance = 0.0;
        for vehicle in &self.vehicles {
            sum_distance += self.get_trip_distance(vehicle);
        }

        return sum_distance;
    }

    pub fn get_trip_distance(&self, vehicle: &Vehicle) -> f64 {
        
        let depot = &vehicle.depot;
        let trip_path = vehicle.customers.clone();
        if trip_path.len() == 0 {
            return 0.0;
        }

        let depot_to_first_stop_distance = depot.get_distance_to_other_customer( &trip_path[0] );
        let last_stop_to_depot_distance = trip_path[trip_path.len() - 1].get_distance_to_other_customer( &depot );
        let mut interim_stops_distance = 0.0;
        for i in 1..trip_path.len() {
            let stop_from = &trip_path[i-1];
            let stop_to = &trip_path[i];
            interim_stops_distance += stop_from.get_distance_to_other_customer(&stop_to);
        }
        let travel_distance = depot_to_first_stop_distance + interim_stops_distance + last_stop_to_depot_distance;

        return travel_distance;

    }

    pub fn print_metrics(&self) {
        println!("Solution distance: {}", self.get_sum_travel_distance());
        println!("Unique stops (excluding depot): {}", self.get_unique_stops().len());
    }

    pub fn get_trip_demand(&self, vehicle: &Vehicle) -> u64 {
        let trip_path = vehicle.customers.clone();
        if trip_path.len() == 0 {
            return 0;
        }

        let mut  trip_demand = 0;
        for customer in trip_path {
            trip_demand += customer.demand;
        }

        return trip_demand;
    }

    pub fn print_trip_paths(&self) {

        for (k, vehicle) in self.vehicles.iter().enumerate() {

            let mut path_names_string = vec![vehicle.depot.name.clone(); 1];
            let mut path_ids_string = vec![vehicle.depot.id.to_string(); 1];
            for cutomer in &vehicle.customers {
                path_names_string.push( cutomer.name.clone() );
                path_ids_string.push( cutomer.id.to_string() );
            }
            path_names_string.push( vehicle.depot.name.clone() );
            path_ids_string.push( vehicle.depot.id.to_string() );

            let path_names_string = path_names_string.join(" --> ");

            let trip_length = self.get_trip_distance( vehicle );
            let trip_demand = self.get_trip_demand( vehicle );

            println!();
            println!( "vehicle {} trip metrics: ", k );
            println!( "Distance: {}", trip_length);
            println!( "Demand / capacity: {} / {}", trip_demand, vehicle.capacity );
            println!( "{}", path_names_string );
            println!();
        }

    }

}
