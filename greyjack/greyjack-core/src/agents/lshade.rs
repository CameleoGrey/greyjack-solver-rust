
use super::base::agent_base::Agent;
use super::metaheuristic_bases::LSHADEBase;
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
pub struct LSHADE<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    population_size: usize, // Very task specific. Typically I set 32, 64, 128, 256, etc
    history_archive_size: usize, // recommendation from article: 100
    p_best_rate: f64, // 0.05 - faster convergence, 0.2 - better exploration
    memory_pruning_rate: f64, // My idea to use prunning instead of changing archive size. If != 0, gives faster convergence, but stucks more often
    guarantee_of_change_size: usize, // in article there are always changing at least 1 component of vector, but I found during prototyping, that 0 can be useful too
    initial_f: f64, // recommendation from article: 0.5
    initial_cr: f64, // I set it typically 0.5
    initial_mutation_proba: f64, // 0.5 too
    tabu_entity_rate: f64,
    mutation_rate_multiplier: Option<f64>,
    move_probas: Option<Vec<f64>>,
    migration_rate: f64, 
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> LSHADE<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize {
    
    pub fn new (
        population_size: usize,
        history_archive_size: usize,
        p_best_rate: f64,
        memory_pruning_rate: f64,
        guarantee_of_change_size: usize,
        initial_f: f64,
        initial_cr: f64,
        initial_mutation_proba: f64,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>, 
        move_probas: Option<Vec<f64>>,
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            population_size: population_size,
            history_archive_size: history_archive_size,
            p_best_rate: p_best_rate,
            memory_pruning_rate: memory_pruning_rate,
            guarantee_of_change_size: guarantee_of_change_size,
            initial_f: initial_f,
            initial_cr: initial_cr,
            initial_mutation_proba: initial_mutation_proba,
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

        let metaheuristic_base = LSHADEBase::new(self.population_size,
                                                                        self.history_archive_size,
                                                                        self.p_best_rate,
                                                                        self.memory_pruning_rate,
                                                                        self.guarantee_of_change_size,
                                                                        self.initial_f,
                                                                        self.initial_cr,
                                                                        self.initial_mutation_proba,
                                                                        self.tabu_entity_rate, 
                                                                        self.mutation_rate_multiplier, 
                                                                        self.move_probas.clone(),
                                                                        semantic_groups_dict, 
                                                                        discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::LSH(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(self.migration_rate, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        self.population_size, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}