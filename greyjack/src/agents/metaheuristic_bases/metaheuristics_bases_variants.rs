

use super::{GeneticAlgorithmBase, LateAcceptanceBase, MetaheuristicBaseTrait, TabuSearchBase, SimulatedAnnealingBase, LSHADEBase};
use crate::score_calculation::scores::ScoreTrait;
use std::fmt::{Debug, Display};
use std::ops::{AddAssign, Sub};
use serde::{Serialize};

pub enum MetaheuristicsBasesVariants<ScoreType>
where 
ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Display + Send + Serialize {
    GAB(GeneticAlgorithmBase),
    LAB(LateAcceptanceBase<ScoreType>),
    TSB(TabuSearchBase),
    SAB(SimulatedAnnealingBase),
    LSH(LSHADEBase<ScoreType>),
    None
}

impl<ScoreType> MetaheuristicsBasesVariants<ScoreType> 
where 
ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Display + Send + Serialize {

    pub fn as_trait(&mut self) -> &mut dyn MetaheuristicBaseTrait<ScoreType> {

        match self {
            MetaheuristicsBasesVariants::None => panic!("Metaheuristic base is not initialized"),
            MetaheuristicsBasesVariants::GAB(gab) => gab,
            MetaheuristicsBasesVariants::TSB(tsb) => tsb,
            MetaheuristicsBasesVariants::LAB(lab) => lab,
            MetaheuristicsBasesVariants::SAB(sab) => sab,
            MetaheuristicsBasesVariants::LSH(lsh) => lsh,


        }
    }
}