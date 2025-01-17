
use crate::agents::base::agent_base::Agent;

pub trait TerminationStrategyTrait {

    fn is_accomplish(&self) -> bool;

    fn get_accomplish_rate(&self) -> f64;
}