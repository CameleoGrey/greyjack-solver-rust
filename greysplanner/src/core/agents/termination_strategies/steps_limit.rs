

use super::termination_strategy_trait::TerminationStrategyTrait;

pub struct StepsLimit {

    steps_limit: u64,
    steps_made: u64

}

impl StepsLimit {

    pub fn update(&mut self) {
        self.steps_made += 1;
    }

}

impl TerminationStrategyTrait for StepsLimit {

    fn is_accomplish(&self) -> bool {
        self.steps_made >= self.steps_limit
    }

    fn get_accomplish_rate(&self) -> f64 {
        (self.steps_made as f64) / (self.steps_limit as f64)
    }
    
}