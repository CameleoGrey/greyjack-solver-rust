
use greyjack::greynet_fact_for_struct;
use greyjack::score_calculation::greynet::GreynetFact;
use greyjack::score_calculation::greynet::ModifiableFact;
use std::hash::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use std::any::Any;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GreynetComputer {
    pub computer_id: i64,
    pub cpu_power: i64,
    pub memory_size: i64,
    pub network_bandwidth: i64,
    pub cost: i64,
}

greynet_fact_for_struct!(GreynetComputer);

impl ModifiableFact for GreynetComputer {
    fn set_field(&mut self, field_name: &str, value: f64) {
        match field_name {
            _ => panic!(
                "Attempted to modify an unknown or immutable field: {}",
                field_name
            ),
        }
    }

    fn clone_modifiable(&self) -> Box<dyn ModifiableFact + Send> {
        Box::new(self.clone())
    }
}