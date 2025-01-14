

use super::steps_limit::StepsLimit;
use super::score_no_improvement::ScoreNoImprovement;

pub enum TerminationStrategiesVariants {
    SL(StepsLimit),
    SNI(ScoreNoImprovement)
}