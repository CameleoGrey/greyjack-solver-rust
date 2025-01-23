

use crate::agents::termination_strategies::TerminationStrategiesVariants;
use crate::agents::termination_strategies::TerminationStrategiesVariants::*;
use crate::agents::termination_strategies::TerminationStrategyTrait;
use crate::score_calculation::score_requesters::OOPScoreRequester;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use crate::agents::metaheuristic_bases::MetaheuristicBaseTrait;
use crate::agents::metaheuristic_bases::MetaheuristicsBasesVariants;
use crate::agents::metaheuristic_bases::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::cotwin::CotwinEntityTrait;
use crate::solver::SolverLoggingLevels;
use crate::solver::observable_trait::ObservableTrait;
use crate::solver::observer_trait::ObserverTrait;
use super::AgentToAgentUpdate;
use super::AgentStatuses;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use std::ops::AddAssign;
use crossbeam_channel::*;
use ndarray::Array1;
use chrono::*;
use polars::datatypes::AnyValue;
use serde::{Serialize};
use serde_json::json;
use serde_json::Value;

pub struct Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send + Serialize {

    pub migration_rate: f64, 
    pub migration_frequency: usize, 
    pub termination_strategy: TerminationStrategiesVariants<ScoreType>,

    pub agent_id: usize,
    pub population_size: usize,
    pub population: Vec<Individual<ScoreType>>,
    pub agent_top_individual: Individual<ScoreType>,
    pub global_top_individual: Arc<Mutex<Individual<ScoreType>>>,
    pub global_top_json: Arc<Mutex<Value>>,
    
    pub score_requester: OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>,
    pub score_precision: Option<Vec<u64>>,
    pub metaheuristic_base: MetaheuristicsBasesVariants,
    
    pub steps_to_send_updates: usize,
    pub agent_status: AgentStatuses,
    pub round_robin_status_vec: Vec<AgentStatuses>,
    pub alive_agents_count: usize,
    pub comparisons_to_global_count: usize,

    pub updates_to_agent_sender: Option<Sender<AgentToAgentUpdate<ScoreType>>>,
    pub updates_for_agent_receiver: Option<Receiver<AgentToAgentUpdate<ScoreType>>>,
    pub solving_start: i64,
    pub step_id: u64,
    pub logging_level: SolverLoggingLevels,
    pub end_work_message_printed: bool,

    pub observers: Arc<Mutex<Option<Vec<Box<dyn ObserverTrait + Send>>>>>,
    pub observers_count: usize,
}

