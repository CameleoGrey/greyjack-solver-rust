

use super::StepsLimit;
use super::TimeSpentLimit;
use super::ScoreNoImprovement;
use super::ScoreLimit;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;
use std::fmt::Debug;

#[derive(Clone)]
pub enum TerminationStrategiesVariants<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {
    StL(StepsLimit),
    SNI(ScoreNoImprovement<ScoreType>),
    TSL(TimeSpentLimit),
    ScL(ScoreLimit<ScoreType>)

}