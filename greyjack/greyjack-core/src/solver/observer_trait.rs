

use serde_json::Value;


pub trait ObserverTrait {

    fn update(&mut self, solution: Value);

}