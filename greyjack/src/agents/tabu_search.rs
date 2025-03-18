
use super::base::agent_base::Agent;
use super::metaheuristic_bases::TabuSearchBase;
use super::metaheuristic_bases::MetaheuristicsBasesVariants;
use crate::agents::termination_strategies::TerminationStrategiesVariants;
use crate::score_calculation::score_requesters::OOPScoreRequester;
use crate::score_calculation::scores::ScoreTrait;
use crate::cotwin::CotwinEntityTrait;
use crate::cotwin::Cotwin;
use std::ops::AddAssign;
use std::fmt::{Debug, Display};
use serde::Serialize;


#[derive(Clone)]
pub struct TabuSearch<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    neighbours_count: usize,
    tabu_entity_rate: f64,
    compare_to_global: bool,
    mutation_rate_multiplier: Option<f64>,
    move_probas: Option<Vec<f64>>,
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> TabuSearch<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    
    pub fn new (
        neighbours_count: usize,
        tabu_entity_rate: f64,
        compare_to_global: bool,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            neighbours_count: neighbours_count,
            tabu_entity_rate: tabu_entity_rate,
            compare_to_global: compare_to_global,
            mutation_rate_multiplier: mutation_rate_multiplier,
            move_probas: move_probas,
            migration_frequency: migration_frequency, 
            termination_strategy: termination_strategy
        }
    }

    pub fn build_agent<EntityVariants, UtilityObjectVariants>(
        &self, 
        cotwin: Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>
    )  -> Agent<EntityVariants, UtilityObjectVariants, ScoreType>
    where 
        EntityVariants: CotwinEntityTrait {

        let score_requester = OOPScoreRequester::new(cotwin);
        let semantic_groups_dict = score_requester.variables_manager.semantic_groups_map.clone();
        let discrete_ids = score_requester.variables_manager.discrete_ids.clone();

        let metaheuristic_base = TabuSearchBase::new(self.neighbours_count, self.tabu_entity_rate, self.compare_to_global,
                                                                     self.mutation_rate_multiplier, self.move_probas.clone(), semantic_groups_dict, discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::TSB(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(1.0, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        1, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}