

use std::collections::HashMap;
use std::ops::AddAssign;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::agents::base::{Agent, AgentStatuses, AgentToAgentUpdate, Individual};
use crate::agents::base::AgentStatuses::*;
use crate::agents::AgentBuildersVariants;
use crate::cotwin::{CotwinBuilderTrait, CotwinEntityTrait};
use crate::score_calculation::scores::ScoreTrait;
use crossbeam_channel::*;
use ndarray::Array1;
use rayon::prelude::*;
use std::env;
use serde::Serialize;
use serde_json::Value;

use super::ObserverTrait;
use super::ObservableTrait;

#[derive(Clone)]
pub enum SolverLoggingLevels {
    Info,
    Warn,
    Silent
}

pub struct Solver {}

impl Solver {

    pub fn solve<DomainType, CotwinBuilder, EntityVariants, UtilityObjectVariants, ScoreType> (
        domain: &DomainType,
        cotwin_builder: CotwinBuilder,
        agent_builder: AgentBuildersVariants<ScoreType>,
        n_jobs: usize,
        score_precision: Option<Vec<u64>>,
        logging_level: SolverLoggingLevels,
        observers: Option<Vec<Box<dyn ObserverTrait + Send>>>
    ) -> Value
    where
    DomainType: Clone + Send,
    CotwinBuilder: CotwinBuilderTrait<DomainType, EntityVariants, UtilityObjectVariants, ScoreType> + Clone + Send,
    EntityVariants: CotwinEntityTrait + Send,
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send + Serialize {

        /* 
            Returns serde_json::Value to use it as json string (solution.to_string()) to send via http or
            parse it directly by serde_json::from_value(solution).unwrap() and use it in DomainUpdater.
            Note: serde_json::from_value needs type annotation. For solution it's (Vec<(String, AnyValue)>, ScoreType),
            where ScoreType is current score type used for task.
        */

        match &score_precision {
            Some(precision) => {
                if precision.len() != ScoreType::precision_len() {
                    panic!("Invalid score_precision. Suggest: vec![a] for SimpleScore, vec![a, b] for HardSoft, vec![a, b, c] for HardMediumSoft.");
                }
            }
            None => ()
        }

        let agent_ids:Vec<usize> = (0..n_jobs).collect();
        let domains: Vec<DomainType> = vec![domain.clone(); n_jobs];
        let cotwin_builders: Vec<CotwinBuilder> = vec![cotwin_builder.clone(); n_jobs];
        let agent_builders: Vec<AgentBuildersVariants<ScoreType>> = vec![agent_builder.clone(); n_jobs];
        let score_precisions = vec![score_precision; n_jobs];
        let logging_levels = vec![logging_level; n_jobs];
        let mut round_robin_status_vec: Vec<AgentStatuses> = Vec::new();
        let mut agents_updates_senders: Vec<Sender<AgentToAgentUpdate<ScoreType>>> = Vec::new();
        let mut agents_updates_receivers: Vec<Receiver<AgentToAgentUpdate<ScoreType>>> = Vec::new();
        let global_top_individual: Individual<ScoreType> = Individual::new(Array1::from_vec(vec![1.0]), ScoreType::get_stub_score());
        let global_top_individual = Arc::new(Mutex::new(global_top_individual));
        let global_top_json = Arc::new(Mutex::new(Value::Null));
        
        let observers_counts: Vec<usize>;
        let observers_arc:Arc<Mutex<Option<Vec<Box<dyn ObserverTrait + Send>>>>>;
        match observers {
            None => {
                observers_counts = vec![0; n_jobs];
                observers_arc = Arc::new(Mutex::new(None));
            }
            Some(observers) => {
                observers_counts = vec![observers.len(); n_jobs];
                observers_arc = Arc::new(Mutex::new(Some(observers)));
            }
        }

        for i in 0..n_jobs {
            round_robin_status_vec.insert(i, AgentStatuses::Alive);
            let (agent_i_updates_sender, agent_i_updates_receiver): (Sender<AgentToAgentUpdate<ScoreType>>, Receiver<AgentToAgentUpdate<ScoreType>>) = bounded(1);
            agents_updates_senders.push(agent_i_updates_sender);
            agents_updates_receivers.push(agent_i_updates_receiver);
        }
        let agents_round_robin_status_clones = vec![round_robin_status_vec.clone(); n_jobs];
        agents_updates_receivers.rotate_right(1);

        domains.into_par_iter()
        .zip(cotwin_builders.into_par_iter())
        .zip(agent_builders.into_par_iter())
        .zip(agent_ids.into_par_iter())
        .zip(agents_round_robin_status_clones.into_par_iter())
        .zip(agents_updates_senders.into_par_iter())
        .zip(agents_updates_receivers.into_par_iter())
        .zip(score_precisions.into_par_iter())
        .zip(logging_levels.into_par_iter())
        .zip(observers_counts.into_par_iter())
        .for_each(|(((((((((domain_i, cotwin_builder_i), agent_builder_i), id_i), rrs_i), us_i), rc_i), sp), log_lev), oc)| {
            let cotwin_i = cotwin_builder_i.build_cotwin(domain_i);
            let mut agent_i;
            match agent_builder_i {
                AgentBuildersVariants::GA(ga_builder) => agent_i = ga_builder.build_agent(cotwin_i)
            }
            agent_i.agent_id = id_i;
            agent_i.score_precision = sp;
            agent_i.round_robin_status_vec = rrs_i;
            agent_i.alive_agents_count = n_jobs;
            agent_i.updates_to_agent_sender = Some(us_i);
            agent_i.updates_for_agent_receiver = Some(rc_i);
            agent_i.global_top_individual = Arc::clone(&global_top_individual);
            agent_i.global_top_json = Arc::clone(&global_top_json);
            agent_i.logging_level = log_lev;
            agent_i.observers = observers_arc.clone();
            agent_i.observers_count = oc;
            
            //env::set_var("POLARS_MAX_THREADS",  (24 * n_jobs).to_string());
            agent_i.solve();
        });


        let solution_json =  global_top_json.lock().unwrap().clone();
        return solution_json;

    }
}