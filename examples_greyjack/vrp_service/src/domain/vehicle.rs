

use super::Customer;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    
    pub depot: Customer,
    pub customers: Vec<Customer>,
    pub depot_vec_id: usize,
    pub work_day_start: u64,
    pub work_day_end: u64,
    pub capacity: u64,
    pub max_stops: usize,

}

impl Vehicle {
    
    pub fn new( 
        depot: Customer, 
        customers: Vec<Customer>, 
        depot_vec_id: usize,
        work_day_start: u64,
        work_day_end: u64,
        capacity: u64,
        max_stops: usize, 
    ) -> Self {

        Self {
            depot: depot,
            customers: customers,
            depot_vec_id: depot_vec_id,
            work_day_start: work_day_start,
            work_day_end: work_day_end,
            capacity: capacity,
            max_stops: max_stops,
        }

    }

}