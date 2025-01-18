

use ndarray::Array1;

use crate::agents::termination_strategies::TerminationStrategiesVariants;
use crate::agents::termination_strategies::TerminationStrategiesVariants::*;
use crate::agents::termination_strategies::TerminationStrategyTrait;
use crate::score_calculation::score_requesters::OOPScoreRequester;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use crate::agents::metaheuristic_bases::MetaheuristicBaseTrait;
use crate::cotwin::CotwinEntityTrait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::AddAssign;
use crossbeam_channel::*;

use super::AgentToAgentUpdate;
use super::AgentToSolverUpdate;
use chrono::*;

#[derive(Clone, Copy)]
pub enum AgentStatuses {
    Alive,
    Dead
}

pub struct Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub migration_rate: f64, 
    pub migration_frequency: usize, 
    pub termination_strategy: TerminationStrategiesVariants<ScoreType>,

    pub agent_id: usize,
    pub population_size: usize,
    pub population: Vec<Individual<ScoreType>>,
    pub current_top_individual: Individual<ScoreType>,
    
    // for future Python/Rust version:
    // remove cotwin from requester, place there only address of python replier. How to build VariablesManager without double borrowing of cotwin?
    // score_replier: &'b mut Cotwin<EntityVariants, UtilityObjectVariants, ScoreType>,
    pub score_requester: OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>,
    pub score_precision: Option<Vec<usize>>,
    pub metaheuristic_base: Box<dyn MetaheuristicBaseTrait<ScoreType> + Send>,
    
    pub steps_to_send_updates: usize,
    pub agent_status: AgentStatuses,
    pub round_robin_status_vec: Vec<AgentStatuses>,

    pub updates_to_agent_sender: Option<Sender<AgentToAgentUpdate>>,
    pub updates_for_agent_receiver: Option<Receiver<AgentToAgentUpdate>>

}

impl<EntityVariants, UtilityObjectVariants, ScoreType> 
Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    pub fn new(
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>,
        population_size: usize,
        score_requester: OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>,
        metaheuristic_base: Box<dyn MetaheuristicBaseTrait<ScoreType> + Send>
    ) -> Agent<EntityVariants, UtilityObjectVariants, ScoreType> {

        // agent_id, round_robin_status_dict and channels will be set by Solver, not by agent 
        Self {
            migration_rate: migration_rate,
            migration_frequency: migration_frequency,
            termination_strategy: termination_strategy,

            agent_id: 777777777, // setups by Solver
            population_size: population_size,
            population: Vec::new(),
            current_top_individual: Individual::new(Array1::default(1), ScoreType::get_stub_score()),
            
            score_requester: score_requester,
            score_precision: None, // setups by Solver
            metaheuristic_base: metaheuristic_base,
            
            steps_to_send_updates: migration_frequency,
            agent_status: AgentStatuses::Alive,
            round_robin_status_vec: Vec::new(), // setups by Solver
            updates_to_agent_sender: None, // setups by Solver
            updates_for_agent_receiver: None // setups by Solver
        }
    }

    pub fn solve(&mut self) {

        self.init_population();
        self.population.sort();
        self.update_top_individual();
        self.update_termination_strategy();
        let mut step_id:u64 = 0;

        loop {

            match self.agent_status {
                AgentStatuses::Alive => self.step(),
                AgentStatuses::Dead => (),
            }
            step_id += 1;

            self.population.sort();
            self.update_top_individual();
            self.update_termination_strategy();
            println!("{}, {}, {:?}", self.agent_id, step_id, self.current_top_individual.score);
            
            let is_accomplish;
            match &self.termination_strategy {
                StL(steps_limit) => is_accomplish = steps_limit.is_accomplish(),
                SNI(no_improvement) => is_accomplish = no_improvement.is_accomplish(),
                TSL(time_spent_limit) => is_accomplish = time_spent_limit.is_accomplish(),
                ScL(score_limit) => is_accomplish = score_limit.is_accomplish()
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
            StL(steps_limit) => steps_limit.update(),
            SNI(no_improvement) => no_improvement.update(&self.current_top_individual),
            TSL(time_spent_limit) => time_spent_limit.update(),
            ScL(score_limit) => score_limit.update(&self.current_top_individual)
        }
    }

    fn update_agent_status(&mut self) {

        let is_accomplish;
        match &self.termination_strategy {
            StL(steps_limit) => is_accomplish = steps_limit.is_accomplish(),
            SNI(no_improvement) => is_accomplish = no_improvement.is_accomplish(),
            TSL(time_spent_limit) => is_accomplish = time_spent_limit.is_accomplish(),
            ScL(score_limit) => is_accomplish = score_limit.is_accomplish()
        }

        if is_accomplish {
            self.agent_status = AgentStatuses::Dead;
            self.round_robin_status_vec.insert(self.agent_id, self.agent_status);
        }
    }

    fn step(&mut self) {

        //let start_time = chrono::Utc::now().timestamp_millis();
        let samples: Vec<Array1<f64>> = self.metaheuristic_base.sample_candidates(&mut self.population, &self.current_top_individual, &mut self.score_requester.variables_manager);
        //println!("sampling time: {}", chrono::Utc::now().timestamp_millis() - start_time );

        //let start_time = chrono::Utc::now().timestamp_millis();
        let scores = self.score_requester.request_score(&samples);
        //println!("scoring time: {}", chrono::Utc::now().timestamp_millis() - start_time );
        
        //let start_time = chrono::Utc::now().timestamp_millis();
        let mut candidates: Vec<Individual<ScoreType>> = Vec::new();
        for i in 0..samples.len() {
            candidates.push(Individual::new(samples[i].to_owned(), scores[i].to_owned()));
        }
        self.population = self.metaheuristic_base.build_updated_population(&self.population, &candidates);
        //println!("update population time: {}", chrono::Utc::now().timestamp_millis() - start_time );
    }

    fn send_updates(&mut self) {

    }

    fn receive_updates(&mut self) {

    }

    fn send_publication_to_solver(&self) {

    }
        
}

unsafe impl<EntityVariants, UtilityObjectVariants, ScoreType> Send for 
Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {}