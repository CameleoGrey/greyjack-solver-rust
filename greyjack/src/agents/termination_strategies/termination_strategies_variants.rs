

use super::StepsLimit;
use super::TimeSpentLimit;
use super::ScoreNoImprovement;
use super::ScoreLimit;
use super::TerminationStrategyTrait;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;

#[derive(Clone)]
pub enum TerminationStrategiesVariants<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {
    StL(StepsLimit),
    SNI(ScoreNoImprovement<ScoreType>),
    TSL(TimeSpentLimit),
    ScL(ScoreLimit<ScoreType>)

}

impl<ScoreType> TerminationStrategiesVariants<ScoreType> 
where 
ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub fn as_trait(&mut self) -> &mut dyn TerminationStrategyTrait {

        match self {
            TerminationStrategiesVariants::StL(gab) => gab,
            TerminationStrategiesVariants::SNI(tsb) => tsb,
            TerminationStrategiesVariants::TSL(lab) => lab,
            TerminationStrategiesVariants::ScL(sab) => sab,


        }
    }
}