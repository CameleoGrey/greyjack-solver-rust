

use std::{collections::HashMap, hash::Hash};
use std::fmt::Display;
use greyjack::utils::math_utils::round;

#[derive(Debug, Clone)]
pub struct Location {

    pub id: usize,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub distances_to_other_locations_map: Option<HashMap<String, f64>>,

}

impl Location {
    
    pub fn new(
        id: usize,
        name: String,
        latitude: f64,
        longitude: f64,
        distances_to_other_locations_map: Option<HashMap<String, f64>>,

    ) -> Self {

        Self {
            id: id,
            name: name,
            latitude: latitude,
            longitude: longitude,
            distances_to_other_locations_map: distances_to_other_locations_map,
        }
    }

    pub fn get_distance_to_other_location(&self, other_location: &Location) -> f64 {

        let mut distance;
        match self.distances_to_other_locations_map {
            None => distance = ((other_location.latitude - self.latitude).powf(2.0) + (other_location.longitude - self.longitude).powf(2.0)).sqrt(),
            _ => distance = (self.distances_to_other_locations_map.as_ref().unwrap()[&other_location.name].clone() as f64),
            
        }

        distance = round(distance, 3);

        return distance;
    }

}

/*
def __str__(self):
         return "Location id: " + str(self.id) + " | " + self.name + ": " + "lat=" + str(self.latitude) + ", " + "lon=" + str(self.longitude)
*/

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Location id: {} | {}:  | lat= {}, lon= {}", self.id, self.name, self.latitude, self.longitude)
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Location {}

impl Hash for Location {
    
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }

}