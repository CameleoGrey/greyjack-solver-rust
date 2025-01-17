

use super::termination_strategy_trait::TerminationStrategyTrait;
use crate::core::agents::base::individual::Individual;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::ops::AddAssign;
use std::fmt::Debug;

pub struct ScoreLimit<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {

    target_score: ScoreType,
    current_best_score: ScoreType

}

impl<ScoreType> ScoreLimit<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {

    pub fn new(target_score:ScoreType) -> Self {
        Self {
            target_score: target_score,
            current_best_score: ScoreType::get_stub_score()
        }
    }

    pub fn update(&mut self, agent_top_individual: &Individual<ScoreType>) {
        self.current_best_score = agent_top_individual.score.clone();
    }

}

impl<ScoreType> TerminationStrategyTrait for ScoreLimit<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {

    fn is_accomplish(&self) -> bool {
        self.current_best_score <= self.target_score
    }

    fn get_accomplish_rate(&self) -> f64 {
        (self.current_best_score.get_fitness_value()) / (self.target_score.get_fitness_value() + 1e-10)
    }
    
}