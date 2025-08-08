use greyjack::score_calculation::greynet::fact::GreynetFact;
use greyjack::score_calculation::greynet::greynet_traits::ModifiableFact;
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GreynetProcess {
    pub process_id: i64,
    pub cpu_power_req: i64,
    pub memory_size_req: i64,
    pub network_bandwidth_req: i64,
    pub computer_id: i64,
}

impl GreynetFact for GreynetProcess {
    fn fact_id(&self) -> i64 {
        self.process_id
    }

    fn clone_fact(&self) -> Box<dyn GreynetFact> {
        Box::new(self.clone())
    }

    fn hash_fact(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn eq_fact(&self, other: &dyn GreynetFact) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |a| self == a)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ModifiableFact for GreynetProcess {
    fn set_field(&mut self, field_name: &str, value: f64) {
        match field_name {
            "computer_id" => self.computer_id = value as i64,
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