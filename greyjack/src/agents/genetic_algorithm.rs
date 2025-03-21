
use super::base::agent_base::Agent;
use super::metaheuristic_bases::GeneticAlgorithmBase;
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
pub struct GeneticAlgorithm<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    population_size: usize, 
    crossover_probability: f64, 
    p_best_rate: f64,
    tabu_entity_rate: f64,
    mutation_rate_multiplier: Option<f64>,
    move_probas: Option<Vec<f64>>,
    migration_rate: f64, 
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> GeneticAlgorithm<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    
    pub fn new (
        population_size: usize, 
        crossover_probability: f64, 
        p_best_rate: f64,
        tabu_entity_rate: f64, 
        mutation_rate_multiplier: Option<f64>, 
        move_probas: Option<Vec<f64>>,
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            population_size: population_size, 
            crossover_probability: crossover_probability, 
            p_best_rate: p_best_rate,
            tabu_entity_rate: tabu_entity_rate,
            mutation_rate_multiplier: mutation_rate_multiplier, 
            move_probas: move_probas,
            migration_rate: migration_rate, 
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

        let metaheuristic_base = GeneticAlgorithmBase::new(self.population_size, self.crossover_probability, self.p_best_rate, self.tabu_entity_rate, 
                                                                                 self.mutation_rate_multiplier, self.move_probas.clone(),
                                                                                 semantic_groups_dict, discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::GAB(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(self.migration_rate, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        self.population_size, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}