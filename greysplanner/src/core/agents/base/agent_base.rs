

use ndarray::Array1;

use crate::core::agents::termination_strategies::termination_strategies_variants::TerminationStrategiesVariants;
use crate::core::agents::termination_strategies::termination_strategies_variants::TerminationStrategiesVariants::*;
use crate::core::agents::termination_strategies::termination_strategy_trait::TerminationStrategyTrait;
use crate::core::score_calculation::score_requesters::oop_score_requester::OOPScoreRequester;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use crate::core::agents::base::individual::Individual;
use crate::core::agents::metaheuristic_bases::metaheuristic_trait::MetaheuristicBaseTrait;
use crate::api::oop::cotwin_entity_trait::CotwinEntityTrait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::AddAssign;

#[derive(Clone, Copy)]
pub enum AgentStatuses {
    Alive,
    Dead
}

pub struct Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug, {

    pub migration_rate: f64, 
    pub migration_frequency: usize, 
    pub termination_strategy: TerminationStrategiesVariants,

    pub agent_id: usize,
    pub population_size: usize,
    pub population: Vec<Individual<ScoreType>>,
    pub current_top_individual: Individual<ScoreType>,
    
    // for future Python/Rust version:
    // remove cotwin from requester, place there only address of python replier. How to build VariablesManager without double borrowing of cotwin?
    // score_replier: &'b mut Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
    pub score_requester: OOPScoreRequester<'b, EntityVariants, UtilityObjectVariants, ScoreType>,
    pub metaheuristic_base: Box<dyn MetaheuristicBaseTrait<ScoreType>>,
    
    pub steps_to_send_updates: usize,
    pub agent_status: AgentStatuses,
    pub round_robin_status_dict: HashMap<usize, AgentStatuses>,
}

impl<'b, EntityVariants, UtilityObjectVariants, ScoreType> 
Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug {

    pub fn new(
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants,
        population_size: usize,
        score_requester: OOPScoreRequester<'b, EntityVariants, UtilityObjectVariants, ScoreType>,
        metaheuristic_base: Box<dyn MetaheuristicBaseTrait<ScoreType>>
    ) -> Agent<'b, EntityVariants, UtilityObjectVariants, ScoreType> {

        // agent_id, round_robin_status_dict and channels will be set by Solver, not by agent 
        Self {
            migration_rate: migration_rate,
            migration_frequency: migration_frequency,
            termination_strategy: termination_strategy,

            agent_id: 10000000,
            population_size: population_size,
            population: Vec::new(),
            current_top_individual: Individual::new(Array1::default(1), ScoreType::get_stub_score()),
            
            // TODO:
            // move current cotwin to agent, save it until solving, and only on solving start
            // init score_requester and population (to use distinct thread)
            score_requester: score_requester,
            metaheuristic_base: metaheuristic_base,
            
            steps_to_send_updates: migration_frequency,
            agent_status: AgentStatuses::Alive,
            round_robin_status_dict: HashMap::new(),
        }
    }

    pub fn solve(&mut self) {

        self.init_population();
        self.population.sort();
        self.update_top_individual();
        self.update_termination_strategy();
        let mut step_id:u64 = 0;

        loop {
            
            println!("{}, {:?}", step_id, self.current_top_individual.score);

            match self.agent_status {
                AgentStatuses::Alive => self.step(),
                AgentStatuses::Dead => (),
            }
            step_id += 1;

            self.population.sort();
            self.update_top_individual();
            self.update_termination_strategy();
            let is_accomplish;
            match &self.termination_strategy {
                SL(steps_limit) => is_accomplish = steps_limit.is_accomplish(),
                SNI(no_improvement) => is_accomplish = no_improvement.is_accomplish(),
            }

            if is_accomplish {
                self.agent_status = AgentStatuses::Dead;
                break;
            }
            
            /*self.population.sort();
            self.update_top_individual();
            self.update_termination_strategy();

            self.steps_to_send_updates -= 1;
            if self.steps_to_send_updates <= 0 {
                if self.agent_id % 2 == 0 {
                    self.send_updates();
                    self.receive_updates();
                } else {
                    self.receive_updates();
                    self.send_updates();
                }
                self.steps_to_send_updates = self.migration_frequency;
            }

            self.update_agent_status();
            self.send_publication_to_solver();*/

        }

    }

    fn init_population(&mut self) {


        let mut samples:Vec<Array1<f64>> = Vec::new();
        for i in 0..self.population_size {
            let generated_sample = self.score_requester.variables_manager.sample_variables();
            samples.push(generated_sample);
        }
        let scores = self.score_requester.request_score(&samples);

        for i in 0..self.population_size {
            self.population.push(Individual::new(samples[i].clone(), scores[i].clone()));
        }

    }

    fn update_top_individual(&mut self) {
        if &self.population[0] <= &self.current_top_individual {
            self.current_top_individual = self.population[0].clone();
        }
    }

    fn update_termination_strategy(&mut self) {
        
        match &mut self.termination_strategy {
            SL(steps_limit) => steps_limit.update(),
            SNI(no_improvement) => no_improvement.update(),
        }
    }

    fn update_agent_status(&mut self) {

        let is_accomplish;
        match &self.termination_strategy {
            SL(steps_limit) => is_accomplish = steps_limit.is_accomplish(),
            SNI(no_improvement) => is_accomplish = no_improvement.is_accomplish(),
        }

        if is_accomplish {
            self.agent_status = AgentStatuses::Dead;
            self.round_robin_status_dict.insert(self.agent_id, self.agent_status);
        }
    }

    fn step(&mut self) {

        let samples: Vec<Array1<f64>> = self.metaheuristic_base.sample_candidates(&mut self.population, &self.current_top_individual, &mut self.score_requester.variables_manager);
        let scores = self.score_requester.request_score(&samples);
        let mut candidates: Vec<Individual<ScoreType>> = Vec::new();
        for i in 0..samples.len() {
            candidates.push(Individual::new(samples[i].to_owned(), scores[i].to_owned()));
        }
        self.population = self.metaheuristic_base.build_updated_population(&self.population, &candidates);
    }

    fn send_updates(&mut self) {

    }

    fn receive_updates(&mut self) {

    }

    fn send_publication_to_solver(&self) {

    }
        
}