use greyjack::score_calculation::greynet::fact::GreynetFact;
use greyjack::score_calculation::greynet::greynet_traits::ModifiableFact;
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GreynetQueen {
    pub queen_id: i64,
    pub row_id: i64,
    pub column_id: i64,
}

impl GreynetFact for GreynetQueen {
    fn fact_id(&self) -> i64 {
        self.queen_id
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

impl ModifiableFact for GreynetQueen {
    fn set_field(&mut self, field_name: &str, value: f64) {
        match field_name {
            "row_id" => self.row_id = value as i64,
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