impl<EntityVariants, UtilityObjectVariants, ScoreType> 
Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send + Serialize {

    pub fn new(
        migration_rate: f64, 
        migration_frequency: usize, 
        termination_strategy: TerminationStrategiesVariants<ScoreType>,
        population_size: usize,
        score_requester: OOPScoreRequester<EntityVariants, UtilityObjectVariants, ScoreType>,
        metaheuristic_base: MetaheuristicsBasesVariants,
    ) -> Agent<EntityVariants, UtilityObjectVariants, ScoreType> {

        // agent_id, round_robin_status_dict and channels will be set by Solver, not by agent 
        let global_top_individual: Individual<ScoreType> = Individual::new(Array1::from_vec(vec![1.0]), ScoreType::get_stub_score());
        let global_top_individual = Arc::new(Mutex::new(global_top_individual));
        Self {
            migration_rate: migration_rate,
            migration_frequency: migration_frequency,
            termination_strategy: termination_strategy,

            agent_id: 777777777, // setups by Solver
            population_size: population_size,
            population: Vec::new(),
            agent_top_individual: Individual::new(Array1::default(1), ScoreType::get_stub_score()),
            global_top_individual: global_top_individual,
            global_top_json: Arc::new(Mutex::new(Value::Null)),
            
            
            score_requester: score_requester,
            score_precision: None, // setups by Solver
            metaheuristic_base: metaheuristic_base,
            
            steps_to_send_updates: migration_frequency,
            agent_status: AgentStatuses::Alive,
            round_robin_status_vec: Vec::new(), // setups by Solver
            updates_to_agent_sender: None, // setups by Solver
            updates_for_agent_receiver: None, // setups by Solver
            alive_agents_count: 1, // setups by Solver
            comparisons_to_global_count: 0,
            solving_start: Utc::now().timestamp_millis(),
            step_id: 0,
            logging_level: SolverLoggingLevels::Info,
            end_work_message_printed: false,

            observers: Arc::new(Mutex::new(None)), // setups by Solver
            observers_count: 0 // setups by Solver
        }
    }

    pub fn solve(&mut self) {

        self.init_population();
        self.population.sort();
        self.update_top_individual();
        self.update_termination_strategy();
        self.update_agent_status();
        self.update_alive_agents_count();
        self.solving_start = Utc::now().timestamp_millis();
        self.step_id = 0;

        loop {

            match self.agent_status {
                AgentStatuses::Alive => self.step(),
                AgentStatuses::Dead => (),
            }
            self.step_id += 1;
            
            self.population.sort();
            self.update_top_individual();
            self.update_termination_strategy();
            self.update_agent_status();
            self.update_alive_agents_count();
            if self.alive_agents_count == 0 {
                break;
            }
            
            self.steps_to_send_updates -= 1;
            if self.steps_to_send_updates <= 0 {
                if self.agent_id % 2 == 0 {
                    match self.send_updates() {
                        Err(x) => break,
                        _ => ()
                    }
                    match self.receive_updates() {
                        Err(x) => break,
                        _ => ()
                    }
                } else {
                    match self.receive_updates() {
                        Err(x) => break,
                        _ => ()
                    }
                    match self.send_updates() {
                        Err(x) => break,
                        _ => ()
                    }
                }
                self.steps_to_send_updates = self.migration_frequency;
            }
            
            self.update_global_top();
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
        if &self.population[0] <= &self.agent_top_individual {
            self.agent_top_individual = self.population[0].clone();
        }
    }

    fn update_termination_strategy(&mut self) {
        
        match &mut self.termination_strategy {
            StL(steps_limit) => steps_limit.update(),
            SNI(no_improvement) => no_improvement.update(&self.agent_top_individual),
            TSL(time_spent_limit) => time_spent_limit.update(),
            ScL(score_limit) => score_limit.update(&self.agent_top_individual)
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
            self.round_robin_status_vec[self.agent_id] = self.agent_status;
            
            if self.end_work_message_printed == false {
                match self.logging_level {
                    SolverLoggingLevels::Silent => (),
                    _ => {
                        let end_work_message = format!("Agent {} has successfully terminated work. Now it's just transmitting updates between its neighbours until at least one agent is alive.", self.agent_id);
                        println!("{}", end_work_message);
                    }
                }
                self.end_work_message_printed = true;
                //println!("{}", self.step_id);
            }
        }
    }

    fn update_alive_agents_count(&mut self) {
        self.alive_agents_count = self.round_robin_status_vec.iter().filter(|x| {
            match x {
                AgentStatuses::Alive => true,
                AgentStatuses::Dead => false,
            }
        }).count();
    }

    fn step(&mut self) {

        //Box<dyn MetaheuristicBaseTrait<ScoreType> + Send>

        let metaheuristic_base;
        match &mut self.metaheuristic_base {
            MetaheuristicsBasesVariants::None => panic!("Metaheuristic base is not initialized"),
            MetaheuristicsBasesVariants::GAB(gab) => metaheuristic_base = gab,
        }

        let samples: Vec<Array1<f64>> = metaheuristic_base.sample_candidates(&mut self.population, &self.agent_top_individual, &mut self.score_requester.variables_manager);
        //println!("sampling time: {}", chrono::Utc::now().timestamp_millis() - start_time );

        //let start_time = chrono::Utc::now().timestamp_millis();
        let mut scores = self.score_requester.request_score(&samples);
        match &self.score_precision {
            Some(precision) => scores.iter_mut().for_each(|score| score.round(&precision)),
            None => ()
        }
        //println!("scoring time: {}", chrono::Utc::now().timestamp_millis() - start_time );
        
        //let start_time = chrono::Utc::now().timestamp_millis();
        let mut candidates: Vec<Individual<ScoreType>> = Vec::new();
        for i in 0..samples.len() {
            candidates.push(Individual::new(samples[i].to_owned(), scores[i].to_owned()));
        }
        self.population = metaheuristic_base.build_updated_population(&self.population, &candidates);
        //println!("update population time: {}", chrono::Utc::now().timestamp_millis() - start_time );
    }

    fn send_updates(&mut self) -> Result<usize, usize> {

        // assume that the agent's population is already sorted 
        let migrants_count = (self.migration_rate * (self.population_size as f64)).ceil() as usize;
        let migrants:Vec<Individual<ScoreType>> = (0..migrants_count).map(|i| self.population[i].clone()).collect();
        let round_robin_status_vec = self.round_robin_status_vec.clone();

        let agent_update = AgentToAgentUpdate::new(self.agent_id, migrants, round_robin_status_vec);
        let send_result = self.updates_to_agent_sender.as_mut().unwrap().send(agent_update);
        match send_result {
            Err(e) => {
                match self.logging_level {
                    SolverLoggingLevels::Silent => (),
                    _ => {
                        let error_message = format!("Warning! Failed to send updates by Agent {} due to {e}", self.agent_id);
                        println!("{}", error_message);
                    }
                }
                return Err(1);
            },
            _ => ()
        }

        Ok(0)
    }

    fn receive_updates(&mut self) -> Result<usize, usize> {

        // assume that the agent's population is already sorted

        let received_updates;
        let received_updates_result = self.updates_for_agent_receiver.as_mut().unwrap().recv();
        match received_updates_result {
            Err(e) => {
                match self.logging_level {
                    SolverLoggingLevels::Silent => (),
                    _ => {
                        let error_message = format!("Warning! Failed to receive updates by Agent {} due to {e}", self.agent_id);
                        println!("{}", error_message)
                    },
                }
                return Err(1);
            },
            Ok(updates) => received_updates = updates
        }

        (0..self.round_robin_status_vec.len()).for_each(|i| {
            if i != self.agent_id {
                self.round_robin_status_vec[i] = received_updates.round_robin_status_vec[i];
            }
        });

        let current_agent_kind: MetaheuristicKind;
        match &self.metaheuristic_base {
            MetaheuristicsBasesVariants::None => panic!("Metaheuristic base is not initialized"),
            MetaheuristicsBasesVariants::GAB(gab) => current_agent_kind = gab.metaheuristic_kind.clone(),
        }

        let comparison_ids:Vec<usize>;
        match current_agent_kind {
            MetaheuristicKind::Population => {
                let migrants_count = received_updates.migrants.len();
                comparison_ids = ((self.population_size - migrants_count)..self.population_size).collect();
            },
            MetaheuristicKind::LocalSearch => comparison_ids = (0..received_updates.migrants.len()).collect()
        }

        (0..received_updates.migrants.len()).for_each(|i| {
            if received_updates.migrants[i] <= self.population[comparison_ids[i]] {
                self.population[comparison_ids[i]] = received_updates.migrants[i].clone();
            }
        });

        Ok(0)

    }

    fn update_global_top(&mut self) {
        let mut global_top_individual = self.global_top_individual.lock().unwrap();
            let mut global_top_json = self.global_top_json.lock().unwrap();
            if self.agent_top_individual.score < global_top_individual.score {
                *global_top_individual = self.agent_top_individual.clone();
                *global_top_json = self.convert_to_json(self.agent_top_individual.clone());

                if self.observers_count > 0 {
                    self.notify_observers(global_top_json.clone());
                }
            }
            
            match self.agent_status {
                AgentStatuses::Alive => {
                    match self.logging_level {
                        SolverLoggingLevels::Info => {
                            let solving_time = ((Utc::now().timestamp_millis() - self.solving_start) as f64) / 1000.0;
                            let info_message = format!("{}, Agent: {:3}, Steps: {}, Global best score: {:?}, Solving time: {}", 
                                Local::now().format("%Y-%m-%d %H:%M:%S"), self.agent_id, self.step_id, global_top_individual.score, solving_time);
                            println!("{}", info_message);
                        },
                        _ => (),
                    }
                },
                _ => ()
            }
    }

    pub fn convert_to_json(&self, individual: Individual<ScoreType>) -> Value {

        let inverse_transformed_variables = self.score_requester.variables_manager.inverse_transform_variables(&individual.variable_values);
        let variables_names = self.score_requester.variables_manager.get_variables_names_vec();
        let inverse_transformed_variables: Vec<(String, AnyValue)> = 
        inverse_transformed_variables.iter()
        .zip(variables_names.iter())
        .map(|(x, name)| {
            (name.clone(), x.clone())
        }).collect();
        let individual_json = json!((inverse_transformed_variables, individual.score));
        return individual_json;

    }
        
}

unsafe impl<EntityVariants, UtilityObjectVariants, ScoreType> Send for 
Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send + Serialize {}

impl<EntityVariants, UtilityObjectVariants, ScoreType> ObservableTrait 
for Agent<EntityVariants, UtilityObjectVariants, ScoreType>
where
    EntityVariants: CotwinEntityTrait,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send + Serialize {

        // Solver gets observers as arguments of solve. This is stub implementation just for pattern Observer be "clean".
        fn register_observer(&mut self, observer: Box<dyn ObserverTrait>){}

        fn notify_observers(&self, solution: Value) {
            
            match &mut (*self.observers.lock().unwrap()) {
                None => (),
                Some(observers) => {
                    for observer in observers {
                        observer.update(solution.clone());
                    }
                }
            }
        }

}