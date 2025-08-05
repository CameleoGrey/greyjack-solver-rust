
use super::{IncrementalScoreCalculator, PlainScoreCalculator};
use crate::score_calculation::{score_calculators::GreynetScoreCalculator, scores::ScoreTrait};
use std::ops::{AddAssign, Sub};

pub enum ScoreCalculatorVariants<UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send {
    PSC(PlainScoreCalculator<UtilityObjectVariants, ScoreType>),
    ISC(IncrementalScoreCalculator<UtilityObjectVariants, ScoreType>),
    Greynet(GreynetScoreCalculator<ScoreType>),
    None
}