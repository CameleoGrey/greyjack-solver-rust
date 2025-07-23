

use super::Location;

#[derive(Debug, Clone)]
pub struct Vehicle {
    
    pub depot: Location,
    pub trip_path: Vec<Location>,

}

impl Vehicle {
    
    pub fn new( depot: Location, trip_path: Vec<Location> ) -> Self {

        Self {
            depot: depot,
            trip_path: trip_path,
        }

    }

}