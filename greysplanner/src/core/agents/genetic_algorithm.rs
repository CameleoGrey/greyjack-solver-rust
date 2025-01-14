
use super::base::agent_base::Agent;
use super::metaheuristic_bases::genetic_algorithm_base::GeneticAlgorithmBase;
use super::metaheuristic_bases::metaheuristic_trait::MetaheuristicBaseTrait;
use crate::core::agents::termination_strategies::termination_strategies_variants::TerminationStrategiesVariants;
use crate::core::score_calculation::score_requesters::oop_score_requester::OOPScoreRequester;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use crate::api::oop::cotwin_entity_trait::CotwinEntityTrait;
use crate::api::oop::cotwin::Cotwin;
use std::ops::AddAssign;
use std::fmt::Debug;


pub struct GeneticAlgorithm {

}

impl GeneticAlgorithm {
    
    pub fn new<'b, EntityVariants, UtilityObjectVariants, ScoreType> (
        cotwin: &'b mut Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
        population_size: usize, 
        crossover_probability: f64, 
        mutation_rate_multiplier: Option<f64>, 
        p_best_rate: f64,
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants
    ) -> Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType>
    where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {

        let score_requester = OOPScoreRequester::new(cotwin);
        let semantic_groups_dict = score_requester.variables_manager.semantic_groups_dict.clone();
        let discrete_ids = score_requester.variables_manager.discrete_ids.clone();

        let metaheuristic_base = GeneticAlgorithmBase::new(population_size, crossover_probability, 
                                                                                 mutation_rate_multiplier, p_best_rate, 
                                                                                 semantic_groups_dict, discrete_ids);
        
        let agent: Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(migration_rate, 
                                                                                            migration_frequency, termination_strategy, 
                                                                                            population_size, score_requester, 
                                                                                            Box::new(metaheuristic_base));
        
        return agent;
        
    }

}