
use super::termination_strategy_trait::TerminationStrategyTrait;
use chrono::prelude::*;

pub struct TimeSpentLimit {

    limit_milliseconds: i64,
    start_time: i64,
    time_delta: i64

}

impl TimeSpentLimit {

    pub fn new(limit_milliseconds: i64) -> Self{
        Self {
            limit_milliseconds: limit_milliseconds,
            start_time: -1,
            time_delta: 0
        }
    }

    pub fn update(&mut self) {

        if self.start_time == -1 {
            self.start_time = Local::now().timestamp_millis();
            return;
        }

        self.time_delta = Local::now().timestamp_millis() - self.start_time;
        
    }

}

impl TerminationStrategyTrait for TimeSpentLimit {

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