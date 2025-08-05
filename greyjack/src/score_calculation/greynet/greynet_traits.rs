// src/score_calculation/greynet/greynet_traits.rs

//! This module defines traits that your domain entities must implement to be used
//! with the incremental Greynet score calculator.

use crate::score_calculation::greynet::fact::GreynetFact;
use crate::cotwin::CotwinEntityTrait;
use std::rc::Rc;

/// A trait for facts that can have their fields modified by name.
/// This is necessary for the incremental score calculator to apply deltas from the solver.
///
/// You will need to implement this for your planning entity structs. The implementation
/// will likely involve a `match` statement on the `field_name` string to set the
/// corresponding struct field.
///
/// # Example Implementation
/// ```rust,ignore
/// impl ModifiableFact for MyEntity {
///     fn set_field(&mut self, field_name: &str, value: f64) {
///         match field_name {
///             "startTime" => self.start_time = value as i64,
///             "vehicle" => { /* handle assignment of related entity */ },
///             _ => panic!("Unknown field: {}", field_name),
///         }
///     }
///
///     fn clone_modifiable(&self) -> Box<dyn ModifiableFact + Send> {
///         Box::new(self.clone())
///     }
/// }
/// ```
pub trait ModifiableFact: GreynetFact {
    /// Sets a field's value.
    fn set_field(&mut self, field_name: &str, value: f64);

    /// Creates a new `Box<dyn ModifiableFact>` by cloning the current object.
    fn clone_modifiable(&self) -> Box<dyn ModifiableFact + Send>;
}

/// A trait for domain entities that can be converted into an initial, concrete fact
/// for the Greynet engine.
///
/// This trait is used by the `GreynetScoreRequester` to perform the initial load of
/// facts into the Greynet session. Your implementation should read the initial values
/// from the `PlanningVariable` definitions within your entity and create a new, concrete
/// fact instance.
pub trait InitializableFact: CotwinEntityTrait {
    /// Converts the entity definition (which contains PlanningVariables) into a
    /// concrete fact instance with initial values.
    fn to_initialized_fact(&self) -> Rc<dyn ModifiableFact + Send>;
}

