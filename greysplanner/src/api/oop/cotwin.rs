

use std::collections::HashMap;
use polars::frame::DataFrame;

use crate::core::score_calculation::score_calculators::oop_score_calculator::OOPScoreCalculator;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::ops::AddAssign;


pub struct Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign {
    pub planning_entities: HashMap<String, Vec<EntityVariants>>,
    pub problem_facts: HashMap<String, Vec<EntityVariants>>,
    pub score_calculator: OOPScoreCalculator<UtilityObjectVariants, ScoreType>
}

impl<EntityVariants, UtilityObjectVariants, ScoreType> 
Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign {

    pub fn new() -> Self {
        Self {
            planning_entities: HashMap::new(),
            problem_facts: HashMap::new(),
            score_calculator: OOPScoreCalculator::new()
        }
    }
    
    pub fn add_planning_entities(&mut self, name: String, planning_entities: Vec<EntityVariants>) {
        self.planning_entities.insert(name, planning_entities);
    }

    pub fn add_problem_facts(&mut self, name: String, problem_facts: Vec<EntityVariants>) {
        self.problem_facts.insert(name, problem_facts);
    }

    pub fn add_score_calculator(&mut self, score_calculator: OOPScoreCalculator<UtilityObjectVariants, ScoreType>) {
        self.score_calculator = score_calculator;
    }

    pub fn get_score(&mut self, planning_entity_dfs: &mut HashMap<String, DataFrame>, problem_fact_dfs: &HashMap<String, DataFrame>) -> Vec<ScoreType>{
        self.score_calculator.get_score(planning_entity_dfs, problem_fact_dfs)
    }

}