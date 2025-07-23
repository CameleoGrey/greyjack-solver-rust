

use serde_json::Value;
use super::observer_trait::ObserverTrait;

pub trait ObservableTrait {

    fn register_observer(&mut self, observer: Box<dyn ObserverTrait>);

    fn notify_observers(&self, solution: Value);

}