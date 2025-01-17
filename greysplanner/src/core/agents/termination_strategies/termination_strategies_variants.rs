

use super::steps_limit::StepsLimit;
use super::time_spent_limit::TimeSpentLimit;
use super::score_no_improvement::ScoreNoImprovement;
use super::score_limit::ScoreLimit;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::ops::AddAssign;
use std::fmt::Debug;

pub enum TerminationStrategiesVariants<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {
    StL(StepsLimit),
    SNI(ScoreNoImprovement<ScoreType>),
    TSL(TimeSpentLimit),
    ScL(ScoreLimit<ScoreType>)

}