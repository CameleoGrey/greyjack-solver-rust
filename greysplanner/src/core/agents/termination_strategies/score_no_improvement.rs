
use super::termination_strategy_trait::TerminationStrategyTrait;

pub struct ScoreNoImprovement {

    steps_limit: u64,
    steps_made: u64

}

impl ScoreNoImprovement {

    pub fn update(&mut self) {
        self.steps_made += 1;
    }

}

impl TerminationStrategyTrait for ScoreNoImprovement {

    fn is_accomplish(&self, ) -> bool {
        false
    }

    fn get_accomplish_rate(&self) -> f64 {
        (self.steps_made as f64) / (self.steps_limit as f64)
    }
    
}