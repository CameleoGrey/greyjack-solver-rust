
use super::TerminationStrategyTrait;
use chrono::prelude::*;
use crate::agents::base::Individual;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;

#[derive(Clone)]
pub struct ScoreNoImprovement<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    limit_milliseconds: i64,
    start_time: i64,
    current_best_score: ScoreType,
    time_delta: i64

}

impl<ScoreType> ScoreNoImprovement<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub fn new(limit_milliseconds: i64) -> Self{
        Self {
            limit_milliseconds: limit_milliseconds,
            start_time: -1,
            current_best_score: ScoreType::get_stub_score(),
            time_delta: 0
        }
    }

    pub fn update(&mut self, agent_top_individual: &Individual<ScoreType>) {

        if self.start_time == -1 {
            self.start_time = Local::now().timestamp_millis();
            self.current_best_score = agent_top_individual.score.clone();
            return;
        }

        // to prevent updates from migrants
        if self.is_accomplish() {
            return;
        }

        let agent_top_score = &agent_top_individual.score;
        if agent_top_score < &self.current_best_score {
            self.current_best_score = agent_top_score.clone();
            self.start_time = Local::now().timestamp_millis();
            self.time_delta = 0;

        } else {
            self.time_delta = Local::now().timestamp_millis() - self.start_time;
        }
    }

}

impl<ScoreType> TerminationStrategyTrait for ScoreNoImprovement<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn is_accomplish(&self, ) -> bool {
        if self.time_delta >= self.limit_milliseconds {
            return true;
        } else {
            return false;
        }
    }

    fn get_accomplish_rate(&self) -> f64 {
        (self.time_delta as f64) / (self.limit_milliseconds as f64)
    }
    
}