
use super::base::agent_base::Agent;
use super::metaheuristic_bases::LateAcceptanceBase;
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
pub struct LateAcceptance<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    late_acceptance_size: usize,
    tabu_entity_rate: f64,
    mutation_rate_multiplier: Option<f64>,
    move_probas: Option<Vec<f64>>,
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> LateAcceptance<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    
    pub fn new (
        late_acceptance_size: usize,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            late_acceptance_size: late_acceptance_size,
            tabu_entity_rate: tabu_entity_rate,
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

        let metaheuristic_base = LateAcceptanceBase::new(self.late_acceptance_size, self.tabu_entity_rate,
                                                                                 self.mutation_rate_multiplier, self.move_probas.clone(),
                                                                                 semantic_groups_dict, discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::LAB(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(1.0, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        1, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}