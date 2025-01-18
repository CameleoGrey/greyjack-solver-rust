

use super::GeneticAlgorithm;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::AddAssign;
use std::fmt::Debug;

#[derive(Clone)]
pub enum AgentBuildersVariants<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send  {
    GA(GeneticAlgorithm<ScoreType>)
}