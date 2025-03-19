

use super::{GeneticAlgorithm, LateAcceptance, TabuSearch, SimulatedAnnealing};
use crate::score_calculation::scores::ScoreTrait;
use std::ops::{AddAssign, Sub};
use std::fmt::{Debug, Display};
use serde::Serialize;

#[derive(Clone)]
pub enum AgentBuildersVariants<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    GA(GeneticAlgorithm<ScoreType>),
    LA(LateAcceptance<ScoreType>),
    TS(TabuSearch<ScoreType>),
    SA(SimulatedAnnealing<ScoreType>),
}