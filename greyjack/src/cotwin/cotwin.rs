

use std::collections::HashMap;
use polars::frame::DataFrame;

use crate::score_calculation::score_calculators::PlainScoreCalculator;
use crate::score_calculation::score_calculators::score_calculator_variants::ScoreCalculatorVariants;
use crate::score_calculation::scores::ScoreTrait;
use std::ops::{AddAssign, Sub};


pub struct Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send {
    pub planning_entities: HashMap<String, Vec<EntityVariants>>,
    pub problem_facts: HashMap<String, Vec<EntityVariants>>,
    pub score_calculator: ScoreCalculatorVariants<UtilityObjectVariants, ScoreType>
}

impl<EntityVariants, UtilityObjectVariants, ScoreType> 
Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send {

    pub fn new() -> Self {
        Self {
            planning_entities: HashMap::new(),
            problem_facts: HashMap::new(),
            score_calculator: ScoreCalculatorVariants::None
        }
    }
    
    pub fn add_planning_entities(&mut self, name: String, planning_entities: Vec<EntityVariants>) {
        self.planning_entities.insert(name, planning_entities);
    }

    pub fn add_problem_facts(&mut self, name: String, problem_facts: Vec<EntityVariants>) {
        self.problem_facts.insert(name, problem_facts);
    }

    pub fn add_score_calculator(&mut self, score_calculator: ScoreCalculatorVariants<UtilityObjectVariants, ScoreType>) {
        self.score_calculator = score_calculator;
    }

    pub fn get_score(
        &mut self, 
        planning_entity_dfs: &HashMap<String, DataFrame>, 
        problem_fact_dfs: &HashMap<String, DataFrame>,
        delta_dfs: Option<&HashMap<String, DataFrame>>
    ) -> Vec<ScoreType> {

        match &mut self.score_calculator {
            ScoreCalculatorVariants::PSC(psc) => psc.get_score(planning_entity_dfs, problem_fact_dfs),
            ScoreCalculatorVariants::ISC(isc) => isc.get_score(planning_entity_dfs, problem_fact_dfs, delta_dfs.unwrap()),
            ScoreCalculatorVariants::None => panic!("No score calculators in cotwin. Add plain or incremental calculator in cotwin builder") 
        }
    }

}

unsafe impl<EntityVariants, UtilityObjectVariants, ScoreType> Send for Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign + Send {}