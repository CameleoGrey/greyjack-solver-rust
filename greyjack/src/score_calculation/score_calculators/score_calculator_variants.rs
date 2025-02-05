
use super::{IncrementalScoreCalculator, PlainScoreCalculator};
use crate::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;

pub enum ScoreCalculatorVariants<UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign {
    PSC(PlainScoreCalculator<UtilityObjectVariants, ScoreType>),
    ISC(IncrementalScoreCalculator<UtilityObjectVariants, ScoreType>),
    None
}