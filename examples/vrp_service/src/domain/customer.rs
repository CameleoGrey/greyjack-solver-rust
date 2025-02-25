

use std::{collections::HashMap, hash::Hash};
use std::fmt::Display;
use greyjack::utils::math_utils::round;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {

    pub id: usize,
    pub vec_id: usize,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub demand: u64,
    pub time_window_start: u64,
    pub time_window_end: u64,
    pub service_time: u64,
    pub distances_to_other_customers_map: Option<HashMap<String, f64>>,
    pub frozen: bool,

}

impl Customer {
    
    pub fn new(
        id: usize,
        vec_id: usize,
        name: String,
        latitude: f64,
        longitude: f64,
        demand: u64,
        time_window_start: u64,
        time_window_end: u64,
        service_time: u64,
        distances_to_other_customers_map: Option<HashMap<String, f64>>,
        frozen: bool,

    ) -> Self {

        Self {
            id: id,
            vec_id: vec_id,
            name: name,
            latitude: latitude,
            longitude: longitude,
            demand: demand,
            time_window_start: time_window_start,
            time_window_end: time_window_end,
            service_time: service_time,
            distances_to_other_customers_map: distances_to_other_customers_map,
            frozen: frozen,
        }
    }

    pub fn get_distance_to_other_customer(&self, other_customer: &Customer) -> f64 {

        let mut distance;
        match self.distances_to_other_customers_map {
            None => distance = ((other_customer.latitude - self.latitude).powf(2.0) + (other_customer.longitude - self.longitude).powf(2.0)).sqrt(),
            _ => distance = (self.distances_to_other_customers_map.as_ref().unwrap()[&other_customer.name].clone() as f64),
            
        }

        distance = round(distance, 3);

        return distance;
    }

}

/*
def __str__(self):
         return "customer id: " + str(self.id) + " | " + self.name + ": " + "lat=" + str(self.latitude) + ", " + "lon=" + str(self.longitude)
*/

impl Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "customer id: {} | {}:  | lat= {}, lon= {}", self.id, self.name, self.latitude, self.longitude)
    }
}

impl PartialEq for Customer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Customer {}

impl Hash for Customer {
    
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }

}