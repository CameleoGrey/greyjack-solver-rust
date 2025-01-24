
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
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    population_size: usize, 
    late_acceptance_size: usize,
    mutation_rate_multiplier: Option<f64>, 
    migration_rate: f64, 
    migration_frequency: usize, 
    termination_strategy: TerminationStrategiesVariants<ScoreType>
}

impl<ScoreType> LateAcceptance<ScoreType>
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Display + Send + Serialize{
    
    pub fn new (
        population_size: usize, 
        late_acceptance_size: usize,
        mutation_rate_multiplier: Option<f64>, 
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>
    ) -> Self {

        Self {
            population_size: population_size, 
            late_acceptance_size: late_acceptance_size,
            mutation_rate_multiplier: mutation_rate_multiplier, 
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
        let semantic_groups_dict = score_requester.variables_manager.semantic_groups_dict.clone();
        let discrete_ids = score_requester.variables_manager.discrete_ids.clone();

        let metaheuristic_base = LateAcceptanceBase::new(self.population_size, self.late_acceptance_size, 
                                                                                 self.mutation_rate_multiplier, 
                                                                                 semantic_groups_dict, discrete_ids);
        let metaheuristic_base = MetaheuristicsBasesVariants::LA(metaheuristic_base);
        
        let agent: Agent<EntityVariants, UtilityObjectVariants, ScoreType> = Agent::new(self.migration_rate, 
                                                                                        self.migration_frequency, self.termination_strategy.clone(), 
                                                                                        self.population_size, score_requester, 
                                                                                        metaheuristic_base);
        
        return agent;

    }

}