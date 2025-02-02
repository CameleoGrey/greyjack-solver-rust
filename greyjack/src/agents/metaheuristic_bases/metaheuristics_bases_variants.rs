

use super::{GeneticAlgorithmBase, LateAcceptanceBase, TabuSearchBase};
use crate::score_calculation::scores::ScoreTrait;
use std::fmt::{Debug, Display};
use std::ops::AddAssign;
use serde::{Serialize};

pub enum MetaheuristicsBasesVariants<ScoreType>
where 
ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Display + Send + Serialize {
    GAB(GeneticAlgorithmBase),
    LAB(LateAcceptanceBase<ScoreType>),
    TSB(TabuSearchBase),
    None
}