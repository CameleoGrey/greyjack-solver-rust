

use serde_json::Value;
use greyjack::solver::ObserverTrait;

/*
Example of observer. Observer can be almost all, what you want. For example, your web app,
that receives each new global top solution from solver and sends it to frontend, other service, database, etc.
*/

pub struct NQueensObserver {

    observer_id: usize,
}

impl NQueensObserver {
    
    pub fn new(observer_id: usize) -> Self {

        Self {
            observer_id: observer_id,
        }
    }
}

impl ObserverTrait for NQueensObserver {

    fn update(&mut self, solution: Value) {

        println!("Observer {} received solution with of score: {:?}", self.observer_id, solution[1]);
        
    }
    
}

unsafe impl Send for NQueensObserver {
    
}