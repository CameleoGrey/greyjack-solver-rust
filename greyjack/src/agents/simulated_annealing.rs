
use super::base::agent_base::Agent;
use super::metaheuristic_bases::SimulatedAnnealingBase;
use super::metaheuristic_bases::MetaheuristicsBasesVariants;
use crate::agents::termination_strategies::TerminationStrategiesVariants;
use crate::score_calculation::score_requesters::OOPScoreRequester;
use crate::score_calculation::scores::ScoreTrait;
use crate::cotwin::CotwinEntityTrait;
use crate::cotwin::Cotwin;
use std::ops::{AddAssign, Sub};
use std::fmt::{Debug, Display};
use serde::Serialize;


#[derive(Clone)]
pub struct SimulatedAnnealing<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    initial_temperature: Vec<f64>,
    cooling_rate: Option<f64>,
    tabu_entity_rate: f64,
    mutation_rate_multiplier: Option<f64>,
    move_probas: Option<Vec<f64>>,
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> SimulatedAnnealing<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    
    pub fn new (
        initial_temperature: Vec<f64>,
        cooling_rate: Option<f64>,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            initial_temperature: initial_temperature,
            cooling_rate: cooling_rate,
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

        let metaheuristic_base = SimulatedAnnealingBase::new(self.initial_temperature.clone(), self.cooling_rate, self.tabu_entity_rate,
                                                                                 self.mutation_rate_multiplier, self.move_probas.clone(),
                                                                                 semantic_groups_dict, discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::SAB(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(1.0, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        1, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}